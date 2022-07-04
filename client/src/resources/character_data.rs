use crate::mode::CharacterDisplayType;

#[derive(Clone,Copy)]
pub struct CharacterData {
	pub t: CharacterDisplayType,
}

impl CharacterData {
	pub fn new() -> Self {
		Self {
			t: CharacterDisplayType::NewCharacter,
		}
	}
}
