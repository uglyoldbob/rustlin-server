//! This holds code generally used for all objects in the game

use std::collections::{HashMap, HashSet};

use crate::{character::FullCharacter, server_message::ServerMessage, world::WorldObjectId};

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

    /// Add an object to the list
    pub fn add_object(&mut self, i: WorldObjectId) {
        self.items.insert(i);
    }

    /// Remove an object from the list
    pub fn remove_object(&mut self, i: WorldObjectId) {
        self.items.remove(&i);
    }

    /// Get list of changes required for this list, for adding items and deleting items as needed. Also apply the changes to self
    pub fn make_changes(&mut self, old_objects: &mut Vec<WorldObjectId>, new_objects: &mut Vec<WorldObjectId>, o: &Self) {
        // delete objects that are no longer present
        for obj in self.items.iter() {
            if !o.items.contains(obj) {
                old_objects.push(obj.clone());
            }
        }
        for obj in &o.items {
            if !self.items.contains(&obj) {
                new_objects.push(obj.clone());
            }
        }
        for o in old_objects {
            self.items.remove(o);
        }
        for o in new_objects {
            self.items.insert(o.clone());
        }
    }
}

/// The generic object trait for the server
#[enum_dispatch::enum_dispatch]
pub trait ObjectTrait {
    /// Get the location of the object
    fn get_location(&self) -> crate::character::Location;

    /// Get the object id for this object
    fn id(&self) -> super::WorldObjectId;

    /// Get the linear distance from this object to another object (as the crow flies).
    /// This assumes the objects are already on the same map
    fn linear_distance_to(&self, o: &Object) -> f32 {
        self.linear_distance(&o.get_location())
    }

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
    fn build_put_object_packet(&self) -> common::packet::Packet;

    /// Get the list of items the object is posessing
    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>>;

    /// Get the list of items, mutable
    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>>;

    /// If applicable (only for Player objects), get the object for sending messages to the user
    fn sender(&mut self) -> Option<&mut tokio::sync::mpsc::Sender<ServerMessage>>;

    /// Returns the name of the character if it is a player
    fn player_name(&self) -> Option<String>;

    /// Get the list of objects known to this object, if it matters for this object
    fn get_known_objects(&self) -> Option<Vec<Object>> { None }

    /// Add an object to the list of known objects, if applicable
    async fn add_object(&mut self, o: &Object) {}

    /// Remove an object from the list of known objects, if applicable
    async fn remove_object(&mut self, o: WorldObjectId) {}
}

/// The things that an object can be
#[enum_dispatch::enum_dispatch(ObjectTrait)]
#[derive(Debug)]
pub enum Object {
    /// A character played by a user
    Player(FullCharacter),
    /// A generic npc
    GenericNpc(super::npc::Npc),
}
