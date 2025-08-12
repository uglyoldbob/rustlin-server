use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod item;
pub mod npc;
pub mod object;

use common::packet::{ServerPacket, ServerPacketSender};

use crate::{
    character::{Character, Location},
    server::ClientError,
    server_message::ServerMessage,
    world::{item::ItemTrait, object::{ObjectList, ObjectTrait}},
};

/// The id for an object that exists in the world
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorldObjectId(u32);

impl Into<u32> for WorldObjectId {
    fn into(self) -> u32 {
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

#[derive(Clone, Copy)]
pub struct ObjectRef {
    map: u16,
    id: WorldObjectId,
}

impl ObjectRef {
    /// Get the map id
    pub fn map(&self) -> u16 {
        self.map
    }
}

/// Represents the dynamic information of a map
#[derive(Debug)]
pub struct MapInfo {
    objects: HashMap<WorldObjectId, object::Object>,
}

impl MapInfo {
    /// Construct a new map info object
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    /// Add an object to the map
    pub async fn add_new_object(&mut self, new_o: object::Object) {
        self.objects.insert(new_o.id(), new_o);
    }

    /// Move an object on the map
    pub async fn move_object(&mut self, r: ObjectRef, new_loc: Location, pw: &mut ServerPacketSender) -> Result<(), ClientError> {
        let mut object_list = ObjectList::new();
        if let Some(myobj) = self.objects.get_mut(&r.id) {
            myobj.set_location(new_loc);
        }
        for (id, o) in &mut self.objects {
            if *id != r.id {
                if o.linear_distance(&new_loc) < 7.0 {
                    object_list.add_object(*id);
                }
            }
        }
        {
            let mut old_objects = Vec::new();
            let mut new_objects = Vec::new();
            if let Some(myobj) = self.objects.get_mut(&r.id) {
                if let Some(ol) = myobj.get_known_objects() {
                    ol.find_changes(&mut old_objects, &mut new_objects, &object_list);
                }
            }
            for objid in old_objects {
                pw.send_packet(ServerPacket::RemoveObject(objid.into()).build()).await?;
                if let Some(obj) = self.objects.get_mut(&r.id) {
                    pw.send_packet(ServerPacket::RemoveObject(objid.into()).build()).await?;
                    obj.remove_object(objid).await;
                }
            }
            for objid in new_objects {
                if let Some(obj) = self.objects.get_mut(&objid) {
                    pw.send_packet(obj.build_put_object_packet()).await?;
                }
                if let Some(obj) = self.objects.get_mut(&r.id) {
                    obj.add_object(objid).await;
                }
            }
        }
        Ok(())
    }

    /// Remove an object from the map
    pub async fn remove_object(&mut self, id: WorldObjectId) {
        self.objects.remove(&id);
        for o in &mut self.objects {
            o.1.remove_object(id).await;
        }
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
    map_info: Arc<tokio::sync::Mutex<HashMap<u16, MapInfo>>>,
    /// The item lookup table
    pub item_table: Arc<Mutex<HashMap<u32, item::Item>>>,
    /// The npc lookup table
    pub npc_table: HashMap<u32, npc::NpcDefinition>,
    /// The npc spawn table
    npc_spawn_table: Vec<npc::NpcSpawn>,
    /// The object for generating object ids
    next_object_id: Arc<Mutex<WorldObjectId>>,
}

impl World {
    /// Construct a new server world
    pub async fn new(mysql: mysql_async::Pool) -> Result<Self, String> {
        let mut conn = mysql.get_conn().await.map_err(|e| format!("{:?}", e))?;
        let npc_spawn_table = Self::load_npc_spawn_table(&mut conn)
            .await
            .map_err(|e| format!("{:?}", e))?;
        let (mapd, mapi) = Self::load_maps_data(&mut conn).await?;
        let items = Self::load_item_data(&mut conn).await?;
        let npc = npc::NpcDefinition::load_table(&mut conn).await?;
        let w = Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
            mysql,
            maps: mapd,
            map_info: Arc::new(tokio::sync::Mutex::new(mapi)),
            item_table: Arc::new(Mutex::new(items)),
            npc_table: npc,
            npc_spawn_table,
            next_object_id: Arc::new(Mutex::new(WorldObjectId(1))),
        };
        for s in &w.npc_spawn_table {
            let new_id = w.new_object_id();
            let npc = s.make_npc(new_id, &w.npc_table);
            let o: object::Object = npc.into();
            let mapid = o.get_location().map;
            let mut mi = w.map_info.lock().await;
            if let Some(map) = mi.get_mut(&mapid) {
                map.add_new_object(o).await;
            }
        }
        Ok(w)
    }

    /// Move an object on the world to a new location
    pub async fn move_object(&self, r: ObjectRef, new_loc: Location, pw: &mut ServerPacketSender) -> Result<(), ClientError> {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&new_loc.map);
        if let Some(map) = map {
            map.move_object(r, new_loc, pw).await?;
        }
        Ok(())
    }

    /// Get a new object id for an object that will live in the world somewhere
    pub fn new_object_id(&self) -> WorldObjectId {
        let mut w = self.next_object_id.lock().unwrap();
        let r = w.clone();
        w.0 += 1;
        r
    }

    async fn load_npc_spawn_table(
        conn: &mut mysql_async::Conn,
    ) -> Result<Vec<npc::NpcSpawn>, ClientError> {
        Ok(npc::NpcSpawn::load_table(conn).await?)
    }

    /// Send a new object packet with the given packet writer and object id
    pub async fn send_new_object(
        &self,
        location: crate::character::Location,
        id: WorldObjectId,
        pw: &mut common::packet::ServerPacketSender,
    ) -> Result<(), ClientError> {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&location.map);
        if let Some(map) = map {
            if let Some(obj) = map.objects.get(&id) {
                let p = obj.build_put_object_packet();
                pw.send_packet(p).await?;
            }
        }
        Ok(())
    }

    /// Add a player to the world
    pub async fn add_player(&self, p: crate::character::FullCharacter, pw: &mut ServerPacketSender) -> Option<ObjectRef> {
        let location = p.location_ref().clone();
        let obj: object::Object = p.into();
        let id = obj.id();
        let mut m = self.map_info.lock().await;
        let m2 = m.get_mut(&location.map);
        if let Some(map) = m2 {
            let location = obj.get_location();
            map.add_new_object(obj).await;
            let or = ObjectRef {
                map: location.map,
                id,
            };
            map.move_object(or, location, pw).await.ok()?;
            Some(or)
        } else {
            None
        }
    }

    /// Remove a player from the world
    pub async fn remove_player(&self, r: &mut Option<ObjectRef>) {
        if let Some(r) = &r {
            let mut mi = self.map_info.lock().await;
            let map = mi.get_mut(&r.map);
            if let Some(map) = map {
                map.remove_object(r.id).await;
            }
        }
        *r = None;
    }

    /// Run an asynchronous closure on the player object
    pub async fn with_player_ref_do<F, T, E>(&self, refo: ObjectRef, gen: &mut T, f: F) -> Option<E>
    where
        F: AsyncFn(&crate::character::FullCharacter, &mut T, &Map) -> Option<E>,
    {
        let mi = self.map_info.lock().await;
        let map = mi.get(&refo.map);
        if let Some(map) = map {
            if let Some(obj) = map.objects.get(&refo.id) {
                if let object::Object::Player(fc) = obj {
                    let themap = self.maps.get(&refo.map).unwrap().clone();
                    return f(fc, gen, &themap).await;
                }
            }
        }
        None
    }

    /// Run an asynchronous closure on the player object
    pub async fn with_player_mut_do<F, T, E>(&self, refo: ObjectRef, gen: &mut T, f: F) -> Option<E>
    where
        F: AsyncFn(&mut crate::character::FullCharacter, &mut T, &Map) -> Option<E>,
    {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&refo.map);
        if let Some(map) = map {
            if let Some(obj) = map.objects.get_mut(&refo.id) {
                if let object::Object::Player(fc) = obj {
                    let themap = self.maps.get(&refo.map).unwrap().clone();
                    return f(fc, gen, &themap).await;
                }
            }
        }
        None
    }

    /// Run an asynchronous closure on objects on the same screen as the specified player ref
    pub async fn with_objects_on_screen_do<F, T, E>(
        &self,
        refo: &ObjectRef,
        distance: f32,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: AsyncFn(&object::Object, &mut T, &Map) -> Result<(), E>,
    {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&refo.map);
        if let Some(map) = map {
            let my_location = map.objects.get(&refo.id).unwrap().get_location();
            let themap = self.maps.get(&refo.map).unwrap().clone();
            for obj in map.objects.values() {
                let d = obj.manhattan_distance(&my_location);
                // TODO a better algorithm for on screen calculation
                if d < 17 && refo.id != obj.id() {
                    f(obj, gen, &themap).await?;
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
        F: AsyncFn(&object::Object, &mut T, &Map) -> Result<(), E>,
    {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&refo.map);
        if let Some(map) = map {
            let my_location = map.objects.get(&refo.id).unwrap().get_location();
            let themap = self.maps.get(&refo.map).unwrap().clone();
            for obj in map.objects.values() {
                if obj.linear_distance(&my_location) < distance && refo.id != obj.id() {
                    f(obj, gen, &themap).await?;
                }
            }
        }
        Ok(())
    }

    /// Send a whisper message to the specified player
    pub async fn send_whisper_to(
        &self,
        other_person: &str,
        m: ServerMessage,
    ) -> Result<(), String> {
        let mut mi = self.map_info.lock().await;
        for map in mi.values_mut() {
            for obj in &mut map.objects {
                if let Some(name) = obj.1.player_name() {
                    if name == other_person {
                        if let Some(sender) = obj.1.sender() {
                            let _ = sender.send(m).await;
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(format!("Character not online right now: {:?}", m))
    }

    /// Send a global chat message
    pub async fn send_global_chat(&self, m: ServerMessage) {
        let mut mi = self.map_info.lock().await;
        for map in mi.values_mut() {
            for obj in &mut map.objects {
                if let Some(sender) = obj.1.sender() {
                    sender.send(m.clone()).await;
                }
            }
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
        F: AsyncFn(&mut object::Object, &mut T) -> Result<(), E>,
    {
        let mut mi = self.map_info.lock().await;
        let map = mi.get_mut(&refo.map);
        if let Some(map) = map {
            let my_location = map.objects.get(&refo.id).unwrap().get_location();
            for obj in map.objects.values_mut() {
                if obj.linear_distance(&my_location) < distance
                    && (include_self || refo.id != obj.id())
                {
                    f(obj, gen).await?;
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
        F: AsyncFn(&object::Object, &mut T) -> Result<(), E>,
    {
        let mi = self.map_info.lock().await;
        let map = mi.get(&refo.map);
        if let Some(map) = map {
            let mylocation = map.objects.get(&refo.id).map(|o| o.get_location()).unwrap();
            for obj in map.objects.values() {
                if obj.linear_distance(&mylocation) < distance && refo.id != obj.id() {
                    f(obj, gen).await?;
                }
            }
        }
        Ok(())
    }

    /// (Re)load the weapons table
    pub async fn load_weapons(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from weapon";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let weapons = mysql
            .exec_map(s, (), |a: item::Weapon| a)
            .await
            .map_err(|e| e.to_string())?;
        for w in weapons {
            item_table.insert(w.id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the etc items table
    pub async fn load_etc_items(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from etcitem";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::EtcItem| a)
            .await
            .map_err(|e| e.to_string())?;
        for w in items {
            item_table.insert(w.id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the armor table
    pub async fn load_armor(
        mysql: &mut mysql_async::Conn,
        item_table: &mut HashMap<u32, item::Item>,
    ) -> Result<(), String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from armor";
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::Armor| a)
            .await
            .map_err(|e| e.to_string())?;
        for w in items {
            item_table.insert(w.id(), w.into());
        }
        Ok(())
    }

    /// (Re)load all item data from database
    pub async fn load_item_data(
        mysql: &mut mysql_async::Conn,
    ) -> Result<HashMap<u32, item::Item>, String> {
        let mut item_table = HashMap::new();
        log::info!("There are {} items", item_table.len());
        Self::load_weapons(mysql, &mut item_table).await?;
        log::info!("There are {} items", item_table.len());
        Self::load_etc_items(mysql, &mut item_table).await?;
        log::info!("There are {} items", item_table.len());
        Self::load_armor(mysql, &mut item_table).await?;
        log::info!("There are {} items", item_table.len());
        Ok(item_table)
    }

    /// (Re)load all maps from the database
    pub async fn load_maps_data(
        mysql: &mut mysql_async::Conn,
    ) -> Result<(HashMap<u16, Map>, HashMap<u16, MapInfo>), String> {
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
            println!("Found map data {:?}", m);
            if !hdata.contains_key(&m.id) {
                hdata.insert(m.id, MapInfo::new());
            }
            hmaps.insert(m.id, m);
        }
        Ok((hmaps, hdata))
    }

    /// Insert a character id into the world
    pub async fn insert_id(&self, id: u32, account: String) -> Result<(), ClientError> {
        let mut u = self.users.lock().unwrap();
        u.insert(id, account);
        Ok(())
    }

    /// lookup account name from user id
    pub async fn lookup_id(&self, id: u32) -> Option<String> {
        let u = self.users.lock().unwrap();
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
        let mut c = self.client_ids.lock().unwrap();
        c.new_entry()
    }

    /// Save a new character into the database
    pub async fn save_new_character(&self, c: &mut Character) -> Result<(), ClientError> {
        let mut conn = self.get_mysql_conn().await?;
        c.save_new_to_db(&mut conn).await
    }

    /// Unregister a user
    pub fn unregister_user(&self, uid: u32) {
        let mut c = self.client_ids.lock().unwrap();
        c.remove_entry(uid);
        let mut d = self.users.lock().unwrap();
        d.remove(&uid);
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        let users = self.users.lock().unwrap();
        users.len() as u16
    }
}
