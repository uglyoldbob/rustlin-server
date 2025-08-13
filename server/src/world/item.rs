//! Code for items that npcs and players can use and have

use std::collections::HashMap;

/// The trait for every item in the game
#[enum_dispatch::enum_dispatch]
pub trait ItemTrait {
    /// Retrieve the item id
    fn id(&self) -> u32;
    /// Get the inventory packet
    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement;
    /// Get the packet for updating the item
    fn update_packet(&self, stuff: &ItemStuff) -> common::packet::InventoryUpdate;
    /// Get the item name
    fn name(&self, stuff: &ItemStuff) -> String;
    /// Get the item usage
    fn usage(&self) -> ItemUsage;
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
    /// Unidentified name
    unidentified: String,
    /// Identified name
    identified: String,
    /// Maximum use time
    max_use_time: u32,
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
            unidentified: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_use_time: row.get(44).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}

impl ItemTrait for Weapon {
    fn id(&self) -> u32 {
        self.id
    }

    fn usage(&self) -> ItemUsage {
        ItemUsage::Weapon
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
        if stuff.identified {
            if let Some((t, l)) = &stuff.elemental_enchant {
                match t {
                    ElementalEnchantType::Earth => match l {
                        1 => description.push_str("$6124 "),
                        2 => description.push_str("$6125 "),
                        3 => description.push_str("$6126 "),
                        _ => {}
                    },
                    ElementalEnchantType::Fire => match l {
                        1 => description.push_str("$6115 "),
                        2 => description.push_str("$6116 "),
                        3 => description.push_str("$6117 "),
                        _ => {}
                    },
                    ElementalEnchantType::Water => match l {
                        1 => description.push_str("$6118 "),
                        2 => description.push_str("$6119 "),
                        3 => description.push_str("$6120 "),
                        _ => {}
                    },
                    ElementalEnchantType::Wind => match l {
                        1 => description.push_str("$6121 "),
                        2 => description.push_str("$6122 "),
                        3 => description.push_str("$6123 "),
                        _ => {}
                    },
                }
            }
            if stuff.enchanted_level > 0 {
                description.push_str(&format!("+{} ", stuff.enchanted_level));
            }
        }
        description.push_str(if stuff.identified {
            &self.identified
        } else {
            &self.unidentified
        });
        if stuff.identified && self.max_use_time > 0 {
            description.push_str(&format!("({}) ", self.max_use_time));
        }

        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }

        if stuff.equipped {
            description.push_str(" ($9)");
        }

        description
    }

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending inventory packet for {:?}", self);

        let mut description = " ".to_string();
        description.push_str(&self.name(stuff));

        common::packet::InventoryElement {
            id: stuff.item_id,
            i_type: 1,
            n_use: ItemUsage::Weapon as u8,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
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
    /// Unidentified name
    unidentified: String,
    /// Identified name
    identified: String,
    /// Maximum use time
    max_use_time: u32,
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

impl ItemTrait for Armor {
    fn id(&self) -> u32 {
        self.id
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
            &self.identified
        } else {
            &self.unidentified
        });
        if stuff.identified && self.max_use_time > 0 {
            description.push_str(&format!("({}) ", self.max_use_time));
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
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}

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

impl ItemTrait for EtcItem {
    fn id(&self) -> u32 {
        self.id
    }

    fn usage(&self) -> ItemUsage {
        self.usage
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
            &self.identified
        } else {
            &self.unidentified
        });
        if self.max_charge_count > 0 {
            description.push_str(&format!("({}) ", stuff.charges));
        }
        if self.id == Self::HORSE_RIDING_HELMET {
            description.push_str(&format!("({}) ", stuff.charges));
        }
        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }
        if self.id == Self::PET_COLLAR_ID || self.id == Self::HIGH_QUALITY_PET_COLLAR {
            description.push_str("[PET TODO]");
        }

        if EtcItemType::Light == self.itype {
            ///TODO
            let emitting_light = false;
            if emitting_light {
                description.push_str("($10) ");
            }
            match self.id {
                Self::LAMP | Self::LANTERN => {
                    if stuff.time_remaining == 0 {
                        description.push_str("($11) )");
                    }
                }
                _ => {}
            }
        }

        if stuff.equipped && self.itype == EtcItemType::PetItem {
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
            i_type: self.itype as i8,
            n_use: self.usage as u8,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}

/// A generic item
#[enum_dispatch::enum_dispatch(ItemTrait)]
#[derive(Clone, Debug)]
pub enum Item {
    /// A player usable weapon
    Weapon(Weapon),
    /// Some other kind of item
    Etc(EtcItem),
    /// A piece of armor
    Armor(Armor),
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
    pub fn update_description_packet(&self) -> common::packet::Packet {
        let a = common::packet::ServerPacket::InventoryDescriptionUpdate {
            id: self.stuff.item_id,
            description: self.name(),
        };
        log::info!("Description packet: {:?}", a);
        a.build()
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

impl mysql_async::prelude::FromRow for ItemStuff {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let elemental_enchant_type: Option<u8> =
            row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?;
        let elemental_enchant_level: Option<u8> =
            row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?;
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
            item_id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            count: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            equipped: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            enchanted_level: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            durability: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            blessed: row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
            charges: row.get(13).ok_or(mysql_async::FromRowError(row.clone()))?,
            time_remaining: row.get(14).ok_or(mysql_async::FromRowError(row.clone()))?,
            elemental_enchant: e,
        })
    }
}

impl mysql_async::prelude::FromRow for ItemInstanceWithoutDefinition {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            stuff: mysql_async::prelude::FromRow::from_row_opt(row)?,
        })
    }
}
