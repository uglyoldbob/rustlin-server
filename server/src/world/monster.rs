//! Monster related code for the world

use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, SocketAddrV4},
};

/// Defines how a monster location is determined when spawned
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum SpawnType {
    /// Normal, possibly randomized spawn
    Normal = 0,
    /// The monster should spawn near a random player on the map it spawns on
    NearPlayer = 1,
}

impl std::convert::TryFrom<u8> for SpawnType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::NearPlayer),
            _ => Err("Invalid monster spawn type".to_string()),
        }
    }
}

/// Defines how to spawn a monster
#[derive(Clone, Debug)]
pub struct MonsterSpawn {
    /// The monster id from the database
    id: u32,
    /// The number of monsters to spawn
    count: u16,
    /// The npc id to refer to when spawning the monster
    npc_definition: u32,
    /// Where to spawn the monster
    location: Location,
    /// Used for randomizing spawn location x coordinate
    randomx: u16,
    /// Used for randomizing spawn location y coordinate
    randomy: u16,
    /// Used for randomizing spawn location x coordinate
    coord1: (u16, u16),
    /// Used for randomizing spawn location y coordinate
    coord2: (u16, u16),
    /// The minimum and maximum delay time for respawn
    respawn_delay: (u32, u32),
    /// The type of spawn for the monster
    spawn_type: SpawnType,
}

impl mysql::prelude::FromRow for MonsterSpawn {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        let locx: u16 = row.get(5).ok_or(mysql::FromRowError(row.clone()))?;
        let locy: u16 = row.get(6).ok_or(mysql::FromRowError(row.clone()))?;
        let map: u16 = row.get(16).ok_or(mysql::FromRowError(row.clone()))?;
        let direction: u8 = row.get(13).ok_or(mysql::FromRowError(row.clone()))?;
        let st: u8 = row.get(20).ok_or(mysql::FromRowError(row.clone()))?;
        Ok(Self {
            id: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            count: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            npc_definition: row.get(3).ok_or(mysql::FromRowError(row.clone()))?,
            location: Location {
                x: locx,
                y: locy,
                map,
                direction,
            },
            randomx: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            randomy: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            coord1: (
                row.get(9).ok_or(mysql::FromRowError(row.clone()))?,
                row.get(10).ok_or(mysql::FromRowError(row.clone()))?,
            ),
            coord2: (
                row.get(11).ok_or(mysql::FromRowError(row.clone()))?,
                row.get(12).ok_or(mysql::FromRowError(row.clone()))?,
            ),
            respawn_delay: (
                row.get(14).ok_or(mysql::FromRowError(row.clone()))?,
                row.get(15).ok_or(mysql::FromRowError(row.clone()))?,
            ),
            spawn_type: std::convert::TryInto::try_into(st)
                .map_err(|_| mysql::FromRowError(row.clone()))?,
        })
    }
}

impl MonsterSpawn {
    /// Load the npc spawn table from the database
    pub fn load_table(mysql: &mut mysql::PooledConn) -> Result<Vec<Self>, super::ClientError> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from spawnlist";
        let s = mysql.exec_map(query, (), |a: Self| a)?;
        Ok(s)
    }

    /// Get the map that this spawn spawns monsters on
    pub fn map(&self) -> u16 {
        self.location.map
    }

    /// Create an npc object
    pub fn make_monster(
        &self,
        id: super::WorldObjectId,
        npcs: &HashMap<u32, super::npc::NpcDefinition>,
    ) -> Monster {
        let npc = npcs.get(&self.npc_definition).unwrap();
        let mut location;
        //a loop to make sure that the random position is on the map in a valid fashion
        let mut attempts_to_randomize = 0;
        const ATTEMPTS_MAX: usize = 50;
        let mut rng = rand::thread_rng();
        use rand::Rng;
        loop {
            location = self.location;
            /// TODO npc 45488 (Caspa randomly on Gludio floor 3 or 4)
            /// TODO npc 45601 (Death Knight randomly on Gludio floor 5,6,7)
            if self.spawn_type == SpawnType::NearPlayer {
                // TODO spawn near a random player on the map it should spawn on?
            }

            if self.coord1.0 != self.coord1.1 && self.coord2.0 != self.coord2.1 {
                //area spawning
                let range = if self.coord1.0 < self.coord1.1 {
                    self.coord1.0..self.coord1.1
                } else {
                    self.coord1.1..self.coord1.0
                };
                let x = rng.gen_range(range);
                let range = if self.coord2.0 < self.coord2.1 {
                    self.coord2.0..self.coord2.1
                } else {
                    self.coord2.1..self.coord2.0
                };
                let y = rng.gen_range(range);
                location.x = x;
                location.y = y;
            } else if self.randomx != 0 && self.randomy != 0 {
                location.x += rng.gen_range(0..self.randomx);
                location.x -= rng.gen_range(0..self.randomx);
                location.y += rng.gen_range(0..self.randomy);
                location.y -= rng.gen_range(0..self.randomy);
            }
            attempts_to_randomize += 1;
            //TODO Actually check to see if the random coordinate is valid on the map
            if true {
                break;
            }
        }

        if attempts_to_randomize == ATTEMPTS_MAX {
            location = self.location;
        }

        let chan = tokio::sync::mpsc::channel(1000);

        Monster {
            id,
            location,
            alignment: npc.alignment,
            icon: npc.graphics_id,
            name: npc.name.clone(),
            light_size: npc.light_size,
            spawn: self.clone(),
            send: chan.0,
            recv: Some(chan.1),
        }
    }
}

/// The holder of a reference to a monster
pub struct MonsterRef {
    /// A reference to the monster on the world
    reference: ObjectRef,
    ///Monster location
    location: Location,
    /// monster id
    id: Option<u32>,
    /// Sender of worldResponse
    send: tokio::sync::mpsc::Sender<WorldResponse>,
    /// The receiver from the world
    recv: tokio::sync::mpsc::Receiver<WorldResponse>,
    /// Objects the monster knows about
    objects: HashMap<WorldObjectId, Location>,
    /// Objects the monster will attack on sight
    attacks: HashSet<WorldObjectId>,
}

impl Drop for MonsterRef {
    fn drop(&mut self) {}
}

impl MonsterRef {
    ///move the monster randomly
    pub async fn moving(&mut self, sender: &mut tokio::sync::mpsc::Sender<super::WorldMessage>) {
        use rand::Rng;
        let direction = rand::thread_rng().gen_range(0..=7u8);
        let (x, y) = (self.location.x, self.location.y);
        let (x2, y2) = match direction {
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
        let new_loc = Location {
            x: x2,
            y: y2,
            map: self.location.map,
            direction,
        };
        sender
            .send(WorldMessage {
                data: crate::world::WorldMessageData::ClientPacket(
                    common::packet::ClientPacket::MoveFrom {
                        x,
                        y,
                        heading: new_loc.direction,
                    },
                ),
                sender: self.id,
                peer: std::net::SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    1234,
                )),
            })
            .await;
        self.location = new_loc;
        let random_wait = rand::thread_rng().gen_range(500..=1000u16);
        tokio::time::sleep(std::time::Duration::from_millis(random_wait as u64)).await;
    }

    /// Find a target to attack, return true if it did something
    pub async fn find_target(
        &mut self,
        sender: &mut tokio::sync::mpsc::Sender<super::WorldMessage>,
    ) -> bool {
        let mut found_target = None;
        for pt in self.attacks.iter() {
            if self.objects.contains_key(pt) {
                found_target = Some(pt);
                break;
            }
        }
        if let Some(target) = found_target {
            log::info!("Im going to attack {}", target.get_u32());
            use rand::Rng;
            let random_wait = rand::thread_rng().gen_range(1000..=2000u16);
            tokio::time::sleep(std::time::Duration::from_millis(random_wait as u64)).await;
        }
        found_target.is_some()
    }

    /// Run the ai for the monster
    pub async fn run_ai(
        mut self,
        mut sender: tokio::sync::mpsc::Sender<super::WorldMessage>,
        m: Monster,
    ) {
        let mut m = Some(m);
        let _ = sender
            .send(WorldMessage {
                data: crate::world::WorldMessageData::RegisterSender(self.send.clone()),
                sender: self.id,
                peer: std::net::SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::new(127, 0, 0, 1),
                    1234,
                )),
            })
            .await;
        use rand::Rng;
        let random_wait = rand::thread_rng().gen_range(0..=1000u16);
        tokio::time::sleep(std::time::Duration::from_millis(random_wait as u64)).await;
        loop {
            while let Ok(msg) = self.recv.try_recv() {
                match msg {
                    super::WorldResponse::ServerPacket(p) => match p {
                        common::packet::ServerPacket::Attack {
                            attack_type,
                            id,
                            id2,
                            impact,
                            direction,
                            effect,
                        } => {
                            if self.reference.id.get_u32() == id2 {
                                self.attacks.insert(WorldObjectId(id));
                                let _ = sender
                                    .send(WorldMessage {
                                        data: crate::world::WorldMessageData::ClientPacket(
                                            common::packet::ClientPacket::NpcChat {
                                                id: self.reference.id.get_u32(),
                                                message: "I'm being attacked".to_string(),
                                            },
                                        ),
                                        sender: self.id,
                                        peer: std::net::SocketAddr::V4(SocketAddrV4::new(
                                            Ipv4Addr::new(127, 0, 0, 1),
                                            1234,
                                        )),
                                    })
                                    .await;
                            }
                        }
                        common::packet::ServerPacket::MoveObject {
                            id,
                            x,
                            y,
                            direction,
                        } => {
                            self.objects.insert(
                                WorldObjectId(id),
                                Location {
                                    x,
                                    y,
                                    map: self.location.map,
                                    direction,
                                },
                            );
                        }
                        common::packet::ServerPacket::PutObject {
                            x,
                            y,
                            id,
                            icon,
                            status,
                            direction,
                            light,
                            speed,
                            xp,
                            alignment,
                            name,
                            title,
                            status2,
                            pledgeid,
                            pledgename,
                            owner_name,
                            v1,
                            hp_bar,
                            v2,
                            level,
                        } => {
                            self.objects.insert(
                                WorldObjectId(id),
                                Location {
                                    x,
                                    y,
                                    map: self.location.map,
                                    direction,
                                },
                            );
                        }
                        common::packet::ServerPacket::RemoveObject(id) => {
                            self.objects.remove(&WorldObjectId(id));
                        }
                        _ => {
                            log::error!("Unhandled packet for monster {:?}: {:?}", self.id, p);
                        }
                    },
                    super::WorldResponse::NewClientId(id) => {
                        if let Some(m) = m.take() {
                            self.id = Some(id);
                            let _ = sender
                                .send(WorldMessage {
                                    data: crate::world::WorldMessageData::RegisterMonster(m),
                                    sender: self.id,
                                    peer: std::net::SocketAddr::V4(SocketAddrV4::new(
                                        Ipv4Addr::new(127, 0, 0, 1),
                                        1234,
                                    )),
                                })
                                .await;
                        }
                    }
                }
            }
            if !self.find_target(&mut sender).await {
                self.moving(&mut sender).await;
            }
        }
        log::info!("Exiting monster ai");
    }
}

use crate::{
    character::Location,
    world::{ObjectRef, WorldMessage, WorldObjectId, WorldResponse},
};

/// A monster on the world
#[derive(Debug)]
pub struct Monster {
    /// The object id for the npc
    id: super::WorldObjectId,
    /// Where the npc currently exists
    location: crate::character::Location,
    /// The npc name
    name: String,
    /// the npc alignment
    alignment: i16,
    /// The size of light emitted by the npc
    light_size: u8,
    /// the icon for displaying the npc
    icon: u16,
    /// The spawner
    spawn: MonsterSpawn,
    /// Sender of worldResponse
    send: tokio::sync::mpsc::Sender<WorldResponse>,
    /// The temporary receiver from the world
    recv: Option<tokio::sync::mpsc::Receiver<WorldResponse>>,
}

impl Monster {
    /// Get a reference to the monster
    pub fn reference(&mut self) -> MonsterRef {
        MonsterRef {
            reference: ObjectRef {
                map: self.location.map,
                id: self.id,
            },
            location: self.location,
            id: None,
            send: self.send.clone(),
            recv: self.recv.take().unwrap(),
            objects: HashMap::new(),
            attacks: HashSet::new(),
        }
    }
}

impl super::ObjectTrait for Monster {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn other_hit_rate_bonus(&self) -> i16 {
        0
    }

    fn critical_hit_miss_values(&self) -> (i16, i16) {
        (0, 19)
    }

    fn str_attack_hit_bonus(&self) -> i8 {
        0
    }

    fn dex_attack_hit_bonus(&self) -> i8 {
        0
    }

    fn weapon(&self) -> Option<&crate::world::item::WeaponInstance> {
        None
    }

    fn hit_rate_bonus(&self) -> i16 {
        0
    }

    fn ranged_hit_rate_bonus(&self) -> i16 {
        0
    }

    fn base_attack_rate(&self) -> i16 {
        1
    }

    fn armor_class(&self) -> i8 {
        0
    }

    fn max_weight(&self) -> u32 {
        1
    }

    fn attack_type(&self) -> super::object::BasicObjectType {
        super::object::BasicObjectType::Monster
    }

    fn set_location(&mut self, l: crate::character::Location) {
        self.location = l;
    }

    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<crate::world::WorldResponse>> {
        Some(self.send.clone())
    }

    fn object_name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> super::WorldObjectId {
        self.id
    }

    fn build_put_object_packet(&self) -> common::packet::ServerPacket {
        common::packet::ServerPacket::PutObject {
            x: self.location.x,
            y: self.location.y,
            id: self.id.get_u32(),
            icon: self.icon,
            status: 0,
            direction: self.location.direction,
            light: self.light_size,
            speed: 50,
            xp: 1235,
            alignment: self.alignment,
            name: self.name.clone(),
            title: "".to_string(),
            status2: 0,
            pledgeid: 0,
            pledgename: "".to_string(),
            owner_name: "".to_string(),
            v1: 0,
            hp_bar: 255,
            v2: 0,
            level: 54,
        }
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn player_name(&self) -> Option<String> {
        None
    }
}
