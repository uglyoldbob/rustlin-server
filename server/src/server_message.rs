use crate::{character::Location, world::WorldObjectId};

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
    /// Add an object to the players known object list
    AddObject {
        /// The id of the object
        id: WorldObjectId,
        /// The location of the object
        location: Location,
    },
    /// Remove an object from the players known object list
    RemoveObject {
        /// The id of the object
        id: WorldObjectId,
    },
}

/// Messages that can be passed back to the top level of the server
pub enum ServerShutdownMessage {
    Shutdown,
    Restart,
}