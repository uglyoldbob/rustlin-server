//! Represents the world in the server

use std::{collections::HashMap, future::AsyncDrop, pin::Pin, sync::Arc};

pub mod item;
pub mod map_info;
pub mod monster;
pub mod npc;
pub mod object;

use common::packet::{ClientPacket, ServerPacket, ServerPacketSender};

use crate::{
    character::{Character, FullCharacter, Location},
    server::ClientError,
    user::UserAccount,
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

impl mysql::prelude::FromRow for Map {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql::FromRowError(row.clone()))?,
            min_x: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            max_x: row.get(3).ok_or(mysql::FromRowError(row.clone()))?,
            min_y: row.get(4).ok_or(mysql::FromRowError(row.clone()))?,
            max_y: row.get(5).ok_or(mysql::FromRowError(row.clone()))?,
            monster_rate: row.get(6).ok_or(mysql::FromRowError(row.clone()))?,
            drop_rate: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            underwater: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            bookmarkable: row.get(9).ok_or(mysql::FromRowError(row.clone()))?,
            random_teleport: row.get(10).ok_or(mysql::FromRowError(row.clone()))?,
            escapable: row.get(11).ok_or(mysql::FromRowError(row.clone()))?,
            resurrection: row.get(12).ok_or(mysql::FromRowError(row.clone()))?,
            spawn_monster: row.get(13).ok_or(mysql::FromRowError(row.clone()))?,
            death_exp_penalty: row.get(14).ok_or(mysql::FromRowError(row.clone()))?,
            pets: row.get(15).ok_or(mysql::FromRowError(row.clone()))?,
            summon_monster: row.get(16).ok_or(mysql::FromRowError(row.clone()))?,
            item_usage: row.get(17).ok_or(mysql::FromRowError(row.clone()))?,
            skill_usage: row.get(18).ok_or(mysql::FromRowError(row.clone()))?,
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

#[derive(Debug)]
/// A message to the world
pub enum WorldMessageData {
    /// A packet from the client
    ClientPacket(common::packet::ClientPacket),
    /// Unregister a client by id
    UnregisterClient(u32),
    /// Register sender with a new client id
    RegisterSender(tokio::sync::mpsc::Sender<WorldResponse>),
}

#[derive(Debug)]
pub struct WorldMessage {
    pub data: WorldMessageData,
    pub sender: Option<u32>,
    pub peer: std::net::SocketAddr,
}

#[derive(Debug)]
pub enum WorldResponse {
    ServerPacket(common::packet::ServerPacket),
    NewClientId(u32),
}

/// Data to convey when the user wants to use an item
pub struct ItemUseData {
    /// the packet to process the item use command with
    pub p: common::packet::Packet,
    /// The packets to send
    pub packets: Vec<ServerPacket>,
}

/// Represents the world for a server
pub struct World {
    /// The users logged into the world
    users: HashMap<u32, String>,
    /// The id generator for users
    client_ids: crate::ClientList,
    /// The sender for each client registered
    object_senders: HashMap<u32, tokio::sync::mpsc::Sender<WorldResponse>>,
    /// A lookup table to convert object ids to object refs
    object_ref_table: HashMap<WorldObjectId, ObjectRef>,
    /// A lookup table to convert client ids to user accounts
    account_table: HashMap<u32, UserAccount>,
    /// The object id for each player
    characters: HashMap<u32, WorldObjectId>,
    /// The connection to the database
    mysql: mysql::Pool,
    /// maps of the world
    maps: HashMap<u16, Map>,
    /// dynamic information for all maps
    map_info: HashMap<u16, map_info::MapInfo>,
    /// The item lookup table
    pub item_table: HashMap<u32, item::Item>,
    /// The npc lookup table
    pub npc_table: HashMap<u32, npc::NpcDefinition>,
    /// The npc spawn table
    npc_spawn_table: Vec<npc::NpcSpawn>,
    /// The monster spawn table
    monster_spawn_table: Vec<monster::MonsterSpawn>,
    /// The object for generating object ids
    id_generator: WorldIdGenerator,
    /// Monster tasks
    monster_set: Option<tokio::task::JoinSet<()>>,
    /// The sender for special messages to the server
    server_s: tokio::sync::mpsc::Sender<crate::server_message::ServerShutdownMessage>,
    /// The receiver for messages affecting the world
    recv: tokio::sync::mpsc::Receiver<WorldMessage>,
    /// The sender for users that need to send worldMessage items
    sender: tokio::sync::mpsc::Sender<WorldMessage>,
    /// Configuration from the server config
    config: crate::ServerConfiguration,
}

impl Drop for World {
    fn drop(&mut self) {}
}

impl AsyncDrop for World {
    async fn drop(mut self: Pin<&mut Self>) {
        if let Some(mut m) = self.monster_set.take() {
            m.abort_all();
        }
    }
}

impl World {
    /// Construct a new server world
    pub fn new(
        mysql: mysql::Pool,
        server_s: tokio::sync::mpsc::Sender<crate::server_message::ServerShutdownMessage>,
        recv: tokio::sync::mpsc::Receiver<WorldMessage>,
        sender: tokio::sync::mpsc::Sender<WorldMessage>,
        config: crate::ServerConfiguration,
    ) -> Result<Self, String> {
        let mut conn = mysql.get_conn().map_err(|e| format!("{:?}", e))?;
        let npc_spawn_table =
            npc::NpcSpawn::load_table(&mut conn).map_err(|e| format!("{:?}", e))?;
        let monster_spawn_table =
            monster::MonsterSpawn::load_table(&mut conn).map_err(|e| format!("{:?}", e))?;
        let (mapd, mapi) = Self::load_maps_data(&mut conn)?;
        let mut id_generator = WorldIdGenerator::new(1);
        let items = Self::load_item_data(&mut conn, &mut id_generator)?;
        let npc = npc::NpcDefinition::load_table(&mut conn)?;
        let mut w = Self {
            users: HashMap::new(),
            client_ids: crate::ClientList::new(),
            object_ref_table: HashMap::new(),
            account_table: HashMap::new(),
            characters: HashMap::new(),
            mysql,
            maps: mapd,
            map_info: mapi,
            item_table: items,
            npc_table: npc,
            npc_spawn_table,
            monster_spawn_table,
            id_generator,
            monster_set: Some(tokio::task::JoinSet::new()),
            server_s,
            object_senders: HashMap::new(),
            recv,
            sender,
            config,
        };
        {
            for s in &w.npc_spawn_table {
                let new_id = w.id_generator.new_id();
                let npc = s.make_npc(new_id, &w.npc_table);
                let o: object::Object = npc.into();
                let mapid = o.get_location().map;
                if let Some(map) = w.map_info.get_mut(&mapid) {
                    map.add_new_object(o);
                }
            }
        }
        Ok(w)
    }

    /// Retrieve the sender for WorldMessage items
    pub fn get_sender(&self) -> tokio::sync::mpsc::Sender<WorldMessage> {
        self.sender.clone()
    }

    /// Run the login process, delivering news if applicable
    fn login_with_news(
        &mut self,
        id: u32,
        username: String,
        s: &mut tokio::sync::mpsc::Sender<WorldResponse>,
    ) -> Result<(), ClientError> {
        s.blocking_send(WorldResponse::ServerPacket(ServerPacket::LoginResult {
            code: 0,
        }));
        let news = self.config.get_news();
        if news.is_empty() {
            self.after_news(id, s)?;
        } else {
            s.blocking_send(WorldResponse::ServerPacket(ServerPacket::News(news)));
        }
        Ok(())
    }

    /// Send the client details that happens after the news (if there was any news at all)
    /// This still should be called even if there was no news.
    fn after_news(
        &mut self,
        id: u32,
        s: &mut tokio::sync::mpsc::Sender<WorldResponse>,
    ) -> Result<(), ClientError> {
        if let Some(account) = self.account_table.get(&id) {
            let mut conn = self.get_mysql_conn()?;
            let chars = account.retrieve_chars(&mut conn)?;
            log::info!("Characters are {:?}", chars);
            let response = ServerPacket::NumberCharacters(chars.len() as u8, 8);
            s.blocking_send(WorldResponse::ServerPacket(response));

            for c in &chars {
                let response = c.get_details_packet();
                s.blocking_send(WorldResponse::ServerPacket(response));
            }
        }
        Ok(())
    }

    /// Run the game world
    pub fn run(mut self) {
        while let Some(m) = self.recv.blocking_recv() {
            match m.data {
                WorldMessageData::RegisterSender(s) => {
                    let newid = self.client_ids.new_entry();
                    s.blocking_send(WorldResponse::NewClientId(newid));
                    self.object_senders.insert(newid, s);
                }
                WorldMessageData::UnregisterClient(id) => {
                    self.object_senders.remove(&id);
                    self.client_ids.remove_entry(id);
                }
                WorldMessageData::ClientPacket(client_packet) => match client_packet {
                    ClientPacket::AttackObject { id, x, y } => {
                        if let Some(sender) = m.sender {
                            if let Some(myid) = self.characters.get(&sender) {
                                if let Some(s) = self.object_senders.get_mut(&sender) {
                                    s.blocking_send(WorldResponse::ServerPacket(
                                        ServerPacket::Attack {
                                            attack_type: 3,
                                            id: myid.get_u32(),
                                            id2: id,
                                            impact: 1,
                                            direction: 2,
                                            effect: None,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    ClientPacket::UseItem { id, remainder } => {
                        log::info!("User wants to use item {}: {:X?}", id, remainder);
                        let p = common::packet::Packet::raw_packet(remainder);
                        let mut p2 = ItemUseData {
                            p,
                            packets: Vec::new(),
                        };
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        if let Some(obj) = map.get_object_mut(*re) {
                                            match obj {
                                                object::Object::Player(fc) => {
                                                    // Can't use items when you are dead
                                                    if fc.curr_hp() > 0 {
                                                        //TODO fc.use_item(&id, p2, map);
                                                    }
                                                }
                                                object::Object::GenericNpc(npc) => todo!(),
                                                object::Object::Monster(monster) => todo!(),
                                                object::Object::GroundItem(item_with_location) => {
                                                    todo!()
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::Ping(v) => {
                        log::info!("The user pinged us {v}");
                    }
                    ClientPacket::Restart => {
                        log::info!("Player restarts");
                        if let Some(sender) = m.sender {
                            self.characters.remove(&sender);
                            if let Some(s) = self.object_senders.get_mut(&sender) {
                                s.blocking_send(WorldResponse::ServerPacket(
                                    ServerPacket::BackToCharacterSelect,
                                ));
                            }
                        }
                    }
                    ClientPacket::RemoveFriend(name) => {
                        log::info!("User used the remove friend command with {name}");
                    }
                    ClientPacket::AddFriend(name) => {
                        log::info!("User used the add friend command with {name}");
                    }
                    ClientPacket::WhoCommand(name) => {
                        log::info!("User used the who command on {name}");
                    }
                    ClientPacket::CreateBookmark(n) => {
                        log::info!("User wants to create a bookmark named {n}");
                    }
                    ClientPacket::Version(a, b, c, d) => {
                        log::info!("version {} {} {} {}", a, b, c, d);
                        let response = ServerPacket::ServerVersion {
                            id: 2,
                            version: 0x16009,
                            time: 3,
                            new_accounts: 1,
                            english: 1,
                            country: 0,
                        };
                        if let Some(sender) = m.sender {
                            if let Some(s) = self.object_senders.get_mut(&sender) {
                                s.blocking_send(WorldResponse::ServerPacket(response));
                            }
                        }
                    }
                    ClientPacket::Login(u, p, v1, v2, v3, v4, v5, v6, v7) => {
                        log::info!(
                            "login attempt for {} {} {} {} {} {} {} {}",
                            &u,
                            v1,
                            v2,
                            v3,
                            v4,
                            v5,
                            v6,
                            v7
                        );
                        if let Some(sender) = m.sender {
                            if let Ok(mut mysql_conn) = self.get_mysql_conn() {
                                if let Some(s) = self.object_senders.get(&sender) {
                                    let mut s = s.clone();
                                    let user =
                                        crate::user::get_user_details(u.clone(), &mut mysql_conn);
                                    match user {
                                        Some(us) => {
                                            log::info!("User {} exists: {:?}", u.clone(), us);
                                            let password_success = us.check_login(
                                                &self.config.account_creation_salt,
                                                &p,
                                            );
                                            log::info!("User login check is {}", password_success);
                                            if password_success {
                                                self.account_table.insert(sender, us);
                                                self.login_with_news(sender, u, &mut s);
                                            } else {
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::LoginResult { code: 8 },
                                                ));
                                            }
                                        }
                                        None => {
                                            log::info!("User {} does not exist!", u.clone());
                                            if self.config.automatic_account_creation {
                                                let newaccount = UserAccount::new(
                                                    u.clone(),
                                                    p,
                                                    m.peer.to_string(),
                                                    self.config.account_creation_salt.clone(),
                                                );
                                                if let Ok(mut mysql) = self.get_mysql_conn() {
                                                    newaccount.insert_into_db(&mut mysql);
                                                }
                                                self.login_with_news(sender, u, &mut s);
                                            } else {
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::LoginResult { code: 8 },
                                                ));
                                            }
                                        }
                                    }
                                } else {
                                    todo!();
                                }
                            }
                        }
                    }
                    ClientPacket::NewsDone => {
                        //send number of characters the player has
                        if let Some(sender) = m.sender {
                            if let Some(s) = self.object_senders.get(&sender) {
                                let mut s = s.clone();
                                self.after_news(sender, &mut s);
                            }
                        }
                    }
                    ClientPacket::NewCharacter {
                        name,
                        class,
                        gender,
                        strength,
                        dexterity,
                        constitution,
                        wisdom,
                        charisma,
                        intelligence,
                    } => {
                        todo!();
                    }
                    ClientPacket::DeleteCharacter(n) => {
                        if let Some(sender) = m.sender {
                            if let Ok(mut mysql) = self.get_mysql_conn() {
                                if let Some(s) = self.object_senders.get(&sender) {
                                    let s = s.clone();
                                    if let Some(account) = self.account_table.get(&sender) {
                                        if let Ok(chars) = account.retrieve_chars(&mut mysql) {
                                            let char = chars.iter().find(|a| a.name() == n);
                                            if let Some(char) = char {
                                                if char.needs_delete_waiting() {
                                                    //TODO implement the actual delete in a scheduled async task
                                                    s.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::DeleteCharacterWait,
                                                    ));
                                                } else {
                                                    //TODO actually delete the character
                                                    s.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::DeleteCharacterOk,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::CharacterSelect { name } => {
                        log::info!("login with {}", name);
                        if let Some(sender) = m.sender {
                            if let Some(s) = self.object_senders.get(&sender) {
                                let mut s = s.clone();
                                let mut fco: Option<FullCharacter> = None;
                                if let Ok(mut mysql) = self.get_mysql_conn() {
                                    if let Some(account) = self.account_table.get(&sender) {
                                        if let Ok(chars) = account.retrieve_chars(&mut mysql) {
                                            let char = chars.iter().find(|a| a.name() == name);
                                            if let Some(char) = char {
                                                if let Ok(pc) = char.get_partial_details(
                                                    self.id_generator.new_id(),
                                                    &mut mysql,
                                                ) {
                                                    s.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::StartGame(0),
                                                    ));
                                                    let fc = pc.into_full(&self.item_table);
                                                    fco = Some(fc);
                                                } else {
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some(fc) = fco {
                                    self.add_player(fc, &mut s);
                                }
                            } else {
                                todo!();
                            }
                        }
                    }
                    ClientPacket::KeepAlive => {}
                    ClientPacket::GameInitDone => {}
                    ClientPacket::WindowActivate(v2) => {
                        log::info!("Client window activate {}", v2);
                    }
                    ClientPacket::Save => {}
                    ClientPacket::MoveFrom { x, y, heading } => {
                        let (x2, y2) = match heading {
                            0 => (x, y - 1),
                            1 => (x + 1, y - 1),
                            2 => (x + 1, y),
                            3 => (x + 1, y + 1),
                            4 => (x, y + 1),
                            5 => (x - 1, y + 1),
                            6 => (x - 1, y),
                            7 => (x - 1, y - 1),
                            _ => (x, y),
                        };
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        map.move_object(
                                            *re,
                                            crate::character::Location {
                                                map: re.map(),
                                                x: x2,
                                                y: y2,
                                                direction: heading,
                                            },
                                            todo!(),
                                            todo!(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::ChangeDirection(d) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        if let Some(o) = map.get_object_mut(*re) {
                                            let mut loc = o.get_location();
                                            loc.direction = d;
                                            o.set_location(loc);
                                            // TODO notify all visible parties of change in direction
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::Chat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let amsg =
                                            format!("[{}] {}", map.get_name(*re).unwrap(), msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        for (id, o) in map.objects_iter() {
                                            if o.linear_distance(&chatter_location) < 30.0 {
                                                if let Some(se) =
                                                    self.object_senders.get(&id.get_u32())
                                                {
                                                    se.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::RegularChat {
                                                            id: 0,
                                                            msg: amsg.clone(),
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::YellChat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let name = map.get_name(*re).unwrap();
                                        let amsg = format!("[{}] {}", name, msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        for (id, o) in map.objects_iter() {
                                            if o.linear_distance(&chatter_location) < 60.0 {
                                                if let Some(se) =
                                                    self.object_senders.get(&id.get_u32())
                                                {
                                                    se.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::YellChat {
                                                            id: 0,
                                                            msg: amsg.clone(),
                                                            x: chatter_location.x,
                                                            y: chatter_location.y,
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::PartyChat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let name = map.get_name(*re).unwrap();
                                        let amsg = format!("[{}] {}", name, msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        for (id, o) in map.objects_iter() {
                                            if let Some(se) = self.object_senders.get(&id.get_u32())
                                            {
                                                se.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::PartyChat(amsg.clone()),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::PledgeChat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let name = map.get_name(*re).unwrap();
                                        let amsg = format!("[{}] {}", name, msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        for (id, o) in map.objects_iter() {
                                            if let Some(se) = self.object_senders.get(&id.get_u32())
                                            {
                                                se.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::PledgeChat(amsg.clone()),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::WhisperChat(n, msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let sender_name = map.get_name(*re).unwrap();
                                        let amsg = format!("[{}] {}", sender_name, msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        let mut found_player = false;
                                        for (id, o) in map.objects_iter() {
                                            if let Some(receiver_name) = o.player_name() {
                                                if receiver_name == n {
                                                    if let Some(se) =
                                                        self.object_senders.get(&id.get_u32())
                                                    {
                                                        se.blocking_send(
                                                            WorldResponse::ServerPacket(
                                                                ServerPacket::WhisperChat {
                                                                    name: sender_name,
                                                                    msg: msg.clone(),
                                                                },
                                                            ),
                                                        );
                                                        found_player = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if !found_player {
                                            if let Some(sender) = m.sender {
                                                if let Some(se) = self.object_senders.get(&sender) {
                                                    se.blocking_send(WorldResponse::ServerPacket(
                                                        ServerPacket::Message {
                                                            ty: 73,
                                                            msgs: vec![n],
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::GlobalChat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(r) = self.characters.get(&sender) {
                                if let Some(re) = self.object_ref_table.get(r) {
                                    if let Some(map) = self.map_info.get_mut(&re.map) {
                                        let name = map.get_name(*re).unwrap();
                                        let amsg = format!("[{}] {}", name, msg);
                                        let chatter_location = map.get_location(*re).unwrap();
                                        for (id, o) in map.objects_iter() {
                                            if let Some(se) = self.object_senders.get(&id.get_u32())
                                            {
                                                se.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::GlobalChat(amsg.clone()),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::CommandChat(msg) => {
                        if let Some(sender) = m.sender {
                            if let Some(s) = self.object_senders.get_mut(&sender) {
                                let mut words = msg.split_whitespace();
                                let first_word = words.next();
                                if let Some(mw) = first_word {
                                    /// TODO replace with a hashmap converting strings to functions
                                    match mw {
                                        "asdf" => {
                                            log::info!("A command called asdf");
                                        }
                                        "shutdown" => {
                                            log::info!("A shutdown command was received");
                                            if let Some(r) = self.characters.get(&sender) {
                                                if let Some(re) = self.object_ref_table.get(r) {
                                                    self.shutdown(re);
                                                }
                                            }
                                        }
                                        "restart" => {
                                            log::info!("A restart command was received");
                                            if let Some(r) = self.characters.get(&sender) {
                                                if let Some(re) = self.object_ref_table.get(r) {
                                                    self.restart(re);
                                                }
                                            }
                                        }
                                        "quit" => {
                                            if let Some(s) = self.object_senders.get_mut(&sender) {
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::Disconnect,
                                                ));
                                            }
                                        }
                                        "test" => {
                                            log::info!("Test requested");
                                        }
                                        "chat" => {
                                            if let Some(s) = self.object_senders.get_mut(&sender) {
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::SystemMessage(
                                                        "This is a test of the system message"
                                                            .to_string(),
                                                    ),
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::NpcShout(
                                                        "NPC Shout test".to_string(),
                                                    ),
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::RegularChat {
                                                        id: 0,
                                                        msg: "regular chat".to_string(),
                                                    },
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::YellChat {
                                                        id: 0,
                                                        msg: "yelling".to_string(),
                                                        x: 32768,
                                                        y: 32768,
                                                    },
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::GlobalChat(
                                                        "global chat".to_string(),
                                                    ),
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::PledgeChat(
                                                        "pledge chat".to_string(),
                                                    ),
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::PartyChat(
                                                        "party chat".to_string(),
                                                    ),
                                                ));
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::WhisperChat {
                                                        name: "test".to_string(),
                                                        msg: "whisper message".to_string(),
                                                    },
                                                ));
                                            }
                                        }
                                        _ => {
                                            log::info!("An unknown command {}", mw);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::SpecialCommandChat(m) => {
                        log::info!("special command chat {}", m);
                    }
                    ClientPacket::ChangePassword {
                        account,
                        oldpass,
                        newpass: _,
                    } => {
                        if let Ok(mut mysql) = self.get_mysql_conn() {
                            if let Some(sender) = m.sender {
                                if let Some(s) = self.object_senders.get_mut(&sender) {
                                    let user =
                                        crate::user::get_user_details(account.clone(), &mut mysql);
                                    match user {
                                        Some(us) => {
                                            let password_success = us.check_login(
                                                &self.config.account_creation_salt,
                                                &oldpass,
                                            );
                                            if password_success {
                                                log::info!(
                                                    "User wants to change password and entered correct details"
                                                );
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::LoginResult { code: 0x30 },
                                                ));
                                            } else {
                                                s.blocking_send(WorldResponse::ServerPacket(
                                                    ServerPacket::LoginResult { code: 8 },
                                                ));
                                            }
                                        }
                                        _ => {
                                            s.blocking_send(WorldResponse::ServerPacket(
                                                ServerPacket::LoginResult { code: 8 },
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientPacket::Unknown(d) => {
                        log::info!("received unknown packet {:x?}", d);
                    }
                },
            }
        }
        log::error!("Exiting world run instance");
    }

    /// Get a new object id
    pub fn new_object_id(&mut self) -> WorldObjectId {
        self.id_generator.new_id()
    }

    /// Get a location of an object reference
    pub fn get_location(&self, r: ObjectRef) -> Option<Location> {
        let map = self.map_info.get(&r.map);
        if let Some(map) = map {
            return map.get_location(r);
        }
        None
    }

    /// Shutdown the server if the player is authorized to do so
    pub fn shutdown(&self, r: &ObjectRef) {
        let shutdown = {
            let map = self.map_info.get(&r.map);
            if let Some(map) = map {
                if let Some(obj) = map.get_object_from_id(r.id) {
                    obj.can_shutdown()
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
                .blocking_send(crate::server_message::ServerShutdownMessage::Shutdown);
        }
    }

    /// Restart the server if the player is authorized to do so
    pub fn restart(&self, r: &ObjectRef) {
        let shutdown = {
            let map = self.map_info.get(&r.map);
            if let Some(map) = map {
                if let Some(obj) = map.get_object_from_id(r.id) {
                    obj.can_shutdown()
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
                .blocking_send(crate::server_message::ServerShutdownMessage::Restart);
        }
    }

    /// Spawn all monsters
    pub fn spawn_monsters(&mut self) {
        if let Some(mset) = &mut self.monster_set {
            let mut monsters = Vec::new();
            let mut idgen = &mut self.id_generator;
            for ms in &self.monster_spawn_table {
                let m = ms.make_monster(idgen.new_id(), &self.npc_table);
                monsters.push(m);
            }
            {
                for m in &monsters {
                    let mut monref = m.reference();
                    mset.spawn(async move { monref.run_ai().await });
                }
            }
            {
                log::info!("There are {} monsters to spawn", monsters.len());
                for m in monsters {
                    if let Some(map) = self.map_info.get_mut(&m.get_location().map) {
                        map.add_new_object(m.into());
                    }
                }
            }
        }
    }

    /// Send a new object packet with the given packet writer and object id
    pub fn send_new_object(
        &self,
        location: crate::character::Location,
        id: WorldObjectId,
        pw: &mut common::packet::ServerPacketSender,
    ) -> Result<(), ClientError> {
        let map = self.map_info.get(&location.map);
        if let Some(map) = map {
            if let Some(obj) = map.get_object_from_id(id) {
                let p = obj.build_put_object_packet();
                pw.queue_packet(p);
            }
        }
        Ok(())
    }

    /// Add a player to the world
    pub fn add_player(
        &mut self,
        p: crate::character::FullCharacter,
        s: &mut tokio::sync::mpsc::Sender<WorldResponse>,
    ) -> Option<ObjectRef> {
        let location = p.location_ref().to_owned();

        s.blocking_send(WorldResponse::ServerPacket(p.details_packet()));
        s.blocking_send(WorldResponse::ServerPacket(p.get_map_packet()));
        s.blocking_send(WorldResponse::ServerPacket(p.get_object_packet()));
        p.send_all_items(s).ok()?;
        s.blocking_send(WorldResponse::ServerPacket(ServerPacket::CharSpMrBonus {
            sp: 0,
            mr: 0,
        }));
        s.blocking_send(WorldResponse::ServerPacket(ServerPacket::Weather(0)));

        let obj: object::Object = p.into();
        let id = obj.id();

        let m2 = self.map_info.get_mut(&location.map);
        log::error!("add player 1");
        if let Some(map) = m2 {
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
                .get_known_objects()
                .unwrap()
                .get_objects()
            {
                log::error!("Player knows about object {:?}", o);
            }
            //TODO map.move_object(or, location, Some(pw), list).ok()?;
            log::error!("add player 5");
            Some(or)
        } else {
            log::error!("add player 6");
            None
        }
    }

    /// Remove a player from the world
    pub fn remove_player(&mut self, r: &mut Option<ObjectRef>) {
        if let Some(r) = &r {
            let map = self.map_info.get_mut(&r.map);
            if let Some(map) = map {
                map.remove_object(r.id);
            }
        }
        *r = None;
    }

    /// (Re)load the weapons table
    pub fn load_weapons(
        mysql: &mut mysql::PooledConn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &mut WorldIdGenerator,
    ) -> Result<(), String> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from weapon";
        let s = mysql.prep(query).map_err(|e| e.to_string())?;
        let weapons = mysql
            .exec_map(s, (), |a: item::Weapon| a)
            .map_err(|e| e.to_string())?;
        for w in weapons {
            let w = w.get_instance(next_object_id.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the etc items table
    pub fn load_etc_items(
        mysql: &mut mysql::PooledConn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &mut WorldIdGenerator,
    ) -> Result<(), String> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from etcitem";
        let s = mysql.prep(query).map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::EtcItem| a)
            .map_err(|e| e.to_string())?;
        for w in items {
            let w = w.get_instance(next_object_id.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load the armor table
    pub fn load_armor(
        mysql: &mut mysql::PooledConn,
        item_table: &mut HashMap<u32, item::Item>,
        next_object_id: &mut WorldIdGenerator,
    ) -> Result<(), String> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from armor";
        let s = mysql.prep(query).map_err(|e| e.to_string())?;
        let items = mysql
            .exec_map(s, (), |a: item::Armor| a)
            .map_err(|e| e.to_string())?;
        for w in items {
            let w = w.get_instance(next_object_id.new_id());
            item_table.insert(w.db_id(), w.into());
        }
        Ok(())
    }

    /// (Re)load all item data from database
    pub fn load_item_data(
        mysql: &mut mysql::PooledConn,
        next_object_id: &mut WorldIdGenerator,
    ) -> Result<HashMap<u32, item::Item>, String> {
        let mut item_table = HashMap::new();
        log::info!("There are {} items", item_table.len());
        Self::load_weapons(mysql, &mut item_table, next_object_id)?;
        log::info!("There are {} items", item_table.len());
        Self::load_etc_items(mysql, &mut item_table, next_object_id)?;
        log::info!("There are {} items", item_table.len());
        Self::load_armor(mysql, &mut item_table, next_object_id)?;
        log::info!("There are {} items", item_table.len());
        Ok(item_table)
    }

    /// (Re)load all maps from the database
    pub fn load_maps_data(
        mysql: &mut mysql::PooledConn,
    ) -> Result<(HashMap<u16, Map>, HashMap<u16, map_info::MapInfo>), String> {
        let mut hmaps = HashMap::new();
        use mysql::prelude::Queryable;
        let query = "SELECT mapid, locationname, startX, endX, startY, endY, monster_amount, drop_rate, underwater, markable, teleportable, escapable, resurrection, painwand, penalty, take_pets, recall_pets, usable_item, usable_skill from mapids";
        let s = mysql.prep(query).map_err(|e| e.to_string())?;
        let maps = mysql
            .exec_map(s, (), |a: Map| a)
            .map_err(|e| e.to_string())?;
        let mut hdata = HashMap::new();
        for m in maps {
            hdata
                .entry(m.id)
                .or_insert_with(|| map_info::MapInfo::new());
            hmaps.insert(m.id, m);
        }
        Ok((hmaps, hdata))
    }

    /// Insert a character id into the world
    pub fn insert_id(&mut self, id: u32, account: String) -> Result<(), ClientError> {
        self.users.insert(id, account);
        Ok(())
    }

    /// lookup account name from user id
    pub fn lookup_id(&self, id: u32) -> Option<String> {
        self.users.get(&id).map(|e| e.to_owned())
    }

    /// Get a new object id as part of a transaction.
    /// This prevents atomicity problems where two threads can get the same new id, and try to insert the same id into the database.
    /// # Arguments:
    /// * t - The transaction object
    pub fn get_new_id(t: &mut mysql::Transaction<'_>) -> Result<Option<u32>, mysql::Error> {
        use mysql::prelude::Queryable;
        let query = "select max(id)+1 as nextid from (select id from character_items union all select id from character_teleport union all select id from character_warehouse union all select id from character_elf_warehouse union all select objid as id from characters union all select clan_id as id from clan_data union all select id from clan_warehouse union all select objid as id from pets) t";
        let a: Vec<Option<u32>> = t.exec(query, ())?;
        let r = if let Some(a) = a.first() {
            Ok(*a)
        } else {
            Ok(None)
        };
        r
    }

    /// Get a connection to the database
    pub fn get_mysql_conn(&self) -> Result<mysql::PooledConn, mysql::Error> {
        self.mysql.get_conn()
    }

    /// Register a new user
    pub fn register_user(&mut self) -> u32 {
        self.client_ids.new_entry()
    }

    /// Save a new character into the database
    pub fn save_new_character(&self, c: &mut Character) -> Result<(), ClientError> {
        let mut conn = self.get_mysql_conn()?;
        c.save_new_to_db(&mut conn)
    }

    /// Unregister a user
    pub fn unregister_user(&mut self, uid: u32) {
        self.client_ids.remove_entry(uid);
        self.users.remove(&uid);
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        self.users.len() as u16
    }
}
