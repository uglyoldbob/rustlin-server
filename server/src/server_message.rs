//! Messages that can be sent by or to the server
//!
//! TODO: delete this module

use crate::{character::Location, world::WorldObjectId};

/// Represents messages sent by the server to player clients
#[derive(Clone, Debug)]
pub enum ServerMessage {
    /// Disconnect the user
    Disconnect,
    /// regular chat
    RegularChat {
        /// The id of the chatting player?
        id: u32,
        /// The message being chatted
        msg: String,
    },
    ///yell level chat, msg = "player name: message"
    YellChat {
        /// The id of the yelling party?
        id: u32,
        /// The yell message
        msg: String,
        /// x coordinate where the yelling came from
        x: u16,
        /// y coordinate where the yelling came from
        y: u16,
    },
    ///global level chat, msg = "<player name> message"
    GlobalChat(String),
    ///pledge level chat, msg = "[player name] message"
    PledgeChat(String),
    /// Party level chat
    PartyChat(String),
    ///whisper to another player, name message
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
    /// The server should shutdown
    Shutdown,
    /// The server should restart
    Restart,
}
