//! This holds code generally used for all objects in the game

use crate::character::FullCharacter;

/// The generic object trait for the server
#[enum_dispatch::enum_dispatch]
pub trait ObjectTrait {
    /// Get the location of the object
    async fn get_location(&self) -> crate::character::Location;

    /// Get the object ide for this object
    async fn id(&self) -> u32;

    /// Get the linear distance from this object to another object (as the crow flies).
    /// This assumes the objects are already on the same map
    async fn linear_distance_to(&self, o: &Object) -> f32 {
        let l1 = self.get_location().await;
        let l2 = o.get_location().await;
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
}

impl ObjectTrait for u32 {
    async fn id(&self) -> u32 {
        3
    }

    async fn get_location(&self) -> crate::character::Location {
        crate::character::Location {
            x: 33435,
            y: 32820,
            map: 4,
        }
    }
}

/// The things that an object can be
#[enum_dispatch::enum_dispatch(ObjectTrait)]
#[derive(Debug)]
pub enum Object {
    Test(u32),
    Player(FullCharacter),
}