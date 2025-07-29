use std::{collections::HashMap, sync::{Arc, Mutex}};

/// Represents the world for a server
pub struct World {
    /// The users logged into the world
    users: Arc<Mutex<HashMap<u32, String>>>,
    /// The id generator for users
    client_ids: Arc<Mutex<crate::ClientList>>,
}

impl World {
    /// Construct a new server world
    pub fn new() -> Self {
        Self {
           users: Arc::new(Mutex::new(HashMap::new())),
           client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
        }
    }

    /// Register a new user
    pub fn register_user(&self) -> u32 {
        let mut c = self.client_ids.lock().unwrap();
        c.new_entry()
    }

    /// Unregister a user
    pub fn unregister_user(&self, uid: u32) {
        let mut c = self.client_ids.lock().unwrap();
        c.remove_entry(uid);
        let mut d = self.users.lock().unwrap();
        d.remove(&uid);
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        let users = self.users.lock().unwrap();
        users.len() as u16
    }
}
