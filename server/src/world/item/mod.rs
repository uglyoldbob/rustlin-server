//! Code for items that npcs and players can use and have

mod armor;
pub use armor::*;

mod etc;
pub use etc::*;

mod weapon;
pub use weapon::*;

use std::collections::HashMap;

use crate::world::{object::ObjectTrait, WorldObjectId};

/// The trait for every item in the game
#[enum_dispatch::enum_dispatch]
pub trait ItemTrait {
    /// Retrieve the item id
    fn db_id(&self) -> u32;
    /// Get the world id
    fn world_id(&self) -> WorldObjectId;
    /// Get the inventory packet
    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement;
    /// Get the packet for updating the item
    fn update_packet(&self, stuff: &ItemStuff) -> common::packet::InventoryUpdate;
    /// Get the item name
    fn name(&self, stuff: &ItemStuff) -> String;
    /// Get the item usage
    fn usage(&self) -> ItemUsage;
    /// Get the ground icon for this particular item
    fn ground_icon(&self) -> u16;
}

/// The elemental types that an item can be enchanted with
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ElementalEnchantType {
    /// Earth elemental
    Earth = 1,
    /// Fire elemental
    Fire = 2,
    /// Water elemental
    Water = 4,
    /// Wind elemental
    Wind = 8,
}

/// The ways an item can be used
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i8)]
pub enum ItemUsage {
    None = -1,
    Normal = 0,
    Weapon = 1,
    Armor = 2,
    Wand = 3,
    WishWand = 4,
    WandWithTarget = 5,
    TeleportScroll0 = 9,
    TeleportScroll1 = 6,
    IdentifyScroll = 7,
    ResurrectScroll = 8,
    Letter = 12,
    Letter2 = 13,
    WithTarget = 14,
    MusicalInstrument = 15,
    Polymorph = 16,
    WandNearbyTarget = 17,
    HeroRing = 23,
    EnchantWeaponScroll = 26,
    EnchantArmorScroll = 27,
    BlankScroll = 28,
    BlessedTeleport = 29,
    SpellBuff = 30,
    ChristmasCard = 31,
    ChristmasCard2 = 32,
    ValentinesCard = 33,
    ValentinesCard2 = 34,
    WhiteDayCard = 35,
    WhiteDayCard2 = 36,
    Earring = 40,
    FishingRod = 42,
    EnchantAccessoryScroll = 46,
}

impl From<i8> for ItemUsage {
    fn from(value: i8) -> Self {
        match value {
            -1 => Self::None,
            0 => Self::Normal,
            1 => Self::Weapon,
            2 => Self::Armor,
            3 => Self::Wand,
            4 | 38 => Self::WishWand,
            5 => Self::WandWithTarget,
            9 => Self::TeleportScroll0,
            6 => Self::TeleportScroll1,
            7 => Self::IdentifyScroll,
            8 => Self::ResurrectScroll,
            12 => Self::Letter,
            13 => Self::Letter2,
            14 => Self::WithTarget,
            15 => Self::MusicalInstrument,
            16 => Self::Polymorph,
            17 => Self::WandNearbyTarget,
            23 => Self::HeroRing,
            26 => Self::EnchantWeaponScroll,
            27 => Self::EnchantArmorScroll,
            28 => Self::BlankScroll,
            29 => Self::BlessedTeleport,
            30 => Self::SpellBuff,
            31 => Self::ChristmasCard,
            32 => Self::ChristmasCard2,
            33 => Self::ValentinesCard,
            34 => Self::ValentinesCard2,
            35 => Self::WhiteDayCard,
            36 => Self::WhiteDayCard2,
            40 => Self::Earring,
            42 => Self::FishingRod,
            46 => Self::EnchantAccessoryScroll,
            _ => unimplemented!(),
        }
    }
}

impl From<&str> for ItemUsage {
    fn from(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "normal" => Self::Normal,
            "weapon" => Self::Weapon,
            "armor" => Self::Armor,
            "spell_long" => Self::WandWithTarget,
            "ntele" => Self::TeleportScroll1,
            "identify" => Self::IdentifyScroll,
            "res" => Self::ResurrectScroll,
            "letter" => Self::Letter,
            "letter_w" => Self::Letter2,
            "choice" => Self::WithTarget,
            "instrument" => Self::MusicalInstrument,
            "sosc" => Self::Polymorph,
            "spell_short" => Self::WandNearbyTarget,
            "zel" => Self::EnchantArmorScroll,
            "dai" => Self::EnchantWeaponScroll,
            "blank" => Self::BlankScroll,
            "ccard" => Self::ChristmasCard,
            "ccard_w" => Self::ChristmasCard2,
            "vcard" => Self::ValentinesCard,
            "vcard_w" => Self::ValentinesCard2,
            "spell_buff" => Self::SpellBuff,
            "earring" => Self::Earring,
            "ring" => Self::HeroRing,
            "btele" => Self::BlessedTeleport,
            "fishing_rod" => Self::FishingRod,
            "del" => Self::EnchantAccessoryScroll,
            _ => Self::None,
        }
    }
}

/// A generic item
#[enum_dispatch::enum_dispatch(ItemTrait)]
#[derive(Clone, Debug)]
pub enum Item {
    /// A player usable weapon
    Weapon(WeaponInstance),
    /// Some other kind of item
    Etc(EtcItemInstance),
    /// A piece of armor
    Armor(ArmorInstance),
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
    time_remaining: u32,
    /// Elemental enchant type and level
    /// Items 41429, 41430, 41431, and 41432 perform the enchanting on items
    elemental_enchant: Option<(ElementalEnchantType, u8)>,
}

/// an instance of an item, as opposed to Item, which defines what an item is
#[derive(Clone, Debug)]
pub struct ItemInstanceWithoutDefinition {
    /// The item id
    id: u32,
    /// The item configuration details
    stuff: ItemStuff,
}

/// an instance of an item, as opposed to Item, which defines what an item is
#[derive(Clone, Debug)]
pub struct ItemInstance {
    /// The item definition
    definition: Item,
    /// The item definition id
    id: u32,
    /// The item configuration details
    stuff: ItemStuff,
}

impl ItemInstance {
    /// Get the item id
    pub fn id(&self) -> u32 {
        self.stuff.item_id
    }

    /// Toggle if the item is equipped or not
    pub fn toggle_equip(&mut self) {
        self.stuff.equipped = !self.stuff.equipped;
    }

    /// Get the item definition
    pub fn definition(&self) -> &Item {
        &self.definition
    }

    /// Get the item name
    pub fn name(&self) -> String {
        self.definition.name(&self.stuff)
    }

    /// Get item usage
    pub fn usage(&self) -> ItemUsage {
        self.definition.usage()
    }

    /// Get the inventory packet for this item instance, retrieving the item details if needed
    pub fn inventory_element(&self) -> common::packet::InventoryElement {
        self.definition.inventory_element(&self.stuff)
    }

    /// Get the inventory modification data
    pub fn update_packet(&self) -> common::packet::InventoryUpdate {
        let i = self.definition.update_packet(&self.stuff);
        log::info!("Item update details {:?}", i);
        i
    }

    /// Get the inventory description update packet
    pub fn update_description_packet(&self) -> common::packet::ServerPacket {
        let a = common::packet::ServerPacket::InventoryDescriptionUpdate {
            id: self.stuff.item_id,
            description: self.name(),
        };
        log::info!("Description packet: {:?}", a);
        a
    }
}

impl ItemInstanceWithoutDefinition {
    /// Populate the item definition, if necessary
    pub fn populate_item_definition(self, item_table: &HashMap<u32, Item>) -> Option<ItemInstance> {
        if let Some(item) = item_table.get(&self.id) {
            Some(ItemInstance {
                definition: item.to_owned(),
                id: self.id,
                stuff: self.stuff,
            })
        } else {
            log::error!("Item {} not found in table", self.id);
            None
        }
    }

    /// Get the item id
    pub fn id(&self) -> u32 {
        self.stuff.item_id
    }
}

impl mysql::prelude::FromRow for ItemStuff {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        let elemental_enchant_type: Option<u8> =
            row.get(11).ok_or(mysql::FromRowError(row.clone()))?;
        let elemental_enchant_level: Option<u8> =
            row.get(12).ok_or(mysql::FromRowError(row.clone()))?;
        let e = if let Some(elemental_enchant_type) = elemental_enchant_type {
            if let Some(elemental_enchant_level) = elemental_enchant_level {
                match elemental_enchant_type {
                    1 => Some((ElementalEnchantType::Earth, elemental_enchant_level)),
                    2 => Some((ElementalEnchantType::Fire, elemental_enchant_level)),
                    4 => Some((ElementalEnchantType::Water, elemental_enchant_level)),
                    8 => Some((ElementalEnchantType::Wind, elemental_enchant_level)),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };
        Ok(Self {
            item_id: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            count: row.get(4).ok_or(mysql::FromRowError(row.clone()))?,
            equipped: row.get(5).ok_or(mysql::FromRowError(row.clone()))?,
            enchanted_level: row.get(6).ok_or(mysql::FromRowError(row.clone()))?,
            identified: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            durability: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            blessed: row.get(10).ok_or(mysql::FromRowError(row.clone()))?,
            charges: row.get(13).ok_or(mysql::FromRowError(row.clone()))?,
            time_remaining: row.get(14).ok_or(mysql::FromRowError(row.clone()))?,
            elemental_enchant: e,
        })
    }
}

impl mysql::prelude::FromRow for ItemInstanceWithoutDefinition {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(1).ok_or(mysql::FromRowError(row.clone()))?,
            stuff: mysql::prelude::FromRow::from_row_opt(row)?,
        })
    }
}

/// An item that exists on the ground somewhere
#[derive(Debug)]
pub struct ItemWithLocation {
    /// The map location of the item
    location: crate::character::Location,
    /// The item on that spot of the map
    item: Item,
}

impl ObjectTrait for ItemWithLocation {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn set_location(&mut self, l: crate::character::Location) {
        self.location = l;
    }

    fn id(&self) -> super::WorldObjectId {
        ItemTrait::world_id(&self.item)
    }

    fn build_put_object_packet(&self) -> common::packet::ServerPacket {
        common::packet::ServerPacket::PutObject {
            x: self.location.x,
            y: self.location.y,
            id: ItemTrait::world_id(&self.item).get_u32(),
            icon: self.item.ground_icon(),
            status: 0,
            direction: 0,
            light: 1,
            speed: 1,
            xp: 0,
            alignment: 0,
            name: "TODO".to_string(),
            title: String::new(),
            status2: 0,
            pledgeid: 0,
            pledgename: String::new(),
            owner_name: String::new(),
            v1: 0,
            hp_bar: 255,
            v2: 0,
            level: 1,
        }
    }
}
