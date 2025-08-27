//! This holds code generally used for all objects in the game

use std::{
    collections::{hash_set::Difference, HashMap, HashSet},
    hash::RandomState,
};

use crate::{
    character::FullCharacter,
    world::{item::ItemTrait, WorldObjectId},
};

/// The effects that can be on an object
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Effect {
    // absolute barrier
    AbsoluteBarrier,
    /// Additional fire
    AdditionalFire,
    /// area of silence
    AreaSilence,
    /// Barlog?
    Barlog,
    /// Berserkers
    Berserkers,
    /// Blind
    Blind,
    /// Blind hiding
    BlindHiding,
    /// bloodlust
    BloodLust,
    /// blue potion
    BluePotion,
    /// bounce attack
    BounceAttack,
    // brave
    Brave,
    /// Burning spirit
    BurningSpirit,
    /// burning weapon
    BurningWeapon,
    /// concentration
    Concentration,
    /// cooking
    Cooking,
    /// cooking
    Cooking1N,
    //// cooking
    Cooking1S,
    /// cooking
    Cooking2N,
    //// cooking
    Cooking2S,
    /// cooking
    Cooking3N,
    //// cooking
    Cooking3S,
    /// Counter barrier
    CounterBarrier,
    /// Counter magic
    CounterMagic,
    /// cube stuff
    CubeBalance,
    /// cube stuff
    CubeIgnition,
    /// cube stuff
    CubeQuake,
    /// cube stuff
    CubeShock,
    /// Darkness
    Darkness,
    /// Decay potion
    DecayPotion,
    /// Decrease weight
    DecreaseWeight,
    /// double break
    DoubleBreak,
    /// dragon skin
    DragonSkin,
    /// dress evasion
    DressEvasion,
    /// earth bind
    EarthBind,
    /// elemental fire
    ElementalFire,
    /// elemental protection
    ElementalProtection,
    /// elf brave
    ElfBrave,
    /// enchant venom
    EnchantVenom,
    /// Entangle
    Entangle,
    /// Erase magic
    EraseMagic,
    /// Exotic vitalize
    ExoticVitalize,
    /// Fire bless
    FireBless,
    /// fire weapon
    FireWeapon,
    /// Floating eye
    FloatingEye,
    /// freeze
    Freeze,
    /// freezing blizzard
    FreezingBlizzard,
    /// freezing breath
    FreezingBreath,
    /// greater haste
    GreaterHaste,
    /// Haste
    Haste,
    /// holy mithril powder
    HolyMithrilPowder,
    /// holy walk
    HolyWalk,
    /// Holy water
    HolyWater,
    /// Holy water of eva
    HolyWaterEva,
    /// ice lance
    IceLance,
    /// illisionist avatar
    IllusionistAvatar,
    /// immune to harm
    ImmuneToHarm,
    /// invisibility
    Invisibility,
    /// joy of pain
    JoyOfPain,
    /// light
    Light,
    /// mass slow
    MassSlow,
    /// meditation
    Meditation,
    /// Mirror image
    MirrorImage,
    /// mortal body
    MortalBody,
    /// moving acceleration
    MovingAcceleration,
    // Natures touch
    NaturesTouch,
    /// No chat allowed
    NoChat,
    /// patience
    Patience,
    /// poison
    Poison,
    /// poison paralyzed
    PoisonParalyzed,
    /// poison paralyzing
    PoisonParalyzing,
    /// poison silence
    PoisonSilence,
    /// pollute water
    PolluteWater,
    /// poly effect
    PolyEffect,
    /// Polymorph
    Polymorph,
    /// Reduction armor
    ReductionArmor,
    /// resist fear
    ResistFear,
    /// ri brave?
    RiBrave,
    /// shock stun
    ShockStun,
    /// silence
    Silence,
    /// Slow
    Slow,
    /// Solid carriage
    SolidCarriage,
    /// soul of flame
    SoulOfFlame,
    // Striker gale
    StrikerGale,
    /// uncanny dodge
    UncannyDodge,
    /// UnderwaterBreath
    UnderwaterBreath,
    /// venom resist
    VenomResist,
    /// water life
    WaterLife,
    /// wind walk
    WindWalk,
    /// Wind shackle
    WindShackle,
    /// Wisdom potion
    WisdomPotion,
    /// Yahee
    Yahee,
}

/// A helper struct for managin a list of objects known to a player
#[derive(Debug)]
pub struct ObjectList {
    /// The items
    items: HashSet<WorldObjectId>,
}

impl ObjectList {
    /// Construct a blank list
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }

    /// The difference function, self - other
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, WorldObjectId, RandomState> {
        self.items.difference(&other.items)
    }

    /// Add an object to the list
    pub fn add_object(&mut self, i: WorldObjectId) {
        self.items.insert(i);
    }

    /// Remove an object from the list
    pub fn remove_object(&mut self, i: WorldObjectId) {
        self.items.remove(&i);
    }

    /// Get the list of objects
    pub fn get_objects(&self) -> &HashSet<WorldObjectId> {
        &self.items
    }

    /// Get list of changes required for this list, for adding items and deleting items as needed.
    pub fn find_changes(
        &self,
        old_objects: &mut Vec<WorldObjectId>,
        new_objects: &mut Vec<WorldObjectId>,
        o: &Self,
    ) {
        // delete objects that are no longer present
        for obj in self.items.iter() {
            if !o.items.contains(obj) {
                old_objects.push(*obj);
            }
        }
        for obj in &o.items {
            if !self.items.contains(obj) {
                new_objects.push(*obj);
            }
        }
    }
}

/// The generic object trait for the server
#[enum_dispatch::enum_dispatch]
pub trait ObjectTrait {
    /// Get the location of the object
    fn get_location(&self) -> crate::character::Location;

    /// Set the location of the object
    fn set_location(&mut self, l: crate::character::Location);

    /// Get the object id for this object
    fn id(&self) -> super::WorldObjectId;

    /// Get the linear distance between the location of this object and the specified location (as the crow flies).
    /// This assumes the objects are already on the same map
    fn linear_distance(&self, l2: &crate::character::Location) -> f32 {
        let l1 = self.get_location();
        l1.linear_distance(l2)
    }

    /// Get the manhattan distance between the location of this object and the specified location.
    /// This assumes the objects are already on the same map
    fn manhattan_distance(&self, l2: &crate::character::Location) -> u16 {
        let l1 = self.get_location();
        l1.manhattan_distance(l2)
    }

    /// Build a packet for placing the object on the map for a user
    fn build_put_object_packet(&self) -> common::packet::ServerPacket;

    /// Build a packet for moving the object on the map
    fn build_move_object_packet(&self) -> common::packet::ServerPacket {
        let id = self.id();
        let olocation = self.get_location();
        let location = olocation.compute_for_move();
        common::packet::ServerPacket::MoveObject {
            id: id.get_u32(),
            x: location.x,
            y: location.y,
            direction: location.direction,
        }
    }

    /// Get the list of items the object is posessing
    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    /// Get the list of items, mutable
    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    /// If applicable (only for Player objects), get the object for sending messages to the user
    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<crate::world::WorldResponse>> {
        None
    }

    /// Returns the name of the character if it is a player
    fn player_name(&self) -> Option<String> {
        None
    }

    /// Returns the name of the object
    fn object_name(&self) -> String;

    /// Get the list of objects known to this object, if it matters for this object
    fn get_known_objects(&self) -> Option<&ObjectList> {
        None
    }

    /// Add an object to the list of known objects, if applicable
    fn add_object(&mut self, o: WorldObjectId) {}

    /// Remove an object from the list of known objects, if applicable
    fn remove_object(&mut self, o: WorldObjectId) {}

    /// Return true if the object can initiate a server shutdown
    fn can_shutdown(&self) -> bool {
        false
    }

    /// The weapon the object is wielding, if there is one
    fn weapon(&self) -> Option<&crate::world::item::WeaponInstance>;

    /// Get the object type for attacking and being attacked
    fn attack_type(&self) -> BasicObjectType;

    /// The base attack rate
    fn base_attack_rate(&self) -> i16;

    /// The bonus/penalty to attacking success rate based on object strength
    fn str_attack_hit_bonus(&self) -> i8;

    /// The bonus/penalty for damage based on strength
    fn str_attack_dmg_bonus(&self) -> i8;

    /// The bonus/penalty to attacking success rate based on object dexterity
    fn dex_attack_hit_bonus(&self) -> i8;

    /// The bonus/penalty for damage based on dexterity
    fn dex_attack_dmg_bonus(&self) -> i8;

    /// The bonus/penalty a player has for attacks
    fn hit_rate_bonus(&self) -> i16;

    /// Hit rate bonuses for things like auras and stuff
    fn other_hit_rate_bonus(&self) -> i16;

    /// The bonus/penalty a player has for ranged weapons
    fn ranged_hit_rate_bonus(&self) -> i16;

    /// The values for calculating critical hit and critical miss
    fn critical_hit_miss_values(&self) -> (i16, i16);

    /// Get the total weight the object carries
    fn get_weight(&self) -> u32 {
        if let Some(items) = self.get_items() {
            let mut total = 0;
            for i in items {
                total += i.1.weight();
            }
            total
        } else {
            0
        }
    }

    /// Get the armor class of the object
    fn armor_class(&self) -> i8;

    /// Get the max weight that can be carried
    fn max_weight(&self) -> u32;

    /// The percentage of weight being carried
    fn weight_percentage(&self) -> f32 {
        (self.get_weight() as f32 / self.max_weight() as f32).min(1.0)
    }

    /// The list of effects currently in place for the object
    fn get_effects(&self) -> &HashSet<Effect>;

    /// The list of effects, mutable, for the object
    fn effects_mut(&mut self) -> &mut HashSet<Effect>;

    /// Use a single unit of ammunition for the weapon the object is currently using.
    /// Return if using the ammunition suceeded
    fn use_weapon_ammunition(&mut self) -> bool;

    /// Get the evasive rating for the object
    fn get_evasive_rating(&self) -> u8;

    /// Some objects require the attacker to have a particular status in order to hit them
    fn apply_required_status(&self, effects: &HashSet<Effect>, rate: &mut u8);

    /// Some objects require the attacker to be in a particular polymorph to hit them
    fn apply_required_polymorph(&self, poly: Option<u32>, rate: &mut u8);

    /// Get the polymorph if applicable for the object
    fn get_polymorph(&self) -> Option<u32>;
}

/// The things that an object can be
#[enum_dispatch::enum_dispatch(ObjectTrait)]
#[derive(Debug)]
pub enum Object {
    /// A character played by a user
    Player(FullCharacter),
    /// A generic npc
    GenericNpc(super::npc::Npc),
    /// A generic monster
    Monster(super::monster::Monster),
    /// An item on the ground
    GroundItem(super::item::ItemWithLocation),
}

impl Object {
    /// Is the object a player?
    pub fn is_player(&self) -> bool {
        if let Object::Player(_f) = self {
            true
        } else {
            false
        }
    }
}

/// The types of attacked for damage
pub enum BasicObjectType {
    /// The object is a user playing a character
    Player,
    /// The object is an npc
    Npc,
    /// The object is a monster
    Monster,
    /// The object is something else
    Other,
}

impl Damage {
    /// Construct a new attack
    pub fn new(attacker: &mut Object) -> Option<Self> {
        let mut roll_bonus = attacker.base_attack_rate();
        let mut dmg_bonus = 0;
        roll_bonus += attacker.str_attack_hit_bonus() as i16;
        roll_bonus += attacker.dex_attack_hit_bonus() as i16;
        let (wtype, range) = if let Some(weapon) = attacker.weapon() {
            roll_bonus += weapon.hit_rate_bonus();
            if weapon.is_ranged() {
                dmg_bonus += attacker.dex_attack_dmg_bonus();
                roll_bonus += attacker.ranged_hit_rate_bonus();
            } else {
                dmg_bonus += attacker.str_attack_dmg_bonus();
                roll_bonus += attacker.hit_rate_bonus();
            }
            if let crate::world::item::ItemType::Weapon(wt) = weapon.get_type() {
                (Some(wt), weapon.range())
            } else {
                (None, 1)
            }
        } else {
            if let Object::Monster(_m) = attacker {
                /// TODO actually get the range from the monster
                (None, 17)
            } else {
                (None, 1)
            }
        };
        if !attacker.use_weapon_ammunition() {
            ///TODO need a better way to indicate that the attack never happened, as opposed to missing
            return None;
        }
        let carrying = attacker.weight_percentage();
        if carrying <= 1.0 / 3.0 {
            //nothing
        } else if carrying < 0.5 {
            roll_bonus -= 1;
        } else if carrying < 2.0 / 3.0 {
            roll_bonus -= 3;
        } else if carrying < 5.0 / 6.0 {
            roll_bonus -= 5;
        } else {
            roll_bonus -= 5;
        }
        roll_bonus += attacker.other_hit_rate_bonus();
        let critical = attacker.critical_hit_miss_values();
        Some(Self {
            origin: attacker.get_location(),
            range,
            atype: attacker.attack_type(),
            attacker_poly: attacker.get_polymorph(),
            effects: attacker.get_effects().to_owned(),
            attack_roll_bonus: roll_bonus,
            critical,
            weapon_type: wtype,
            damage_bonus: dmg_bonus,
        })
    }

    /// Calculate the damage that might be applied, return true if the attack hit
    pub fn run_damage(&mut self, attacked: &mut Object) -> Option<u16> {
        if !self.should_hit(attacked) {
            return None;
        }
        match self.atype {
            BasicObjectType::Player => match attacked.attack_type() {
                BasicObjectType::Player => {
                    todo!()
                }
                BasicObjectType::Npc => todo!(),
                BasicObjectType::Monster => todo!(),
                BasicObjectType::Other => None,
            },
            BasicObjectType::Npc => match attacked.attack_type() {
                BasicObjectType::Player => todo!(),
                BasicObjectType::Npc => todo!(),
                BasicObjectType::Monster => todo!(),
                BasicObjectType::Other => None,
            },
            BasicObjectType::Monster => match attacked.attack_type() {
                BasicObjectType::Player => todo!(),
                BasicObjectType::Npc => todo!(),
                BasicObjectType::Monster => todo!(),
                BasicObjectType::Other => None,
            },
            BasicObjectType::Other => None,
        }
    }

    ///Calculate if the attacked object is hit
    fn should_hit(&mut self, attacked: &Object) -> bool {
        use rand::Rng;

        let attack_distance = attacked
            .get_location()
            .linear_distance(&self.origin)
            .round() as u32;
        if attack_distance > self.range as u32 {
            return false;
        }

        let defender_effects = attacked.get_effects();
        if defender_effects.contains(&Effect::UncannyDodge) {
            self.attack_roll_bonus -= 5;
        }
        if defender_effects.contains(&Effect::MirrorImage) {
            self.attack_roll_bonus -= 5;
        }
        if defender_effects.contains(&Effect::ResistFear) {
            self.attack_roll_bonus += 5;
        }

        let attacker_roll =
            rand::thread_rng().gen_range(0..20i16) + 1 + self.attack_roll_bonus as i16 - 10;
        let special = if attacker_roll <= (self.attack_roll_bonus + self.critical.0) {
            SpecialAttack::CriticalMiss
        } else if attacker_roll >= (self.attack_roll_bonus + self.critical.1) {
            SpecialAttack::CriticalHit
        } else {
            SpecialAttack::Normal
        };
        let ac = attacked.armor_class() as i16;
        let defender_roll = if ac >= 0 {
            10 - ac
        } else {
            let max_roll = (ac as f32 * -1.5).round() as i16;
            10 - rand::thread_rng().gen_range(0..max_roll) + 1
        };
        let mut hit_percent: u8 = match special {
            SpecialAttack::Normal => {
                if attacker_roll > defender_roll {
                    100
                } else if attacker_roll <= defender_roll {
                    0
                } else {
                    todo!()
                }
            }
            SpecialAttack::CriticalMiss => 0,
            SpecialAttack::CriticalHit => 100,
        };
        if let Some(crate::world::item::WeaponType::Kiringku) = self.weapon_type {
            hit_percent = 100;
        }
        if defender_effects.contains(&Effect::AbsoluteBarrier)
            || defender_effects.contains(&Effect::IceLance)
            || defender_effects.contains(&Effect::FreezingBlizzard)
            || defender_effects.contains(&Effect::FreezingBreath)
            || defender_effects.contains(&Effect::EarthBind)
        {
            hit_percent = 0;
        }
        attacked.apply_required_status(&self.effects, &mut hit_percent);
        attacked.apply_required_polymorph(self.attacker_poly, &mut hit_percent);
        let hit_roll: u8 = rand::thread_rng().gen_range(1..=100);
        if let Some(crate::world::item::WeaponType::Bow) = self.weapon_type {
            let er = attacked.get_evasive_rating();
            let er_roll = rand::thread_rng().gen_range(1..=100);
            er < er_roll && hit_percent > hit_roll
        } else {
            hit_percent > hit_roll
        }
    }
}

/// Specifies if the attack is normal, always misses, or always hits
enum SpecialAttack {
    /// Normal
    Normal,
    /// Chance of hitting is none
    CriticalMiss,
    /// Chance of hitting is maximum
    CriticalHit,
}

/// The damage that can be applied to an object
pub struct Damage {
    /// Where the damage originated from
    origin: super::Location,
    /// The range of the attack
    range: u8,
    /// The type of object doing the damage
    atype: BasicObjectType,
    /// The roll for attack bonus by the attacker
    attack_roll_bonus: i16,
    /// The polymorph, if applicable of the attacker
    attacker_poly: Option<u32>,
    /// The effects applied to the attacker
    effects: HashSet<Effect>,
    /// Damage bonus for the attacker
    damage_bonus: i8,
    /// Attacker crititical hit and miss values
    critical: (i16, i16),
    /// The weapon type
    weapon_type: Option<crate::world::item::WeaponType>,
}
