//! Code for items that npcs and players can use and have

use std::collections::HashMap;

/// The trait for every item in the game
#[enum_dispatch::enum_dispatch]
pub trait ItemTrait {
    /// Retrieve the item id
    fn id(&self) -> u32;
    /// Get the inventory packet
    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement;
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

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending inventory packet for {:?}", self);
        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description: " $1".to_string(),
            ed: Vec::new(),
        }
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

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending armor inventory packet for {:?}", self);
        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description: " $1".to_string(),
            ed: Vec::new(),
        }
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

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending etc inventory element for {:?}", self);
        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description: " $1".to_string(),
            ed: Vec::new(),
        }
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

#[derive(Clone, Debug)]
pub enum ItemOrId {
    Item(Item),
    Id(u32),
}

/// The stuff for an item instance
#[derive(Clone, Debug)]
pub struct ItemStuff {
    /// The item id
    item_id: u32,
    /// Item count
    count: u32,
    /// Equipped
    equipped: bool,
    /// Enchantment
    enchanted_level: i8,
    /// Is the item identified?
    identified: bool,
    /// Durability
    durability: u8,
    /// Is the item blessed
    blessed: u8,
    /// Number of charges
    charges: u8,
    /// Remaining time
    time_ramaining: u32,
}

/// an instance of an item, as opposed to Item, which defines what an item is
#[derive(Clone, Debug)]
pub struct ItemInstance {
    /// The item definition
    definition: ItemOrId,
    stuff: ItemStuff,
}

impl ItemInstance {
    /// Populate the item definition, if necessary
    pub fn populate_item_definition(&mut self, item_table: &HashMap<u32, Item>) {
        if let ItemOrId::Id(id) = self.definition {
            if let Some(item) = item_table.get(&id) {
                self.definition = ItemOrId::Item(item.to_owned());
            } else {
                log::error!("Item {} not found in table", id);
            }
        }
    }

    /// Get the inventory packet for this item instance, retrieving the item details if needed
    pub fn inventory_element(&mut self, item_table: &HashMap<u32, Item>) -> Option<common::packet::InventoryElement> {
        self.populate_item_definition(item_table);
        if let ItemOrId::Item(i) = &self.definition {
            Some(i.inventory_element(&self.stuff))
        } else {
            None
        }
    }
}

impl mysql_async::prelude::FromRow for ItemStuff {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
        where
            Self: Sized {
        Ok(Self {
            item_id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            count: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            equipped: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            enchanted_level: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            durability: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            blessed: row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
            charges: row.get(13).ok_or(mysql_async::FromRowError(row.clone()))?,
            time_ramaining: row.get(14).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl mysql_async::prelude::FromRow for ItemInstance {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let id : u32 = row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            definition: ItemOrId::Id(id),
            stuff: mysql_async::prelude::FromRow::from_row_opt(row)?,
        })
    }
}
