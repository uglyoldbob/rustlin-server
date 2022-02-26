use crate::client_message::*;
use crate::server_message::*;

pub struct ClientData {
    global_tx: tokio::sync::broadcast::Sender<ServerMessage>,
    pub server_tx: tokio::sync::mpsc::Sender<ClientMessage>,
}

impl Clone for ClientData {
    fn clone(&self) -> Self {
        Self {
            global_tx: self.global_tx.clone(),
            server_tx: self.server_tx.clone(),
        }
    }
}

impl ClientData {
    pub fn new(
        gtx: tokio::sync::broadcast::Sender<ServerMessage>,
        stx: tokio::sync::mpsc::Sender<ClientMessage>,
    ) -> ClientData {
        ClientData {
            global_tx: gtx,
            server_tx: stx,
        }
    }

    pub fn get_broadcast_rx(&self) -> tokio::sync::broadcast::Receiver<ServerMessage> {
        self.global_tx.subscribe()
    }
}
