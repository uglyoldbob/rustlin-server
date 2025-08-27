//! Code for weapons

use super::super::{ItemTrait, WorldObjectId};
use super::{ElementalEnchantType, ItemStuff, ItemUsage};

/// The types of weapons
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum WeaponType {
    /// sword
    Sword = 1,
    /// dagger
    Dagger = 2,
    /// Two handed sword
    TwoHandSword = 3,
    /// bow
    Bow = 4,
    /// spear
    Spear = 5,
    /// a blunt weapon
    Blunt = 6,
    /// A staff
    Staff = 7,
    /// gauntlet
    Gauntlet = 10,
    /// A claw
    Claw = 11,
    /// Edoryu
    Edoryu = 12,
    /// Single bow
    SingleBow = 13,
    /// Signle spear
    SingleSpear = 14,
    /// Two handed blunt weapon
    TwoHandBlunt = 15,
    /// Two handed staff
    TwoHandStaff = 16,
    /// Kiringku?
    Kiringku = 17,
    /// chain sword
    ChainSword = 18,
}

impl WeaponType {
    /// Get the weapon id
    pub fn get_id(&self) -> u16 {
        match self {
            WeaponType::Sword => 4,
            WeaponType::Dagger => 46,
            WeaponType::TwoHandSword => 50,
            WeaponType::Bow => 20,
            WeaponType::Spear => 24,
            WeaponType::Blunt => 11,
            WeaponType::Staff => 40,
            WeaponType::Gauntlet => 62,
            WeaponType::Claw => 58,
            WeaponType::Edoryu => 54,
            WeaponType::SingleBow => 20,
            WeaponType::SingleSpear => 24,
            WeaponType::TwoHandBlunt => 1,
            WeaponType::TwoHandStaff => 40,
            WeaponType::Kiringku => 58,
            WeaponType::ChainSword => 24,
        }
    }
}

impl From<String> for WeaponType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "sword" => Self::Sword,
            "dagger" => Self::Dagger,
            "tohandsword" => Self::TwoHandSword,
            "bow" => Self::Bow,
            "spear" => Self::Spear,
            "blunt" => Self::Blunt,
            "staff" => Self::Staff,
            "gauntlet" => Self::Gauntlet,
            "claw" => Self::Claw,
            "edoryu" => Self::Edoryu,
            "singlebow" => Self::SingleBow,
            "singlespear" => Self::SingleSpear,
            "tohandblunt" => Self::TwoHandBlunt,
            "tohandstaff" => Self::TwoHandStaff,
            "kiringku" => Self::Kiringku,
            "chainsword" => Self::ChainSword,
            _ => panic!(),
        }
    }
}

/// A weapon definition
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
    /// hit rate bonus
    hit_rate_bonus: i16,
    /// The weapon type
    wtype: WeaponType,
}

impl Weapon {
    /// Make a weapon instance
    /// TODO eliminate this function
    pub fn get_instance(&self, world_id: WorldObjectId) -> WeaponInstance {
        WeaponInstance {
            world_id,
            definition: self.clone(),
            bonus: WeaponStatus::Normal,
            enchanted: 0,
        }
    }
}

impl mysql::prelude::FromRow for Weapon {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        let wt: String = row.get("type").ok_or(mysql::FromRowError(row.clone()))?;
        Ok(Self {
            id: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            weight: row.get(6).ok_or(mysql::FromRowError(row.clone()))?,
            inventory_graphic: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            ground_graphic: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            unidentified: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            identified: row.get(3).ok_or(mysql::FromRowError(row.clone()))?,
            max_use_time: row.get(44).ok_or(mysql::FromRowError(row.clone()))?,
            hit_rate_bonus: row
                .get("hitmodifier")
                .ok_or(mysql::FromRowError(row.clone()))?,
            wtype: wt.into(),
        })
    }
}

/// Temporary enchantments that can be applied to a weapon
#[derive(Clone, Debug)]
pub enum WeaponStatus {
    /// The item is holy
    Holy,
    /// The item is enchanted with enchant weapon
    Enchant,
    /// The item is blessed
    Blessed,
    /// The item benefits from shadow fang
    ShadowFang,
    /// There are no benefits applied
    Normal,
}

impl WeaponStatus {
    /// Lookup the hit bonus due to temporary enchantments
    fn hit_bonus(&self) -> i16 {
        match self {
            WeaponStatus::Holy => 1,
            WeaponStatus::Enchant => 0,
            WeaponStatus::Blessed => 2,
            WeaponStatus::ShadowFang => 0,
            WeaponStatus::Normal => 0,
        }
    }

    /// Lookup the magical damage bonus due to temporary enchantments
    fn dmg_bonus(&self) -> i16 {
        match self {
            WeaponStatus::Holy => 0,
            WeaponStatus::Enchant => 2,
            WeaponStatus::Blessed => 2,
            WeaponStatus::ShadowFang => 5,
            WeaponStatus::Normal => 0,
        }
    }

    /// Lookup if the weapon applies any holy damage
    fn holy_damage(&self) -> i16 {
        if let WeaponStatus::Holy = self {
            1
        } else {
            0
        }
    }
}

/// A player usable weapon
#[derive(Clone, Debug)]
pub struct WeaponInstance {
    /// Item definition
    definition: Weapon,
    /// World object id
    world_id: WorldObjectId,
    /// Temporary bonus
    bonus: WeaponStatus,
    /// Enchantment level
    enchanted: i8,
}

impl WeaponInstance {
    /// Calculate the bonus to hit rate for the weapon
    pub fn hit_rate_bonus(&self) -> i16 {
        self.definition.hit_rate_bonus + self.bonus.hit_bonus() + self.enchanted as i16 / 2
    }

    /// Is the weapon a ranged weapon
    pub fn is_ranged(&self) -> bool {
        /// TODO
        false
    }

    /// What is the range of this weapon in map units?
    pub fn range(&self) -> u8 {
        ///TODO
        1
    }
}

impl ItemTrait for WeaponInstance {
    fn world_id(&self) -> WorldObjectId {
        self.world_id
    }

    fn get_type(&self) -> super::ItemType {
        super::ItemType::Weapon(self.definition.wtype)
    }

    fn weight(&self) -> u32 {
        self.definition.weight
    }

    fn db_id(&self) -> u32 {
        self.definition.id
    }

    fn ground_icon(&self) -> u16 {
        self.definition.ground_graphic
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
            icon: self.definition.inventory_graphic,
            blessing: common::packet::ItemBlessing::Normal,
            count: stuff.count,
            identified: if stuff.identified { 1 } else { 0 },
            description,
            ed: Vec::new(),
        }
    }
}
