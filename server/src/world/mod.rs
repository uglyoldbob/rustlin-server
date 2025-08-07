use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod item;
pub mod npc;
pub mod object;

use common::packet::{ServerPacket, ServerPacketSender};

use crate::{
    character::Character,
    server::ClientError,
    server_message::ServerMessage,
    world::{item::ItemTrait, object::ObjectTrait},
};

/// Represents a single map of the world
#[derive(Debug)]
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

/// Represents the dynamic information of a map
#[derive(Debug)]
pub struct MapInfo {
    objects: HashMap<u32, object::Object>,
}

impl MapInfo {
    /// Construct a new map info object
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
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
    /// Used to broadcast server messages to the entire server
    pub global_tx: tokio::sync::broadcast::Sender<crate::ServerMessage>,
    /// The connection to the database
    mysql: mysql_async::Pool,
    /// maps of the world
    maps: Arc<Mutex<HashMap<u16, Map>>>,
    /// dynamic information for all maps
    map_info: Arc<tokio::sync::Mutex<HashMap<u16, MapInfo>>>,
    /// The item lookup table
    pub item_table: Arc<Mutex<HashMap<u32, item::Item>>>,
}

impl World {
    /// Construct a new server world
    pub fn new(
        global_tx: tokio::sync::broadcast::Sender<crate::ServerMessage>,
        mysql: mysql_async::Pool,
    ) -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
            global_tx,
            mysql,
            maps: Arc::new(Mutex::new(HashMap::new())),
            map_info: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            item_table: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a player to the world
    pub async fn add_player(&self, p: crate::character::FullCharacter) {
        let location = p.location_ref().clone();
        let obj: object::Object = p.into();
        let id = obj.id();
        let mut m = self.map_info.lock().await;
        let m2 = m.get_mut(&location.map);
        if let Some(map) = m2 {
            map.objects.insert(id, obj);
            log::info!("The objects on the map are {:?}", map);
        }
    }

    /// Run an asynchronous closure on objects close enough to the specified object
    pub async fn with_objects_nearby_do<F, T, E>(
        &self,
        refo: &object::Object,
        distance: f32,
        gen: &mut T,
        f: F,
    ) -> Result<(), E>
    where
        F: AsyncFn(&object::Object, &mut T) -> Result<(), E>,
    {
        let mi = self.map_info.lock().await;
        let map = mi.get(&refo.get_location().map);
        if let Some(map) = map {
            for obj in map.objects.values() {
                if obj.linear_distance_to(refo) < distance && refo.id() != obj.id() {
                    f(obj, gen).await?;
                }
            }
        }
        Ok(())
    }

    /// (Re)load all item data from database
    pub async fn load_item_data(&self) -> Result<(), String> {
        let mut item_table = self.item_table.lock().map_err(|e| e.to_string())?;
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from weapon";
        let mut mysql = self.get_mysql_conn().await.map_err(|e| e.to_string())?;
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let weapons = mysql
            .exec_map(s, (), |a: item::Weapon| a)
            .await
            .map_err(|e| e.to_string())?;
        for w in weapons {
            item_table.insert(w.id(), w.into());
        }
        log::info!("Items: {:?}", item_table);
        Ok(())
    }

    /// (Re)load all maps from the database
    pub async fn load_maps_data(&self) -> Result<(), String> {
        let mut hmaps = self.maps.lock().unwrap();
        use mysql_async::prelude::Queryable;
        let query = "SELECT mapid, locationname, startX, endX, startY, endY, monster_amount, drop_rate, underwater, markable, teleportable, escapable, resurrection, painwand, penalty, take_pets, recall_pets, usable_item, usable_skill from mapids";
        let mut mysql = self.get_mysql_conn().await.map_err(|e| e.to_string())?;
        let s = mysql.prep(query).await.map_err(|e| e.to_string())?;
        let maps = mysql
            .exec_map(s, (), |a: Map| a)
            .await
            .map_err(|e| e.to_string())?;
        let mut hdata = self.map_info.lock().await;
        /// TODO do something when an object is on a map that no longer exists
        hmaps.clear();
        for m in maps {
            println!("Found map data {:?}", m);
            if !hdata.contains_key(&m.id) {
                hdata.insert(m.id, MapInfo::new());
            }
            hmaps.insert(m.id, m);
        }
        if let Some(m) = hdata.get_mut(&4) {
            m.objects.insert(
                3,
                npc::Npc::new(
                    3,
                    crate::character::Location {
                        x: 33430,
                        y: 32820,
                        direction: 7,
                        map: 4,
                    },
                )
                .into(),
            );
        }
        Ok(())
    }

    /// Insert a character id into the world
    pub async fn insert_id(&self, id: u32, account: String) -> Result<(), ClientError> {
        let mut u = self.users.lock().unwrap();
        u.insert(id, account);
        Ok(())
    }

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
        let a: Vec<u32> = t.exec(query, ()).await?;
        let r = if let Some(a) = a.first() {
            Ok(Some(*a))
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

    pub fn get_broadcast_rx(&self) -> tokio::sync::broadcast::Receiver<crate::ServerMessage> {
        self.global_tx.subscribe()
    }

    /// Process a single message for the server.
    pub async fn handle_server_message(
        &self,
        p: ServerMessage,
        packet_writer: &mut ServerPacketSender,
    ) -> Result<u8, ClientError> {
        match p {
            ServerMessage::Disconnect => {
                packet_writer
                    .send_packet(ServerPacket::Disconnect.build())
                    .await?;
            }
            ServerMessage::RegularChat { id, msg } => {
                packet_writer
                    .send_packet(ServerPacket::RegularChat { id: id, msg: msg }.build())
                    .await?;
            }
            ServerMessage::WhisperChat(name, msg) => {
                packet_writer
                    .send_packet(
                        ServerPacket::WhisperChat {
                            name: name,
                            msg: msg,
                        }
                        .build(),
                    )
                    .await?;
            }
            ServerMessage::YellChat { id, msg, x, y } => {
                packet_writer
                    .send_packet(
                        ServerPacket::YellChat {
                            id: id,
                            msg: msg,
                            x: x,
                            y: y,
                        }
                        .build(),
                    )
                    .await?;
            }
            ServerMessage::GlobalChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::GlobalChat(m).build())
                    .await?;
            }
            ServerMessage::PledgeChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::PledgeChat(m).build())
                    .await?;
            }
            ServerMessage::PartyChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::PartyChat(m).build())
                    .await?;
            }
        }
        Ok(0)
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
