//! Code for map definitions

use std::{collections::HashMap, sync::Arc};

use common::packet::{ServerPacket, ServerPacketSender};

use crate::world::object::ObjectTrait;

use super::WorldObjectId;

/// Represents the dynamic information of a map
#[derive(Debug)]
pub struct MapInfo {
    /// The objects on the map
    objects: HashMap<WorldObjectId, super::object::Object>,
}

/// This object is used to collect all of the packets destined for other objects
/// that have senders
pub struct SendsToAnotherObject {
    data: HashMap<WorldObjectId, (tokio::sync::mpsc::Sender<ServerPacket>, Vec<ServerPacket>)>,
}

impl Drop for SendsToAnotherObject {
    fn drop(&mut self) {}
}

impl std::future::AsyncDrop for SendsToAnotherObject {
    async fn drop(mut self: std::pin::Pin<&mut Self>) {
        self.send_all().await
    }
}

impl SendsToAnotherObject {
    /// Construct an empty list
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Add to the list
    pub fn add_to_list(
        &mut self,
        id: WorldObjectId,
        s: tokio::sync::mpsc::Sender<ServerPacket>,
        p: ServerPacket,
    ) {
        if let Some(a) = self.data.get_mut(&id) {
            a.1.push(p);
        } else {
            let d = (s, vec![p]);
            self.data.insert(id, d);
        }
    }

    /// Send all packets
    async fn send_all(&mut self) {
        for (id, s) in &self.data {
            for p in &s.1 {
                let _ = s.0.send(p.clone()).await;
            }
        }
        self.data.clear();
    }
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

    /// Move an object on the map
    pub fn move_object(
        &mut self,
        r: super::ObjectRef,
        new_loc: super::Location,
        mut pw: Option<&mut ServerPacketSender>,
        list: &mut SendsToAnotherObject,
    ) -> Result<(), super::ClientError> {
        let mut object_list = super::ObjectList::new();
        let thing_move_packet = {
            self.get_object_mut(r).unwrap().set_location(new_loc);
            for (id, o) in &mut self.objects {
                if *id != r.id {
                    if o.linear_distance(&new_loc) < 17.0 {
                        object_list.add_object(*id);
                    }
                }
            }
            {
                let mut old_objects = Vec::new();
                let mut new_objects = Vec::new();
                if let Some(ol) = self.get_object_mut(r).unwrap().get_known_objects() {
                    ol.find_changes(&mut old_objects, &mut new_objects, &object_list);
                }
                for objid in old_objects {
                    if let Some(pw) = &mut pw {
                        pw.queue_packet(ServerPacket::RemoveObject(objid.get_u32()));
                    }

                    if let Some(pw) = &mut pw {
                        pw.queue_packet(ServerPacket::RemoveObject(objid.get_u32()));
                    }
                    self.get_object_mut(r).unwrap().remove_object(objid);
                }
                for objid in new_objects {
                    if let Some(pw) = &mut pw {
                        if let Some(obj) = self.objects.get_mut(&objid) {
                            pw.queue_packet(obj.build_put_object_packet());
                        }
                    }
                    self.get_object_mut(r).unwrap().add_object(objid);
                }
                let thing_move_packet = self.get_object_mut(r).unwrap().build_move_object_packet();
                thing_move_packet
            }
        };
        for o in object_list.get_objects() {
            if r.id == *o {
                log::error!("Triggering a bug?");
                panic!();
            }
            let sender = if let Some(o) = self.objects.get_mut(o) {
                let sender = { o.sender().map(|s| s.clone()) };
                sender
            } else {
                None
            };
            if let Some(s) = sender {
                list.add_to_list(*o, s, thing_move_packet.clone());
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
