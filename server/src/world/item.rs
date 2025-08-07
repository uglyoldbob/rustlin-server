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

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ElementalEnchantType {
    Earth = 1,
    Fire = 2,
    Water = 4,
    Wind = 8,
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

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending inventory packet for {:?}", self);

        let mut description = " ".to_string();
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
        if stuff.identified {
            if self.max_use_time > 0 {
                description.push_str(&format!("({}) ", self.max_use_time));
            }
        }

        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }

        if stuff.equipped {
            description.push_str("($9)");
        }

        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
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

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending armor inventory packet for {:?}", self);

        let mut description = " ".to_string();

        if stuff.identified {
            if stuff.enchanted_level > 0 {
                description.push_str(&format!("+{} ", stuff.enchanted_level));
            }
        }
        description.push_str(if stuff.identified {
            &self.identified
        } else {
            &self.unidentified
        });
        if stuff.identified {
            if self.max_use_time > 0 {
                description.push_str(&format!("({}) ", self.max_use_time));
            }
        }
        if stuff.count > 1 {
            description.push_str(&format!("({}) ", stuff.count));
        }

        if stuff.equipped {
            description.push_str("($17)");
        }

        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum EtcItemType {
    Arrow = 0,
    Wand,
    Light,
    Gem,
    Totem,
    Firecracker,
    Potion,
    Food,
    Scroll,
    QuestItem,
    SpellBook,
    PetItem,
    Other,
    Material,
    EventItem,
    Sting,
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
}

impl EtcItem {
    const HORSE_RIDING_HELMET: u32 = 20383;
    const LAMP: u32 = 40001;
    const LANTERN: u32 = 40002;
    const PET_COLLAR_ID: u32 = 40314;
    const HIGH_QUALITY_PET_COLLAR: u32 = 40316;
}

impl mysql_async::prelude::FromRow for EtcItem {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let itype: String = row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            id: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            weight: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            inventory_graphic: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            ground_graphic: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            max_charge_count: row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
            unidentified: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            identified: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            itype: itype.as_str().into(),
        })
    }
}

impl ItemTrait for EtcItem {
    fn id(&self) -> u32 {
        self.id
    }

    fn inventory_element(&self, stuff: &ItemStuff) -> common::packet::InventoryElement {
        log::info!("Sending etc inventory element for {:?}", self);
        let mut description = " ".to_string();

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

        if stuff.equipped {
            if self.itype == EtcItemType::PetItem {
                description.push_str("($117)");
            }
        }

        common::packet::InventoryElement {
            id: self.id,
            i_type: 1,
            n_use: 1,
            icon: self.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
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
    time_remaining: u32,
    /// Elemental enchant type and level
    /// Items 41429, 41430, 41431, and 41432 perform the enchanting on items
    elemental_enchant: Option<(ElementalEnchantType, u8)>,
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
    pub fn inventory_element(
        &mut self,
        item_table: &HashMap<u32, Item>,
    ) -> Option<common::packet::InventoryElement> {
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

impl mysql_async::prelude::FromRow for ItemInstance {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        let id: u32 = row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            definition: ItemOrId::Id(id),
            stuff: mysql_async::prelude::FromRow::from_row_opt(row)?,
        })
    }
}
