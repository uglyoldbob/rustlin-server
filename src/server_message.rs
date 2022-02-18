/// Represents messages sent by the server to player clients
#[derive(Clone)]
pub enum ServerMessage {
    AssignId(u32),
}


