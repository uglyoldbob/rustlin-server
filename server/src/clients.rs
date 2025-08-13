//! For managing clients logged into the server
use std::collections::HashSet;

/// A list of client ids
#[derive(Debug)]
pub struct ClientList {
    /// The list of existing clients
    list: HashSet<u32>,
    /// The next index to use for a new client
    index: u32,
}

impl ClientList {
    /// Construct an empty list
    pub fn new() -> ClientList {
        ClientList {
            list: HashSet::new(),
            index: 0,
        }
    }

    /// Add a new client to the list
    pub fn new_entry(&mut self) -> u32 {
        while self.list.contains(&self.index) {
            self.index = self.index.wrapping_add(1);
        }
        let t = self.index;
        self.list.insert(t);
        self.index = self.index.wrapping_add(1);
        t
    }

    /// Remove a client from the list
    pub fn remove_entry(&mut self, u: u32) {
        self.list.remove(&u);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraparound() {
        let mut l = ClientList {
            list: HashSet::new(),
            index: 0xffffffff,
        };
        for _ in 0..5 {
            l.new_entry();
        }
    }
}
