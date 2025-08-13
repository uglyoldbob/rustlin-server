//! Monster related code for the world

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum SpawnType {
    Normal = 0,
    WhenPlayerNearby = 1,
}

impl std::convert::TryFrom<u8> for SpawnType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::WhenPlayerNearby),
            _ => Err("Invalid monster spawn type".to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonsterSpawn {
    id: u32,
    count: u16,
    npc_definition: u32,
    location: Location,
    randomx: u16,
    randomy: u16,
    coord1: (u16, u16),
    coord2: (u16, u16),
    respawn_delay: (u32, u32),
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
    ) -> Option<Monster> {
        let npc = npcs.get(&self.npc_definition).unwrap();
        if self.spawn_type == SpawnType::WhenPlayerNearby {
            return None;
        }
        Some(Monster {
            id,
            location: self.location,
            alignment: npc.alignment,
            icon: npc.graphics_id,
            name: npc.name.clone(),
            light_size: npc.light_size,
            spawn: self.clone(),
        })
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
        log::info!("Running with {:?}", self.reference);
        let mut index = 0;
        loop {
            if index == 5 { break; }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            log::info!("Monster {:?} is at {}", self.reference, index);
            index += 1;
        }
        log::info!("Exiting monster {:?}", self.reference);
    }
}

use crate::{character::Location, server_message::ServerMessage, world::ObjectRef};
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

    fn set_location(&mut self, l: crate::character::Location) {
        self.location = l;
    }

    fn id(&self) -> super::WorldObjectId {
        self.id
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
        common::packet::ServerPacket::PutObject {
            x: self.location.x,
            y: self.location.y,
            id: self.id.into(),
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
        .build()
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn sender(&mut self) -> Option<&mut tokio::sync::mpsc::Sender<ServerMessage>> {
        None
    }

    fn player_name(&self) -> Option<String> {
        None
    }
}
