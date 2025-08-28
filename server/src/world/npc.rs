//! NPC related code

use std::collections::{HashMap, HashSet};

use crate::character::Location;

/// A definition for an npc
#[derive(Clone, Debug)]
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
    /// The max hp
    pub max_hp: u16,
    /// The max mp
    pub max_mp: u16,
}

impl NpcDefinition {
    /// Load the npc definition table from the database
    pub fn load_table(mysql: &mut mysql::PooledConn) -> Result<HashMap<u32, Self>, String> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from npc";
        let s = mysql
            .exec_map(query, (), |a: Self| a)
            .map_err(|e| format!("{:?}", e))?;
        let mut t = HashMap::new();
        for s in s {
            t.insert(s.id, s);
        }
        Ok(t)
    }
}

impl mysql::prelude::FromRow for NpcDefinition {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            name: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            graphics_id: row.get(5).ok_or(mysql::FromRowError(row.clone()))?,
            light_size: row.get(60).ok_or(mysql::FromRowError(row.clone()))?,
            alignment: row.get(17).ok_or(mysql::FromRowError(row.clone()))?,
            max_hp: row.get("hp").ok_or(mysql::FromRowError(row.clone()))?,
            max_mp: row.get("mp").ok_or(mysql::FromRowError(row.clone()))?,
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

impl mysql::prelude::FromRow for NpcSpawn {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        let locx: u16 = row.get(4).ok_or(mysql::FromRowError(row.clone()))?;
        let locy: u16 = row.get(5).ok_or(mysql::FromRowError(row.clone()))?;
        let map: u16 = row.get(10).ok_or(mysql::FromRowError(row.clone()))?;
        let direction: u8 = row.get(8).ok_or(mysql::FromRowError(row.clone()))?;
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
            randomx: row.get(6).ok_or(mysql::FromRowError(row.clone()))?,
            randomy: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            respawn_delay: row.get(9).ok_or(mysql::FromRowError(row.clone()))?,
            distance: row.get(11).ok_or(mysql::FromRowError(row.clone()))?,
        })
    }
}

impl NpcSpawn {
    /// Load the npc spawn table from the database
    pub fn load_table(mysql: &mut mysql::PooledConn) -> Result<Vec<Self>, super::ClientError> {
        use mysql::prelude::Queryable;
        let query = "SELECT * from spawnlist_npc";
        let s = mysql.exec_map(query, (), |a: Self| a)?;
        Ok(s)
    }

    /// Create an npc object
    pub fn make_npc(&self, id: super::WorldObjectId, npcs: &HashMap<u32, NpcDefinition>) -> Npc {
        let npc = npcs.get(&self.npc_definition).unwrap();
        Npc {
            id,
            location: self.location,
            alignment: npc.alignment,
            icon: npc.graphics_id,
            name: npc.name.clone(),
            light_size: npc.light_size,
            effects: HashSet::new(),
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
    /// The npc name
    name: String,
    /// the npc alignment
    alignment: i16,
    /// The size of light emitted by the npc
    light_size: u8,
    /// the icon for displaying the npc
    icon: u16,
    /// The list of current effects
    effects: HashSet<crate::world::object::Effect>,
}

impl super::object::ObjectTrait for Npc {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn apply_damage(&mut self, dmg: u16) {}

    fn compute_max_attack_damage(
        &self,
        weapon: Option<&crate::world::item::WeaponInstance>,
    ) -> (u16, u16) {
        (0, 0)
    }

    fn get_polymorph(&self) -> Option<u32> {
        None
    }

    fn compute_received_damage(&self, d: (u16, u16)) -> u16 {
        ///TODO
        d.0
    }

    fn apply_required_polymorph(&self, poly: Option<u32>, rate: &mut u8) {}

    fn apply_required_status(
        &self,
        _effects: &HashSet<crate::world::object::Effect>,
        _rate: &mut u8,
    ) {
    }

    fn dex_attack_dmg_bonus(&self) -> i8 {
        0
    }

    fn str_attack_dmg_bonus(&self) -> i8 {
        0
    }

    fn use_weapon_ammunition(&mut self) -> bool {
        true
    }

    fn get_evasive_rating(&self) -> u8 {
        0
    }

    fn get_effects(&self) -> &HashSet<crate::world::object::Effect> {
        &self.effects
    }

    fn effects_mut(&mut self) -> &mut HashSet<super::object::Effect> {
        &mut self.effects
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

    fn max_weight(&self) -> u32 {
        1
    }

    fn armor_class(&self) -> i8 {
        0
    }

    fn attack_type(&self) -> super::object::BasicObjectType {
        super::object::BasicObjectType::Npc
    }

    fn set_location(&mut self, l: crate::character::Location) {
        self.location = l;
    }

    fn id(&self) -> super::WorldObjectId {
        self.id
    }

    fn player_name(&self) -> Option<String> {
        None
    }

    fn object_name(&self) -> String {
        self.name.clone()
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
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
}
