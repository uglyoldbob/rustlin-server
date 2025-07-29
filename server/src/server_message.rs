/// Represents messages sent by the server to player clients
#[derive(Clone)]
pub enum ServerMessage {
    SystemMessage(String),
    CharacterCreateStatus(u8),
    Disconnect,
    NewCharacterDetails {
        name: String,
        pledge: String,
        class: u8,
        gender: u8,
        alignment: i16,
        hp: u16,
        mp: u16,
        ac: i8,
        level: u8,
        strength: u8,
        dexterity: u8,
        constitution: u8,
        wisdom: u8,
        charisma: u8,
        intelligence: u8,
    },
    NpcShout(String),
    RegularChat {
        id: u32,
        msg: String,
    },
    ///msg = "player name: message"
    YellChat {
        id: u32,
        msg: String,
        x: u16,
        y: u16,
    },
    ///msg = "<player name> message"
    GlobalChat(String),
    ///msg = "[player name] message"
    PledgeChat(String),
    PartyChat(String),
    ///name message
    WhisperChat(String, String),
}
