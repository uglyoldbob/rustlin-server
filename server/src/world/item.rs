//! Code for items that npcs and players can use and have

use std::collections::HashMap;

/// The trait for every item in the game
#[enum_dispatch::enum_dispatch]
pub trait ItemTrait {
    /// Retrieve the item id
    fn id(&self) -> u32;
    /// Get the inventory packet
    fn inventory_packet(&self) -> common::packet::Packet;
}

/// A player usable weapon
#[derive(Clone, Debug)]
pub struct Weapon {
    /// Item definition id
    id: u32,
    /// item weight
    weight: u32,
    /// Graphics for inventory
    inventory_graphic: i16,
    /// Graphics for ground
    ground_graphic: u16,
}

impl mysql_async::prelude::FromRow for Weapon {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            weight: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            inventory_graphic: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            ground_graphic: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl ItemTrait for Weapon {
    fn id(&self) -> u32 {
        self.id
    }

    fn inventory_packet(&self) -> common::packet::Packet {
        log::info!("Sending inventory packet for {:?}", self);
        common::packet::ServerPacket::Inventory {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: 1,
            identified: 0,
            description: " $1".to_string(),
            ed: Vec::new(),
        }
        .build()
    }
}

/// A player usable piece of armor
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
        })
    }
}

impl ItemTrait for Armor {
    fn id(&self) -> u32 {
        self.id
    }

    fn inventory_packet(&self) -> common::packet::Packet {
        log::info!("Sending armor inventory packet for {:?}", self);
        common::packet::ServerPacket::Inventory {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: 1,
            identified: 0,
            description: " $1".to_string(),
            ed: Vec::new(),
        }
        .build()
    }
}

/// Other items that a player can have
#[derive(Clone, Debug)]
pub struct EtcItem {
    /// Item definition id
    id: u32,
    /// item weight
    weight: u32,
    /// Graphics for inventory
    inventory_graphic: i16,
    /// Graphics for ground
    ground_graphic: u16,
}

impl mysql_async::prelude::FromRow for EtcItem {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            weight: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            inventory_graphic: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            ground_graphic: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl ItemTrait for EtcItem {
    fn id(&self) -> u32 {
        self.id
    }

    fn inventory_packet(&self) -> common::packet::Packet {
        log::info!("Sending etc inventory packet for {:?}", self);
        common::packet::ServerPacket::Inventory {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: 1,
            identified: 0,
            description: " $1".to_string(),
            ed: Vec::new(),
        }
        .build()
    }
}

// A generic item
#[enum_dispatch::enum_dispatch(ItemTrait)]
#[derive(Clone, Debug)]
pub enum Item {
    Weapon(Weapon),
    Etc(EtcItem),
    Armor(Armor),
}

/// an instance of an item, as opposed to Item, which defines what an item is
#[derive(Clone, Debug)]
pub struct ItemInstance {
    /// The item instance id
    id: u32,
    /// The item id
    item_id: u32,
}

impl ItemInstance {
    /// Get the item instance
    pub fn item<'a>(&'_ self, item_table: &'a HashMap<u32, Item>) -> Option<&'a Item> {
        item_table.get(&self.item_id)
    }

    /// Create a new item instance
    pub fn new(id: u32) -> Self {
        ///TODO actually get a new item id using a mysql transaction
        Self {
            id: 1000,
            item_id: id,
        }
    }
}

impl mysql_async::prelude::FromRow for ItemInstance {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            item_id: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}
