use crate::ServerMessage;

/// This enum represents the messages sent from a client
pub enum ClientMessage {
    Register(tokio::sync::mpsc::UnboundedSender<ServerMessage>),
    Unregister(u32),
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
    GlobalChat(u32, String),
    ///msg = "[player name] message"
    PledgeChat(u32, String),
    PartyChat(u32, String),
    ///id, name message
    WhisperChat(u32, String, String),
	///Id and account name that was successfully logged in
	LoggedIn(u32, String),
	NewCharacter {
		id: u32,
		name: String,
		class: u8,
		gender: u8,
		strength: u8,
		dexterity: u8,
		constitution: u8,
		wisdom: u8,
		charisma: u8,
		intelligence: u8,
	},
	DeleteCharacter {
		id: u32,
		name: String },
}
