//! NPC related code

use std::collections::HashMap;

#[derive(Debug)]
pub struct Npc {
    /// The object id for the npc
    id: u32,
    /// Where the npc currently exists
    location: crate::character::Location,
}

impl Npc {
    /// Build a new Npc, this is a temporary function for testing
    pub fn new(id: u32, location: crate::character::Location) -> Self {
        Self { id, location }
    }
}

impl super::object::ObjectTrait for Npc {
    fn get_location(&self) -> crate::character::Location {
        self.location
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn player_name(&self) -> Option<String> {
        None
    }

    fn get_items(&self) -> Option<&HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, super::item::ItemInstance>> {
        None
    }

    fn sender(
        &mut self,
    ) -> Option<&mut tokio::sync::mpsc::Sender<crate::server_message::ServerMessage>> {
        None
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
        common::packet::ServerPacket::PutObject {
            x: self.location.x,
            y: self.location.y,
            id: self.id,
            icon: 29,
            status: 0,
            direction: self.location.direction,
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
            hp_bar: 255,
            v2: 0,
            level: 54,
        }
        .build()
    }
}
