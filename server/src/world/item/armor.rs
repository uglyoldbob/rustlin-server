//! Code for character armor

use super::{ItemStuff, ItemTrait, ItemUsage};
use crate::world::WorldObjectId;

/// A definition for a piece of armor
#[derive(Clone, Debug)]
pub struct Armor {
    /// Item definition id
    id: u32,
    /// item weight
    weight: u32,
    /// Graphics for inventory
    inventory_graphic: i16,
    /// Graphics for ground
    ground_graphic: u16,
    /// Unidentified name
    unidentified: String,
    /// Identified name
    identified: String,
    /// Maximum use time
    max_use_time: u32,
}

impl Armor {
    /// Make an armor instance
    pub fn get_instance(&self, world_id: WorldObjectId) -> ArmorInstance {
        ArmorInstance {
            world_id,
            definition: self.clone(),
        }
    }
}

impl mysql_async::prelude::FromRow for Armor {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            weight: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            inventory_graphic: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            ground_graphic: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            unidentified: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_use_time: row.get(43).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

/// A player usable piece of armor
#[derive(Clone, Debug)]
pub struct ArmorInstance {
    /// Item definition
    definition: Armor,
    /// World object id
    world_id: WorldObjectId,
}

impl ItemTrait for ArmorInstance {
    fn world_id(&self) -> WorldObjectId {
        self.world_id
    }

    fn db_id(&self) -> u32 {
        self.definition.id
    }

    fn ground_icon(&self) -> u16 {
        self.definition.ground_graphic
    }

    fn usage(&self) -> ItemUsage {
        ItemUsage::Armor
    }

    fn update_packet(&self, stuff: &ItemStuff) -> common::packet::InventoryUpdate {
        log::info!("Item: {:?}, {:?}", self, stuff);
        common::packet::InventoryUpdate {
            id: stuff.item_id,
            description: self.name(stuff),
            count: stuff.count,
            ed: Vec::new(),
        }
    }

    fn name(&self, stuff: &ItemStuff) -> String {
        let mut description = String::new();

        if stuff.identified && stuff.enchanted_level > 0 {
            description.push_str(&format!("+{} ", stuff.enchanted_level));
        }
        description.push_str(if stuff.identified {
            &self.definition.identified
        } else {
            &self.definition.unidentified
        });
        if stuff.identified && self.definition.max_use_time > 0 {
            description.push_str(&format!("({}) ", self.definition.max_use_time));
        }
        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }

        if stuff.equipped {
            description.push_str(" ($117)");
        }

        description
    }

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending armor inventory packet for {:?}", self);

        let mut description = " ".to_string();
        description.push_str(&self.name(stuff));

        common::packet::InventoryElement {
            id: stuff.item_id,
            i_type: 2,
            n_use: ItemUsage::Armor as u8,
            icon: self.definition.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}
