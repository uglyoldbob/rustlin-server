use crate::ServerMessage;

/// This enum represents the messages sent from a client
pub enum ClientMessage {
	Register(tokio::sync::mpsc::UnboundedSender<ServerMessage>),
	Unregister(u32),
	RegularChat{id: u32, msg: String},	///msg = "player name: message"
	YellChat{id: u32, msg: String, x: u16, y: u16}, ///msg = "<player name> message"
	GlobalChat(u32, String), ///msg = "[player name] message"
	PledgeChat(u32, String),
	PartyChat(u32, String),
	WhisperChat(u32, String,String),
}


