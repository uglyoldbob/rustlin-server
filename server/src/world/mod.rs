//! Represents the world in the server

use std::{collections::HashMap, future::AsyncDrop, pin::Pin, sync::Arc};

use parking_lot::FairMutex as Mutex;

pub mod item;
pub mod map_info;
pub mod monster;
pub mod npc;
pub mod object;

use common::packet::{ServerPacket, ServerPacketSender};

use crate::{
    character::{Character, Location},
    server::ClientError,
    world::{
        item::ItemTrait,
        object::{ObjectList, ObjectTrait},
    },
};

/// The id for an object that exists in the world
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorldObjectId(u32);

impl WorldObjectId {
    /// Get the u32 that corresponds to the object id
    pub fn get_u32(&self) -> u32 {
        self.0
    }
}

/// Represents a single map of the world
#[derive(Clone, Debug)]
pub struct Map {
    /// The mapid
    id: u16,
    /// The name of the map
    name: String,
    /// The minimum x coordinate for the map
    min_x: u16,
    /// The maximum x coordinate for the map
    max_x: u16,
    /// The minimum y coordinate for the map
    min_y: u16,
    /// The maximum y coordinate for the map
    max_y: u16,
    /// The rate multiplier for monsters
    monster_rate: f32,
    /// The drop rate multiplier for items from monsters
    drop_rate: f32,
    /// Is the map underwater?
    underwater: bool,
    /// Can players make bookmarks on this map?
    bookmarkable: bool,
    /// Does random teleport work on this map?
    random_teleport: bool,
    /// Is this map escapable?
    escapable: bool,
    /// Does resurrection work on this map?
    resurrection: bool,
    /// Do spawn monster items work here?
    spawn_monster: bool,
    /// Does this map impose an experience penalty upon death?
    death_exp_penalty: bool,
    /// Can pets come to this map?
    pets: bool,
    /// Can monsters be summoned on this map?
    summon_monster: bool,
    /// Is item usage allowed on this map?
    item_usage: bool,
    /// Are skills allowed on this map?
    skill_usage: bool,
}

impl Map {
    /// Can players use items on this map?
    pub fn can_use_items(&self) -> bool {
        self.item_usage
    }
}

/// A reference to an object in the world.
/// TODO: add a World instance to this struct
#[derive(Clone, Copy, Debug)]
pub struct ObjectRef {
    /// The map where the object is located
    map: u16,
    /// The id for the object in the world
    id: WorldObjectId,
}

impl ObjectRef {
    /// Get the map id
    pub fn map(&self) -> u16 {
        self.map
    }

    /// get the world id
    pub fn world_id(&self) -> WorldObjectId {
        self.id
    }
}

impl mysql_async::prelude::FromRow for Map {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            min_x: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_x: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            min_y: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_y: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            monster_rate: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            drop_rate: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            underwater: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            bookmarkable: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            random_teleport: row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
            escapable: row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?,
            resurrection: row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
            spawn_monster: row.get(13).ok_or(mysql_async::FromRowError(row.clone()))?,
            death_exp_penalty: row.get(14).ok_or(mysql_async::FromRowError(row.clone()))?,
            pets: row.get(15).ok_or(mysql_async::FromRowError(row.clone()))?,
            summon_monster: row.get(16).ok_or(mysql_async::FromRowError(row.clone()))?,
            item_usage: row.get(17).ok_or(mysql_async::FromRowError(row.clone()))?,
            skill_usage: row.get(18).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

pub struct WorldIdGenerator {
    next: WorldObjectId,
}

impl WorldIdGenerator {
    /// Build a new generator
    pub fn new(start: u32) -> Self {
        Self {
            next: WorldObjectId(start),
        }
    }

    /// Get a new object id
    pub fn new_id(&mut self) -> WorldObjectId {
        let r = self.next;
        self.next.0 += 1;
        r
    }
}

/// Represents the world for a server
pub struct World {
    /// The users logged into the world
    users: Arc<Mutex<HashMap<u32, String>>>,
    /// The id generator for users
    client_ids: Arc<Mutex<crate::ClientList>>,
    /// The connection to the database
    mysql: mysql_async::Pool,
    /// maps of the world
    maps: HashMap<u16, Map>,
    /// dynamic information for all maps
    map_info: HashMap<u16, Arc<Mutex<map_info::MapInfo>>>,
    /// The item lookup table
    pub item_table: Arc<Mutex<HashMap<u32, item::Item>>>,
    /// The npc lookup table
    pub npc_table: HashMap<u32, npc::NpcDefinition>,
    /// The npc spawn table
    npc_spawn_table: Vec<npc::NpcSpawn>,
    /// The monster spawn table
    monster_spawn_table: Vec<monster::MonsterSpawn>,
    /// The object for generating object ids
    id_generator: Arc<Mutex<WorldIdGenerator>>,
    /// Monster tasks
    monster_set: Option<Arc<Mutex<tokio::task::JoinSet<()>>>>,
    /// The sender for special messages to the server
    server_s: tokio::sync::mpsc::Sender<crate::server_message::ServerShutdownMessage>,
}

impl Drop for World {
    fn drop(&mut self) {}
}

impl AsyncDrop for World {
    async fn drop(mut self: Pin<&mut Self>) {
        if let Some(m) = self.monster_set.take() {
            let mut m2 = m.lock();
            m2.abort_all();
        }
    }
}

impl World {
    /// Construct a new server world
    pub async fn new(
        mysql: mysql_async::Pool,
        server_s: tokio::sync::mpsc::Sender<crate::server_message::ServerShutdownMessage>,
    ) -> Result<Self, String> {
        let mut conn = mysql.get_conn().await.map_err(|e| format!("{:?}", e))?;
        let npc_spawn_table = npc::NpcSpawn::load_table(&mut conn)
            .await
            .map_err(|e| format!("{:?}", e))?;
        let monster_spawn_table = monster::MonsterSpawn::load_table(&mut conn)
            .await
            .map_err(|e| format!("{:?}", e))?;
        let (mapd, mapi) = Self::load_maps_data(&mut conn).await?;
        let id_generator = Arc::new(Mutex::new(WorldIdGenerator::new(1)));
        let items = Self::load_item_data(&mut conn, &id_generator).await?;
        let npc = npc::NpcDefinition::load_table(&mut conn).await?;
        let w = Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
            mysql,
            maps: mapd,
            map_info: mapi,
            item_table: Arc::new(Mutex::new(items)),
            npc_table: npc,
            npc_spawn_table,
            monster_spawn_table,
            id_generator,
            monster_set: Some(Arc::new(Mutex::new(tokio::task::JoinSet::new()))),
            server_s,
        };
        {
            let mut idgen = w.id_generator.lock();
            for s in &w.npc_spawn_table {
                let new_id = idgen.new_id();
                let npc = s.make_npc(new_id, &w.npc_table);
                let o: object::Object = npc.into();
                let mapid = o.get_location().map;
                if let Some(map) = w.map_info.get(&mapid) {
                    map.lock().add_new_object(o);
                }
            }
        }
        Ok(w)
    }

    /// Get a new object id
    pub fn new_object_id(&self) -> WorldObjectId {
        let mut idgen = self.id_generator.lock();
        idgen.new_id()
    }

    /// Get a location of an object reference
    pub fn get_location(&self, r: ObjectRef) -> Option<Location> {
        let map = self.map_info.get(&r.map);
        if let Some(map) = map {
            return map.lock().get_location(r);
        }
        None
    }

    /// Shutdown the server if the player is authorized to do so
    pub async fn shutdown(&self, r: &ObjectRef) {
        let shutdown = {
            let map = self.map_info.get(&r.map);
            if let Some(map) = map {
                let map = map.lock();
                if let Some(obj) = map.get_object_from_id(r.id) {
                    obj.lock().can_shutdown()
                } else {
                    false
                }
            } else {
                false
            }
        };
        if shutdown {
            let _ = self
                .server_s
                .send(crate::server_message::ServerShutdownMessage::Shutdown)
                .await;
        }
    }

    /// Restart the server if the player is authorized to do so
    pub async fn restart(&self, r: &ObjectRef) {
        let shutdown = {
            let map = self.map_info.get(&r.map);
            if let Some(map) = map {
                let map = map.lock();
                if let Some(obj) = map.get_object_from_id(r.id) {
                    obj.lock().can_shutdown()
                } else {
                    false
                }
            } else {
                false
            }
        };
        if shutdown {
            let _ = self
                .server_s
                .send(crate::server_message::ServerShutdownMessage::Restart)
                .await;
        }
    }

    /// Get an object from the world
    pub async fn get_object(&self, r: ObjectRef) -> Option<Arc<Mutex<object::Object>>> {
        let map = self.map_info.get(&r.map);
        if let Some(map) = map {
            return map.lock().get_object(r);
        }
        None
    }

    /// Spawn all monsters
    pub async fn spawn_monsters(self: &Arc<Self>) {
        if let Some(mset) = &self.monster_set {
            let mut monsters = Vec::new();
            let mut idgen = self.id_generator.lock();
            for ms in &self.monster_spawn_table {
                let m = ms.make_monster(idgen.new_id(), &self.npc_table);
                monsters.push(m);
            }
            {
                let mut mset = mset.lock();
                for m in &monsters {
                    let mut monref = m.reference(self.clone());
                    mset.spawn(async move { monref.run_ai().await });
                }
            }
            {
                log::info!("There are {} monsters to spawn", monsters.len());
                for m in monsters {
                    if let Some(map) = self.map_info.get(&m.get_location().map) {
                        let mut map = map.lock();
                        map.add_new_object(m.into());
                    }
                }
            }
        }
    }

    /// Move an object on the world to a new location
    pub async fn move_object(
        &self,
        r: ObjectRef,
        new_loc: Location,
        pw: Option<&mut ServerPacketSender>,
        list: &mut map_info::SendsToAnotherObject,
    ) -> Result<(), ClientError> {
        let map = self.map_info.get(&new_loc.map);
        if let Some(map) = map {
            map.lock().move_object(r, new_loc, pw, list)?;
        }
        Ok(())
    }

    /// Send a new object packet with the given packet writer and object id
    pub async fn send_new_object(
        &self,
        location: crate::character::Location,
        id: WorldObjectId,
        pw: &mut common::packet::ServerPacketSender,
    ) -> Result<(), ClientError> {
        let map = self.map_info.get(&location.map);
        if let Some(map) = map {
            let map = map.lock();
            if let Some(obj) = map.get_object_from_id(id) {
                let p = obj.lock().build_put_object_packet();
                pw.queue_packet(p);
            }
        }
        Ok(())
    }

    /// Add a player to the world
    pub fn add_player(
        &self,
        p: crate::character::FullCharacter,
        pw: &mut ServerPacketSender,
        list: &mut map_info::SendsToAnotherObject,
    ) -> Option<ObjectRef> {
        let location = p.location_ref().to_owned();

        pw.queue_packet(p.details_packet());
        pw.queue_packet(p.get_map_packet());
        pw.queue_packet(p.get_object_packet());
        p.send_all_items(pw).ok()?;
        pw.queue_packet(ServerPacket::CharSpMrBonus { sp: 0, mr: 0 });
        pw.queue_packet(ServerPacket::Weather(0));

        let obj: object::Object = p.into();
        let id = obj.id();

        let m2 = self.map_info.get(&location.map);
        log::error!("add player 1");
        if let Some(map) = m2 {
            let mut map = map.lock();
            log::error!("add player 2");
            let location = obj.get_location();
            map.add_new_object(obj);
            log::error!("add player 3");
            let or = ObjectRef {
                map: location.map,
                id,
            };
            log::error!("add player 4");
            for o in map
                .get_object(or)
                .unwrap()
                .lock()
                .get_known_objects()
                .unwrap()
                .get_objects()
            {
                log::error!("Player knows about object {:?}", o);
            }
            map.move_object(or, location, Some(pw), list).ok()?;
            log::error!("add player 5");
            Some(or)
        } else {
            log::error!("add player 6");
            None
        }
    }

    /// Remove a player from the world
    pub async fn remove_player(&self, r: &mut Option<ObjectRef>) {
        if let Some(r) = &r {
            let map = self.map_info.get(&r.map);
            if let Some(map) = map {
                map.lock().remove_object(r.id);
            }
        }
        *r = None;
    }

    /// Run an asynchronous closure on the player object
    pub async fn with_player_ref_do<F, T, E>(&self, refo: ObjectRef, gen: &mut T, f: F) -> Option<E>
    where
        F: Fn(&crate::character::FullCharacter, &mut T, &Map) -> Option<E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            if let Some(obj) = map.get_object(refo) {
                let obj2 = obj.lock();
                let obj2: &object::Object = &obj2;
                if let object::Object::Player(fc) = obj2 {
                    let themap = self.maps.get(&refo.map).unwrap().clone();
                    return f(fc, gen, &themap);
                }
            }
        }
        None
    }

    /// Run an asynchronous closure on the player object
    pub async fn with_player_mut_do<F, T, E>(&self, refo: ObjectRef, gen: &mut T, f: F) -> Option<E>
    where
        F: Fn(&mut crate::character::FullCharacter, &mut T, &Map) -> Option<E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            if let Some(obj) = map.get_object(refo) {
                let mut obj2 = obj.lock();
                let obj2: &mut object::Object = &mut obj2;
                if let object::Object::Player(fc) = obj2 {
                    let themap = self.maps.get(&refo.map).unwrap().clone();
                    return f(fc, gen, &themap);
                }
            }
        }
        None
    }

    /// Run an asynchronous closure on objects on the same screen as the specified player ref
    pub async fn with_objects_on_screen_do<F, T, E>(
        &self,
        refo: &ObjectRef,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: Fn(&object::Object, &mut T, &Map) -> Result<(), E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            let me = map.get_object(*refo).unwrap();
            let mylocation = {
                let m2 = me.lock();
                m2.get_location()
            };
            let themap = self.maps.get(&refo.map).unwrap().clone();
            for (id, obj) in map.objects_iter() {
                if *id != refo.id {
                    let obj = obj.lock();
                    // TODO a better algorithm for on screen calculation
                    if obj.manhattan_distance(&mylocation) < 17 {
                        f(&obj, gen, &themap)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Send a packet to the specified player
    pub async fn send_packet_to(
        &self,
        other_person: &str,
        m: common::packet::ServerPacket,
    ) -> Result<(), String> {
        let mut send = None;
        {
            for map in self.map_info.values() {
                for obj in map.lock().objects_iter() {
                    let mut o = obj.1.lock();
                    if let Some(name) = o.player_name() {
                        if name == other_person {
                            if let Some(sender) = o.sender() {
                                send = Some(sender);
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Some(sender) = send {
            let _ = sender.send(m).await;
            Ok(())
        } else {
            Err(format!("Character not online right now: {:?}", m))
        }
    }

    /// Send a global chat message
    pub async fn send_global_chat(&self, m: common::packet::ServerPacket) {
        let mut p = Vec::new();
        {
            for map in self.map_info.values() {
                for obj in map.lock().objects_iter() {
                    let mut o = obj.1.lock();
                    if let Some(sender) = o.sender() {
                        p.push((sender, m.clone()));
                    }
                }
            }
        }
        for (sender, p) in p {
            sender.send(p).await;
        }
    }

    /// Run an asynchronous closure on objects close enough to the specified object
    pub async fn with_mut_objects_near_me_do<F, T, E>(
        &self,
        refo: &ObjectRef,
        distance: f32,
        include_self: bool,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: Fn(&mut object::Object, &mut T) -> Result<(), E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            let my_location = map.get_location(*refo).unwrap();
            for (k, obj) in map.objects_iter() {
                if include_self || *k != refo.id {
                    let mut obj = obj.lock();
                    if obj.linear_distance(&my_location) < distance {
                        f(&mut obj, gen)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Run an asynchronous closure on objects close enough to the specified player ref
    pub async fn with_objects_near_me_do<F, T, E>(
        &self,
        refo: &ObjectRef,
        distance: f32,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: Fn(&object::Object, &mut T, &Map) -> Result<(), E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            let me = map.get_object(*refo).unwrap();
            let mylocation = {
                let m2 = me.lock();
                m2.get_location()
            };
            let themap = self.maps.get(&refo.map).unwrap().clone();
            for (id, obj) in map.objects_iter() {
                if *id != refo.id {
                    let obj = obj.lock();
                    if obj.linear_distance(&mylocation) < distance {
                        f(&obj, gen, &themap)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Run an asynchronous closure on objects close enough to the specified object
    pub async fn with_objects_nearby_do<F, T, E>(
        &self,
        refo: ObjectRef,
        distance: f32,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: Fn(&object::Object, &mut T) -> Result<(), E>,
    {
        let map = self.map_info.get(&refo.map);
        if let Some(map) = map {
            let map = map.lock();
            let me = map.get_object(refo).unwrap();
            let mylocation = {
                let m2 = me.lock();
                m2.get_location()
            };
            for (id, obj) in map.objects_iter() {
                if *id != refo.id {
                    let obj = obj.lock();
                    if obj.linear_distance(&mylocation) < distance {
                        f(&obj, gen)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// (Re)load the weapons table
    pub async fn load_weapons(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &Arc<Mutex<WorldIdGenerator>>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from weapon";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let weapons = mysql
            .exec_map(s, (), |a: item::Weapon| a)
            .await
            .map_err(|e| e.to_string())?;
        let mut idgen = next_object_id.lock();
        for w in weapons {
            let w = w.get_instance(idgen.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the etc items table
    pub async fn load_etc_items(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &Arc<Mutex<WorldIdGenerator>>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from etcitem";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::EtcItem| a)
            .await
            .map_err(|e| e.to_string())?;
        let mut idgen = next_object_id.lock();
        for w in items {
            let w = w.get_instance(idgen.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the armor table
    pub async fn load_armor(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &Arc<Mutex<WorldIdGenerator>>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from armor";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::Armor| a)
            .await
            .map_err(|e| e.to_string())?;
        let mut idgen = next_object_id.lock();
        for w in items {
            let w = w.get_instance(idgen.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load all item data from database
    pub async fn load_item_data(
        mysql: &mut mysql_async::Conn,
        next_object_id: &Arc<Mutex<WorldIdGenerator>>,
    ) -> Result<HashMap<u32, item::Item>, String> {
        let mut item_table = HashMap::new();
        log::info!("There are {} items", item_table.len());
        Self::load_weapons(mysql, &mut item_table, next_object_id).await?;
        log::info!("There are {} items", item_table.len());
        Self::load_etc_items(mysql, &mut item_table, next_object_id).await?;
        log::info!("There are {} items", item_table.len());
        Self::load_armor(mysql, &mut item_table, next_object_id).await?;
        log::info!("There are {} items", item_table.len());
        Ok(item_table)
    }

    /// (Re)load all maps from the database
    pub async fn load_maps_data(
        mysql: &mut mysql_async::Conn,
    ) -> Result<(HashMap<u16, Map>, HashMap<u16, Arc<Mutex<map_info::MapInfo>>>), String> {
        let mut hmaps = HashMap::new();
        use mysql_async::prelude::Queryable;
        let query = "SELECT mapid, locationname, startX, endX, startY, endY, monster_amount, drop_rate, underwater, markable, teleportable, escapable, resurrection, painwand, penalty, take_pets, recall_pets, usable_item, usable_skill from mapids";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let maps = mysql
            .exec_map(s, (), |a: Map| a)
            .await
            .map_err(|e| e.to_string())?;
        let mut hdata = HashMap::new();
        for m in maps {
            hdata.entry(m.id).or_insert_with(|| Arc::new(Mutex::new(map_info::MapInfo::new())));
            hmaps.insert(m.id, m);
        }
        Ok((hmaps, hdata))
    }

    /// Insert a character id into the world
    pub async fn insert_id(&self, id: u32, account: String) -> Result<(), ClientError> {
        let mut u = self.users.lock();
        u.insert(id, account);
        Ok(())
    }

    /// lookup account name from user id
    pub async fn lookup_id(&self, id: u32) -> Option<String> {
        let u = self.users.lock();
        u.get(&id).map(|e| e.to_owned())
    }

    /// Get a new object id as part of a transaction.
    /// This prevents atomicity problems where two threads can get the same new id, and try to insert the same id into the database.
    /// # Arguments:
    /// * t - The transaction object
    pub async fn get_new_id(
        t: &mut mysql_async::Transaction<'_>,
    ) -> Result<Option<u32>, mysql_async::Error> {
        use mysql_async::prelude::Queryable;
        let query = "select max(id)+1 as nextid from (select id from character_items union all select id from character_teleport union all select id from character_warehouse union all select id from character_elf_warehouse union all select objid as id from characters union all select clan_id as id from clan_data union all select id from clan_warehouse union all select objid as id from pets) t";
        let a: Vec<Option<u32>> = t.exec(query, ()).await?;
        let r = if let Some(a) = a.first() {
            Ok(*a)
        } else {
            Ok(None)
        };
        r
    }

    /// Get a connection to the database
    pub async fn get_mysql_conn(&self) -> Result<mysql_async::Conn, mysql_async::Error> {
        self.mysql.get_conn().await
    }

    /// Register a new user
    pub fn register_user(&self) -> u32 {
        let mut c = self.client_ids.lock();
        c.new_entry()
    }

    /// Save a new character into the database
    pub async fn save_new_character(&self, c: &mut Character) -> Result<(), ClientError> {
        let mut conn = self.get_mysql_conn().await?;
        c.save_new_to_db(&mut conn).await
    }

    /// Unregister a user
    pub fn unregister_user(&self, uid: u32) {
        let mut c = self.client_ids.lock();
        c.remove_entry(uid);
        let mut d = self.users.lock();
        d.remove(&uid);
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        let users = self.users.lock();
        users.len() as u16
    }
}
