//! Code for map definitions

use std::{collections::HashMap, sync::Arc};

use common::packet::{ServerPacket, ServerPacketSender};

use crate::world::object::{ObjectList, ObjectTrait};

use super::WorldObjectId;

/// Represents the dynamic information of a map
#[derive(Debug)]
pub struct MapInfo {
    /// The objects on the map
    objects: HashMap<WorldObjectId, super::object::Object>,
}

impl MapInfo {
    /// Construct a new map info object
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    /// Add an object to the map
    pub fn add_new_object(&mut self, new_o: super::object::Object) {
        self.objects.insert(new_o.id(), new_o);
    }

    /// Get a location of an object reference
    pub fn get_location(&self, r: super::ObjectRef) -> Option<super::Location> {
        if let Some(o) = self.get_object(r) {
            return Some(o.get_location());
        }
        None
    }

    /// Get the name of an object reference
    pub fn get_name(&self, r: super::ObjectRef) -> Option<String> {
        if let Some(o) = self.get_object(r) {
            return o.player_name();
        }
        None
    }

    /// Get an object reference from the map
    pub fn get_object(&self, r: super::ObjectRef) -> Option<&super::object::Object> {
        self.objects.get(&r.id)
    }

    /// Get a mutable reference from the map
    pub fn get_object_mut(&mut self, r: super::ObjectRef) -> Option<&mut super::object::Object> {
        self.objects.get_mut(&r.id)
    }

    /// Get an object from the object id
    pub fn get_object_from_id(&self, id: WorldObjectId) -> Option<&super::object::Object> {
        if let Some(o) = self.objects.get(&id) {
            return Some(o);
        }
        None
    }

    /// Get an iterator over all objects
    pub fn objects_iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, WorldObjectId, super::object::Object> {
        self.objects.iter()
    }

    /// The object specified is new, all objects around it are new
    pub async fn object_is_new_here(&self, r: super::ObjectRef) {
        let obj = self.objects.get(&r.id).unwrap();
        let loc = obj.get_location();
        if let Some(s) = obj.sender() {
            for (id, o) in &self.objects {
                if *id != r.id {
                    if o.linear_distance(&loc) < 17.0 {
                        if let Some(obj) = self.objects.get(id) {
                            s.send(super::WorldResponse::ServerPacket(
                                obj.build_put_object_packet(),
                            ))
                            .await;
                        }
                    }
                }
            }
        }
    }

    /// Move an object on the map
    /// 1. Find all objects in range of the new location for the moving object
    /// 2. Remove all old objects for the moving object
    /// 3. Add all new objects for the moving object
    pub async fn move_object(
        &mut self,
        r: super::ObjectRef,
        new_loc: super::Location,
    ) -> Result<(), super::ClientError> {
        let mut old_object_list = ObjectList::new();
        let mut new_object_list = ObjectList::new();
        let (old_loc, mut moving_send, pop, move_packet) = {
            let o = self.objects.get_mut(&r.id).unwrap();
            let oldloc = o.get_location();
            o.set_location(new_loc);
            (
                oldloc,
                o.sender(),
                o.build_put_object_packet(),
                o.build_move_object_packet(),
            )
        };
        for (id, o) in &mut self.objects {
            if *id != r.id {
                if o.linear_distance(&old_loc) < 17.0 {
                    old_object_list.add_object(*id);
                }
                if o.linear_distance(&new_loc) < 17.0 {
                    new_object_list.add_object(*id);
                }
            }
        }
        let remove_objects = old_object_list.difference(&new_object_list);
        let add_objects = new_object_list.difference(&old_object_list);
        for obj in remove_objects {
            if let Some(moving_send) = &mut moving_send {
                moving_send
                    .send(super::WorldResponse::ServerPacket(
                        ServerPacket::RemoveObject(obj.get_u32()),
                    ))
                    .await;
            }
            if let Some(other_obj) = self.objects.get(&obj) {
                if let Some(s) = other_obj.sender() {
                    s.send(super::WorldResponse::ServerPacket(
                        ServerPacket::RemoveObject(r.id.get_u32()),
                    ))
                    .await;
                }
            }
        }
        for obj in add_objects {
            if let Some(moving_send) = &mut moving_send {
                if let Some(obj) = self.objects.get_mut(&obj) {
                    moving_send
                        .send(super::WorldResponse::ServerPacket(
                            obj.build_put_object_packet(),
                        ))
                        .await;
                }
            }
            if let Some(other_obj) = self.objects.get(&obj) {
                if let Some(s) = other_obj.sender() {
                    s.send(super::WorldResponse::ServerPacket(pop.clone()))
                        .await;
                }
            }
        }
        for obj in new_object_list.get_objects() {
            if let Some(other_obj) = self.objects.get(&obj) {
                if let Some(s) = other_obj.sender() {
                    s.send(super::WorldResponse::ServerPacket(move_packet.clone()))
                        .await;
                }
            }
        }
        Ok(())
    }

    /// Remove an object from the map
    pub fn remove_object(&mut self, id: WorldObjectId) {
        self.objects.remove(&id);
        for o in &mut self.objects {
            o.1.remove_object(id);
        }
    }
}
