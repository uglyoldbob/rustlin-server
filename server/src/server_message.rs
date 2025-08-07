/// Represents messages sent by the server to player clients
#[derive(Clone, Debug)]
pub enum ServerMessage {
    Disconnect,
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
