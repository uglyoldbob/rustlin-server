//! NPC related code

use std::collections::HashMap;

use crate::character::Location;

/// A definition for an npc
#[derive(Debug)]
pub struct NpcDefinition {
    /// The definition id of the npc from the database
    pub id: u32,
    /// The name of the npc
    pub name: String,
    /// Thra graphics to display the npc with
    pub graphics_id: u16,
    /// How much light the npc emits
    pub light_size: u8,
    /// The alignment of the npc
    pub alignment: i16,
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
            alignment: row.get(17).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

/// Defines how to spawn an npc
#[derive(Debug)]
pub struct NpcSpawn {
    /// The id of the spawn in the database
    id: u32,
    /// The number of npc to spawn
    count: u8,
    /// The definition to look at when spawning the npc
    npc_definition: u32,
    /// Where to spawn the npc
    location: Location,
    /// How to randomize the spawn x coordinate
    randomx: u16,
    /// How to randomize the spawn y coordinate
    randomy: u16,
    /// The amount of time to wait when respawning
    respawn_delay: u32,
    /// The maximum travel distance
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
    pub fn make_npc(&self, id: super::WorldObjectId, npcs: &HashMap<u32, NpcDefinition>) -> Npc {
        let npc = npcs.get(&self.npc_definition).unwrap();
        Npc {
            id,
            location: self.location,
            old_location: None,
            alignment: npc.alignment,
            icon: npc.graphics_id,
            name: npc.name.clone(),
            light_size: npc.light_size,
        }
    }
}

/// An npc in the server (not a monster)
#[derive(Debug)]
pub struct Npc {
    /// The object id for the npc
    id: super::WorldObjectId,
    /// Where the npc currently exists
    location: crate::character::Location,
    /// The last place the npc was
    old_location: Option<crate::character::Location>,
    /// The npc name
    name: String,
    /// the npc alignment
    alignment: i16,
    /// The size of light emitted by the npc
    light_size: u8,
    /// the icon for displaying the npc
    icon: u16,
}

impl super::object::ObjectTrait for Npc {
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

    fn player_name(&self) -> Option<String> {
        None
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn sender(&mut self) -> Option<&mut tokio::sync::mpsc::Sender<common::packet::ServerPacket>> {
        None
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
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
        .build()
    }
}
