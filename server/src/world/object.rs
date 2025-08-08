//! This holds code generally used for all objects in the game

use std::collections::HashMap;

use crate::character::FullCharacter;

/// The generic object trait for the server
#[enum_dispatch::enum_dispatch]
pub trait ObjectTrait {
    /// Get the location of the object
    fn get_location(&self) -> crate::character::Location;

    /// Get the object id for this object
    fn id(&self) -> u32;

    /// Get the linear distance from this object to another object (as the crow flies).
    /// This assumes the objects are already on the same map
    fn linear_distance_to(&self, o: &Object) -> f32 {
        self.linear_distance(&o.get_location())
    }

    /// Get the linear distance between the location of this object and the specified location (as the crow flies).
    /// This assumes the objects are already on the same map
    fn linear_distance(&self, l2: &crate::character::Location) -> f32 {
        let l1 = self.get_location();
        let deltax = if l1.x > l2.x {
            l1.x - l2.x
        } else {
            l2.x - l1.x
        };
        let deltay = if l1.y > l2.y {
            l1.y - l2.y
        } else {
            l2.y - l1.y
        };
        let sum = ((deltax as u32) * (deltax as u32) + (deltay as u32) * (deltay as u32)) as f32;
        sum.sqrt()
    }

    /// Build a packet for placing the object on the map for a user
    fn build_put_object_packet(&self) -> common::packet::Packet;

    /// Get the list of items the object is posessing
    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>>;

    /// Get the list of items, mutable
    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>>;
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
