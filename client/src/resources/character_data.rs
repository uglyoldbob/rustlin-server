use crate::widgets::CharacterDisplayType;

#[derive(Clone)]
pub struct CharacterData {
    pub t: CharacterDisplayType,
    pub name: String,
    pub pledge: String,
    pub alignment: i16,
    pub hp: u16,
    pub mp: u16,
    pub ac: i8,
    pub level: u8,
    pub str: u8,
    pub dex: u8,
    pub con: u8,
    pub wis: u8,
    pub cha: u8,
    pub int: u8,
}

impl CharacterData {
    pub fn new() -> Self {
        Self {
            t: CharacterDisplayType::NewCharacter,
            name: "".to_string(),
            pledge: "".to_string(),
            alignment: 0,
            hp: 0,
            mp: 0,
            ac: 0,
            level: 0,
            str: 0,
            dex: 0,
            con: 0,
            wis: 0,
            cha: 0,
            int: 0,
        }
    }
}
