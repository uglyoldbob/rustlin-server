//! This holds code generally used for all objects in the game

use crate::character::FullCharacter;

/// The generic object trait for the server
#[enum_dispatch::enum_dispatch]
pub trait ObjectTrait {
    /// Get the location of the object
    fn get_location(&self) -> crate::character::Location;

    /// Get the object ide for this object
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
}

impl ObjectTrait for u32 {
    fn id(&self) -> u32 {
        3
    }

    fn get_location(&self) -> crate::character::Location {
        crate::character::Location {
            x: 33435,
            y: 32820,
            map: 4,
        }
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
        common::packet::ServerPacket::PutObject {
            x: 33435,
            y: 32820,
            id: 3,
            icon: 29,
            status: 0,
            direction: 1,
            light: 7,
            speed: 50,
            xp: 1235,
            alignment: -2767,
            name: "steve".to_string(),
            title: "".to_string(),
            status2: 0,
            pledgeid: 0,
            pledgename: "".to_string(),
            owner_name: "".to_string(),
            v1: 0,
            hp_bar: 12,
            v2: 0,
            level: 0,
        }
        .build()
    }
}

/// The things that an object can be
#[enum_dispatch::enum_dispatch(ObjectTrait)]
#[derive(Debug)]
pub enum Object {
    Test(u32),
    Player(FullCharacter),
}
