use std::{collections::HashMap, sync::{Arc, Mutex}};

/// Represents the world for a server
pub struct World {
    users: Arc<Mutex<HashMap<u32, String>>>,
}

impl World {
    /// Construct a new server world
    pub fn new() -> Self {
        Self {
           users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        let users = self.users.lock().unwrap();
        users.len() as u16
    }
}
