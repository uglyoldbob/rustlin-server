//! Code for map definitions

use std::{collections::HashMap, sync::Arc};

use common::packet::{ServerPacket, ServerPacketSender};

use crate::world::{object::ObjectTrait, ObjectRef};

use super::WorldObjectId;

/// Represents the dynamic information of a map
#[derive(Debug)]
pub struct MapInfo {
    /// The objects on the map
    objects: HashMap<WorldObjectId, Arc<tokio::sync::Mutex<super::object::Object>>>,
}

impl MapInfo {
    /// Construct a new map info object
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    /// Add an object to the map
    pub async fn add_new_object(&mut self, new_o: super::object::Object) {
        self.objects
            .insert(new_o.id(), Arc::new(tokio::sync::Mutex::new(new_o)));
    }

    /// Get a location of an object reference
    pub async fn get_location(&self, r: super::ObjectRef) -> Option<super::Location> {
        if let Some(o) = self.get_object(r) {
            return Some(o.lock().await.get_location());
        }
        None
    }

    /// Get an object from the map
    /// Get an object from the world
    pub fn get_object(
        &self,
        r: super::ObjectRef,
    ) -> Option<Arc<tokio::sync::Mutex<super::object::Object>>> {
        if let Some(o) = self.objects.get(&r.id) {
            return Some(o.clone());
        }
        None
    }

    /// Get an object from the object id
    pub fn get_object_from_id(
        &self,
        id: WorldObjectId,
    ) -> Option<Arc<tokio::sync::Mutex<super::object::Object>>> {
        if let Some(o) = self.objects.get(&id) {
            return Some(o.clone());
        }
        None
    }

    /// Get an iterator over all objects
    pub fn objects_iter(
        &self,
    ) -> std::collections::hash_map::Iter<
        '_,
        WorldObjectId,
        Arc<tokio::sync::Mutex<super::object::Object>>,
    > {
        self.objects.iter()
    }

    /// Move an object on the map
    pub async fn move_object(
        &mut self,
        r: super::ObjectRef,
        new_loc: super::Location,
        mut pw: Option<&mut ServerPacketSender>,
    ) -> Result<(), super::ClientError> {
        let mut object_list = super::ObjectList::new();
        let objr = self.get_object(r).unwrap();
        let mut object_to_move = objr.lock().await;
        object_to_move.set_location(new_loc);
        for (id, o) in &mut self.objects {
            if *id != r.id {
                if o.lock().await.linear_distance(&new_loc) < 17.0 {
                    object_list.add_object(*id);
                }
            }
        }
        {
            let mut old_objects = Vec::new();
            let mut new_objects = Vec::new();
            if let Some(ol) = object_to_move.get_known_objects() {
                ol.find_changes(&mut old_objects, &mut new_objects, &object_list);
            }
            let thing_move_packet = object_to_move.build_move_object_packet();
            for o in object_list.get_objects() {
                if let Some(o) = self.objects.get_mut(o) {
                    if let Some(s) = o.lock().await.sender() {
                        let _ = s.send(thing_move_packet.clone()).await;
                    }
                }
            }
            for objid in old_objects {
                if let Some(pw) = &mut pw {
                    pw.send_packet(ServerPacket::RemoveObject(objid.get_u32()))
                        .await?;
                }
                if let Some(obj) = self.objects.get_mut(&r.id) {
                    if let Some(pw) = &mut pw {
                        pw.send_packet(ServerPacket::RemoveObject(objid.get_u32()))
                            .await?;
                    }
                    obj.lock().await.remove_object(objid).await;
                }
            }
            for objid in new_objects {
                if let Some(pw) = &mut pw {
                    if let Some(obj) = self.objects.get_mut(&objid) {
                        pw.send_packet(obj.lock().await.build_put_object_packet())
                            .await?;
                    }
                }
                if let Some(obj) = self.objects.get_mut(&r.id) {
                    obj.lock().await.add_object(objid).await;
                }
            }
        }
        Ok(())
    }

    /// Remove an object from the map
    pub async fn remove_object(&mut self, id: WorldObjectId) {
        self.objects.remove(&id);
        for o in &mut self.objects {
            o.1.lock().await.remove_object(id).await;
        }
    }
}
