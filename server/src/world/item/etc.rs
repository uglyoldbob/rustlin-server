//! Code for other miscellaneous item types

use super::{ItemStuff, ItemTrait, ItemUsage};
use crate::world::WorldObjectId;

/// The item type for etc items
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum EtcItemType {
    /// An arrow to use in a bow
    Arrow = 0,
    /// A wand of some sort
    Wand,
    /// The item can be used to create light
    Light,
    /// a gem of some variety
    Gem,
    /// A totem
    Totem,
    /// A firework that a player can use
    Firecracker,
    /// Potions a player can use
    Potion,
    /// Items that can be eaten
    Food,
    /// scrolls that players can use
    Scroll,
    /// ITems used for quests
    QuestItem,
    /// Items for learning spells/skills
    SpellBook,
    /// Items for pets
    PetItem,
    /// Other item usage
    Other,
    /// Items used for crafting somehow
    Material,
    /// A special event item
    EventItem,
    /// Unsure, includes throwing knives
    Sting,
    /// An item that gives items
    TreasureBox,
}

impl From<&str> for EtcItemType {
    fn from(value: &str) -> Self {
        match value {
            "arrow" => Self::Arrow,
            "wand" => Self::Wand,
            "light" => Self::Light,
            "gem" => Self::Gem,
            "totem" => Self::Totem,
            "firecracker" => Self::Firecracker,
            "potion" => Self::Potion,
            "food" => Self::Food,
            "scroll" => Self::Scroll,
            "questitem" => Self::QuestItem,
            "spellbook" => Self::SpellBook,
            "petitem" => Self::PetItem,
            "other" => Self::Other,
            "material" => Self::Material,
            "event" => Self::EventItem,
            "sting" => Self::Sting,
            "treasure_box" => Self::TreasureBox,
            _ => panic!("Invalid etc item type {}", value),
        }
    }
}

impl From<u8> for EtcItemType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Arrow,
            1 => Self::Wand,
            2 => Self::Light,
            3 => Self::Gem,
            4 => Self::Totem,
            5 => Self::Firecracker,
            6 => Self::Potion,
            7 => Self::Food,
            8 => Self::Scroll,
            9 => Self::QuestItem,
            10 => Self::SpellBook,
            11 => Self::PetItem,
            12 => Self::Other,
            13 => Self::Material,
            14 => Self::EventItem,
            15 => Self::Sting,
            16 => Self::TreasureBox,
            _ => panic!("Invalid etc item type {}", value),
        }
    }
}

/// Other items that a player can have
#[derive(Clone, Debug)]
pub struct EtcItem {
    /// Item definition id
    id: u32,
    /// World object id
    world_id: Option<WorldObjectId>,
    /// The etc item type
    itype: EtcItemType,
    /// item weight
    weight: u32,
    /// Graphics for inventory
    inventory_graphic: i16,
    /// Graphics for ground
    ground_graphic: u16,
    /// Maxinum number of charges
    max_charge_count: u8,
    /// Unidentified name
    unidentified: String,
    /// Identified name
    identified: String,
    /// Item usage
    usage: ItemUsage,
}

impl EtcItem {
    /// Item id for a horse riding helmet
    const HORSE_RIDING_HELMET: u32 = 20383;
    /// Item id for a lamp
    const LAMP: u32 = 40001;
    /// Item id for a lantern
    const LANTERN: u32 = 40002;
    /// Item id for a regular pet collar
    const PET_COLLAR_ID: u32 = 40314;
    /// Item id for a high quality pet collar
    const HIGH_QUALITY_PET_COLLAR: u32 = 40316;

    /// Make a weapon instance
    pub fn get_instance(&self, world_id: WorldObjectId) -> EtcItemInstance {
        EtcItemInstance {
            world_id,
            definition: self.clone(),
        }
    }
}

impl mysql_async::prelude::FromRow for EtcItem {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let itype: String = row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?;
        let usage: String = row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            world_id: None,
            weight: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            inventory_graphic: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            ground_graphic: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_charge_count: row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
            unidentified: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            itype: itype.as_str().into(),
            usage: usage.as_str().into(),
        })
    }
}

/// A miscellanous item
#[derive(Clone, Debug)]
pub struct EtcItemInstance {
    /// Item definition
    definition: EtcItem,
    /// World object id
    world_id: WorldObjectId,
}

impl ItemTrait for EtcItemInstance {
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
        self.definition.usage
    }

    fn update_packet(&self, stuff: &ItemStuff) -> common::packet::InventoryUpdate {
        common::packet::InventoryUpdate {
            id: stuff.item_id,
            description: self.name(stuff),
            count: stuff.count,
            ed: Vec::new(),
        }
    }

    fn name(&self, stuff: &ItemStuff) -> String {
        let mut description = String::new();

        description.push_str(if stuff.identified {
            &self.definition.identified
        } else {
            &self.definition.unidentified
        });
        if self.definition.max_charge_count > 0 {
            description.push_str(&format!("({}) ", stuff.charges));
        }
        if self.definition.id == EtcItem::HORSE_RIDING_HELMET {
            description.push_str(&format!("({}) ", stuff.charges));
        }
        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }
        if self.definition.id == EtcItem::PET_COLLAR_ID
            || self.definition.id == EtcItem::HIGH_QUALITY_PET_COLLAR
        {
            description.push_str("[PET TODO]");
        }

        if EtcItemType::Light == self.definition.itype {
            ///TODO
            let emitting_light = false;
            if emitting_light {
                description.push_str("($10) ");
            }
            match self.definition.id {
                EtcItem::LAMP | EtcItem::LANTERN => {
                    if stuff.time_remaining == 0 {
                        description.push_str("($11) )");
                    }
                }
                _ => {}
            }
        }

        if stuff.equipped && self.definition.itype == EtcItemType::PetItem {
            description.push_str(" ($117)");
        }

        description
    }

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending etc inventory element for {:?}", self);
        let mut description = " ".to_string();
        description.push_str(&self.name(stuff));

        common::packet::InventoryElement {
            id: stuff.item_id,
            i_type: self.definition.itype as i8,
            n_use: self.definition.usage as u8,
            icon: self.definition.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}
