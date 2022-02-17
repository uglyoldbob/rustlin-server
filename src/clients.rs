use std::collections::HashSet;

pub struct ClientList {
	list: HashSet<u32>,
	index: u32,
}

impl ClientList {
	pub fn new() -> ClientList {
		ClientList{
			list: HashSet::new(),
			index: 0,
		}
	}
}