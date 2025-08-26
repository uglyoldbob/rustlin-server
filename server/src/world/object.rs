//! This holds code generally used for all objects in the game

use std::{
    collections::{hash_set::Difference, HashMap, HashSet},
    hash::RandomState,
};

use crate::{
    character::FullCharacter,
    world::{item::Weapon, World, WorldObjectId},
};

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
    fn weapon(&self) -> Option<&crate::world::item::WeaponInstance> { None }

    /// Get the object type for attacking and being attacked
    fn attack_type(&self) -> BasicObjectType;

    /// The base attack rate
    fn base_attack_rate(&self) -> i16 { 0 }

    /// The bonus/penalty to attacking success rate based on object strength
    fn str_attack_hit_bonus(&self) -> i8 { 0 }

    /// The bonus/penalty to attacking success rate based on object dexterity
    fn dex_attack_hit_bonus(&self) -> i8 { 0 }

    /// The bonus/penalty a player has for attacks
    fn hit_rate_bonus(&self) -> i16 { 0 }

    /// The bonus/penalty a player has for ranged weapons
    fn ranged_hit_rate_bonus(&self) -> i16 { 0 }

    /// Get the total weight the object carries
    fn get_weight(&self) -> u32 {
        if let Some(items) = self.get_items() {
            let mut total = 0;
            for i in items {
                total += i.1.weight();
            }
            total
        }
        else {
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
    pub fn new(attacker: &Object) -> Self {
        let mut roll_bonus = attacker.base_attack_rate();
        roll_bonus += attacker.str_attack_hit_bonus() as i16;
        roll_bonus += attacker.dex_attack_hit_bonus() as i16;
        if let Some(weapon) = attacker.weapon() {
            roll_bonus += weapon.hit_rate_bonus();
            if weapon.is_ranged() {
                roll_bonus += attacker.ranged_hit_rate_bonus();
            }
            else {
                roll_bonus += attacker.hit_rate_bonus();
            }
            let carrying = attacker.weight_percentage();
            if carrying <= 1.0/3.0 {
                //nothing
            } else if carrying < 0.5 {
                roll_bonus -= 1;
            } else if carrying < 2.0/3.0 {
                roll_bonus -= 3;
            } else if carrying < 5.0/6.0 {
                roll_bonus -= 5;
            } else {
                roll_bonus -= 5;
            }
        }
        use rand::Rng;
        let roll = rand::thread_rng().gen_range(0..20i16) + 1 + roll_bonus as i16 - 10;
        let special = if roll <= (roll_bonus - 9) {
            SpecialAttack::CriticalMiss
        } else if roll >= roll_bonus + 10 {
            SpecialAttack::CriticalHit
        } else {
            SpecialAttack::Normal
        };
        Self {
            origin: attacker.get_location(),
            atype: attacker.attack_type(),
            attack_roll: roll,
            special,
        }
    }

    /// Calculate the damage that might be applied, return true if the attack hit
    fn run_damage(&self, attacked: &mut Object) -> Option<u16> {
        if !self.should_hit(attacked) {
            return None;
        }
        match self.atype {
            BasicObjectType::Player => {
                match attacked.attack_type() {
                    BasicObjectType::Player => {
                        None
                    }
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => None,
                }
            }
            BasicObjectType::Npc => {
                match attacked.attack_type() {
                    BasicObjectType::Player => todo!(),
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => None,
                }
            }
            BasicObjectType::Monster => {
                match attacked.attack_type() {
                    BasicObjectType::Player => todo!(),
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => None,
                }
            }
            BasicObjectType::Other => {
                None
            }
        }
    }

    ///Calculate if the attacked object is hit
    fn should_hit(&self, attacked: &Object) -> bool {
        use rand::Rng;
        match self.atype {
            BasicObjectType::Player => {
                match attacked.attack_type() {
                    BasicObjectType::Player => {
                        let ac = attacked.armor_class() as i16;
                        let roll = if ac >= 0 {
                            10 - ac
                        } else {
                            let max_roll = (ac as f32 * -1.5).round() as i16;
                            10 - rand::thread_rng().gen_range(0..max_roll) + 1
                        };
                        let hit_percent : u8 = match self.special {
                            SpecialAttack::Normal => if self.attack_roll > roll {
                                100
                            } else {
                                0
                            }
                            SpecialAttack::CriticalMiss => 0,
                            SpecialAttack::CriticalHit => 100,
                        };
                        let hit_roll: u8 = rand::thread_rng().gen_range(1..=100);
                        hit_percent > hit_roll
                    }
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => false,
                }
            }
            BasicObjectType::Npc => {
                match attacked.attack_type() {
                    BasicObjectType::Player => todo!(),
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => false,
                }
            }
            BasicObjectType::Monster => {
                match attacked.attack_type() {
                    BasicObjectType::Player => todo!(),
                    BasicObjectType::Npc => todo!(),
                    BasicObjectType::Monster => todo!(),
                    BasicObjectType::Other => false,
                }
            }
            BasicObjectType::Other => {
                false
            }
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
    /// The type of object doing the damage
    atype: BasicObjectType,
    /// The roll for attack by the attacker
    attack_roll: i16,
    /// The special attack rate
    special: SpecialAttack,
}