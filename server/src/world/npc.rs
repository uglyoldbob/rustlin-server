//! NPC related code

use std::collections::HashMap;

use crate::character::Location;

#[derive(Debug)]
pub struct NpcDefinition {
    id: u32,
    name: String,
    graphics_id: u32,
    light_size: u8,
}

impl NpcDefinition {
    /// Load the npc definition table from the database
    pub async fn load_table(mysql: &mut mysql_async::Conn) -> Result<HashMap<u32, Self>, String> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from npc";
        let s = mysql
            .exec_map(query, (), |a: Self| a)
            .await
            .map_err(|e| format!("{:?}", e))?;
        let mut t = HashMap::new();
        for s in s {
            t.insert(s.id, s);
        }
        Ok(t)
    }
}

impl mysql_async::prelude::FromRow for NpcDefinition {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            name: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            graphics_id: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            light_size: row.get(60).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

#[derive(Debug)]
pub struct NpcSpawn {
    id: u32,
    count: u8,
    npc_definition: u32,
    location: Location,
    randomx: u16,
    randomy: u16,
    respawn_delay: u32,
    distance: u32,
}

impl mysql_async::prelude::FromRow for NpcSpawn {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let locx: u16 = row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?;
        let locy: u16 = row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?;
        let map: u16 = row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?;
        let direction: u8 = row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?;
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
            randomx: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            randomy: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            respawn_delay: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            distance: row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl NpcSpawn {
    /// Load the npc spawn table from the database
    pub async fn load_table(
        mysql: &mut mysql_async::Conn,
    ) -> Result<Vec<Self>, super::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT * from spawnlist_npc";
        let s = mysql.exec_map(query, (), |a: Self| a).await?;
        Ok(s)
    }

    /// Create an npc object
    pub fn make_npc(&self) -> Npc {
        Npc {
            id: 42,
            location: self.location,
        }
    }
}

#[derive(Debug)]
pub struct Npc {
    /// The object id for the npc
    id: u32,
    /// Where the npc currently exists
    location: crate::character::Location,
}

impl Npc {
    /// Build a new Npc, this is a temporary function for testing
    pub fn new(id: u32, location: crate::character::Location) -> Self {
        Self { id, location }
    }
}

impl super::object::ObjectTrait for Npc {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn player_name(&self) -> Option<String> {
        None
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn sender(
        &mut self,
    ) -> Option<&mut tokio::sync::mpsc::Sender<crate::server_message::ServerMessage>> {
        None
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
        log::info!("Put object packet for an npc");
        common::packet::ServerPacket::PutObject {
            x: self.location.x,
            y: self.location.y,
            id: self.id,
            icon: 29,
            status: 0,
            direction: self.location.direction,
            light: 7,
            speed: 50,
            xp: 1235,
            alignment: -2767,
            name: "steve".to_string(),
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
}
