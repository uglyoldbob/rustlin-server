//! Messages that can be sent from a client
//!
//! TODO: delete this module

/// This enum represents the messages sent from a client
pub enum ClientMessage {
    /// A regular chat level
    RegularChat {
        /// The id of the character that did the chat?
        id: u32,
        /// The message
        msg: String,
    },
    ///msg = "player name: message"
    YellChat {
        /// The id of the character that did the yelling?
        id: u32,
        /// The yell message
        msg: String,
        /// They x coordinate where yelling came from
        x: u16,
        /// The y coordinate where yelling came from
        y: u16,
    },
    ///msg = "<player name> message"
    GlobalChat(u32, String),
    ///msg = "[player name] message"
    PledgeChat(u32, String),
    /// party level chat
    PartyChat(u32, String),
    ///id, name message
    WhisperChat(u32, String, String),
    ///Id and account name that was successfully logged in
    LoggedIn(u32, String),
    /// Create a new character
    NewCharacter {
        /// The character id?
        id: u32,
        /// The character name
        name: String,
        /// The class of the character
        class: u8,
        /// gender
        gender: u8,
        /// strength
        strength: u8,
        /// dexterity
        dexterity: u8,
        /// constitution
        constitution: u8,
        /// wisdom
        wisdom: u8,
        /// charisma
        charisma: u8,
        /// intellgience
        intelligence: u8,
    },
    /// The client wishes to delete a character
    DeleteCharacter {
        /// The id of the character to delete?
        id: u32,
        /// The name of the character to delete
        name: String,
    },
}
