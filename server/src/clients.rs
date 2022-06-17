use std::collections::HashSet;

pub struct ClientList {
    list: HashSet<u32>,
    index: u32,
}

impl ClientList {
    pub fn new() -> ClientList {
        ClientList {
            list: HashSet::new(),
            index: 0,
        }
    }

    pub fn new_entry(&mut self) -> u32 {
        while self.list.contains(&self.index) {
            self.index = self.index.wrapping_add(1);
        }
        let t = self.index;
        self.list.insert(t);
        self.index = self.index.wrapping_add(1);
        t
    }

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
