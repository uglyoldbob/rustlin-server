use crate::ServerMessage;

/// This enum represents the messages sent from a client
pub enum ClientMessage {
	Register(tokio::sync::mpsc::UnboundedSender<ServerMessage>),
	Unregister(u32),
}


