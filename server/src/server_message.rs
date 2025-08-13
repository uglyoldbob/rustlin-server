//! Messages that can be sent by or to the server

/// Messages that can be passed back to the top level of the server
pub enum ServerShutdownMessage {
    /// The server should shutdown
    Shutdown,
    /// The server should restart
    Restart,
}
