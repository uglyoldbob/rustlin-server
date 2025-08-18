//! Monster related code for the world

use std::collections::HashMap;

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

impl mysql_async::prelude::FromRow for MonsterSpawn {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let locx: u16 = row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?;
        let locy: u16 = row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?;
        let map: u16 = row.get(16).ok_or(mysql_async::FromRowError(row.clone()))?;
        let direction: u8 = row.get(13).ok_or(mysql_async::FromRowError(row.clone()))?;
        let st: u8 = row.get(20).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            count: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            npc_definition: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            location: Location {
                x: locx,
                y: locy,
                map,
                direction,
            },
            randomx: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            randomy: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            coord1: (
                row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
                row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
            ),
            coord2: (
                row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?,
                row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
            ),
            respawn_delay: (
                row.get(14).ok_or(mysql_async::FromRowError(row.clone()))?,
                row.get(15).ok_or(mysql_async::FromRowError(row.clone()))?,
            ),
            spawn_type: std::convert::TryInto::try_into(st)
                .map_err(|_| mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl MonsterSpawn {
    /// Load the npc spawn table from the database
    pub async fn load_table(
        mysql: &mut mysql_async::Conn,
    ) -> Result<Vec<Self>, super::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from spawnlist";
        let s = mysql.exec_map(query, (), |a: Self| a).await?;
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

        Monster {
            id,
            location,
            alignment: npc.alignment,
            icon: npc.graphics_id,
            name: npc.name.clone(),
            light_size: npc.light_size,
            spawn: self.clone(),
            old_location: None,
        }
    }
}

/// The holder of a reference to a monster
pub struct MonsterRef {
    /// A reference to the monster on the world
    reference: ObjectRef,
    /// The world object
    world: std::sync::Arc<crate::world::World>,
}

impl MonsterRef {
    /// Run the ai for the monster
    pub async fn run_ai(&mut self) {
        use rand::Rng;
        let initial_delay = rand::thread_rng().gen_range(0..=100000000);
        tokio::time::sleep(std::time::Duration::from_micros(initial_delay)).await;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let direction = rand::thread_rng().gen_range(0..=7u8);
            let myloc = self.world.get_location(self.reference);
            if let Some(l) = myloc {
                let (x, y) = (l.x, l.y);
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
                    map: l.map,
                    direction,
                };
                if self.reference.id.get_u32() == 6431 {
                    log::info!("Moving the bear to {:?}", new_loc);
                }
                let mut list = crate::world::map_info::SendsToAnotherObject::new();
                let _ = self.world.move_object(self.reference, new_loc, None, &mut list).await;
                if self.reference.id.get_u32() == 6431 {
                    log::info!("Done moving the bear to {:?}", new_loc);
                }
            }
        }
    }
}

use crate::{character::Location, world::ObjectRef};

/// A monster on the world
#[derive(Debug)]
pub struct Monster {
    /// The object id for the npc
    id: super::WorldObjectId,
    /// Where the npc currently exists
    location: crate::character::Location,
    /// The last place the object was
    old_location: Option<crate::character::Location>,
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
}

impl Monster {
    /// Get a reference to the monster
    pub fn reference(&self, world: std::sync::Arc<crate::world::World>) -> MonsterRef {
        MonsterRef {
            reference: ObjectRef {
                map: self.location.map,
                id: self.id,
            },
            world,
        }
    }
}

impl super::ObjectTrait for Monster {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn get_prev_location(&self) -> crate::character::Location {
        self.old_location.unwrap_or(self.location)
    }

    fn set_location(&mut self, l: crate::character::Location) {
        self.old_location = Some(self.location);
        self.location = l;
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
