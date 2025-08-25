//! This holds code generally used for all objects in the game

use std::{
    collections::{hash_set::Difference, HashMap, HashSet},
    hash::RandomState,
};

use crate::{
    character::FullCharacter,
    world::{World, WorldObjectId},
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
