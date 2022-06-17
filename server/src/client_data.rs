use crate::client_message::*;
use crate::server_message::*;

pub struct ClientData {
    global_tx: tokio::sync::broadcast::Sender<ServerMessage>,
    pub server_tx: tokio::sync::mpsc::Sender<ClientMessage>,
    pub mysql: mysql_async::Pool,
}

impl Clone for ClientData {
    fn clone(&self) -> Self {
        Self {
            global_tx: self.global_tx.clone(),
            server_tx: self.server_tx.clone(),
            mysql: self.mysql.clone(),
        }
    }
}

impl ClientData {
    pub fn new(
        gtx: tokio::sync::broadcast::Sender<ServerMessage>,
        stx: tokio::sync::mpsc::Sender<ClientMessage>,
        m: mysql_async::Pool,
    ) -> ClientData {
        ClientData {
            global_tx: gtx,
            server_tx: stx,
            mysql: m,
        }
    }

    pub fn get_broadcast_rx(&self) -> tokio::sync::broadcast::Receiver<ServerMessage> {
        self.global_tx.subscribe()
    }

    pub async fn get_mysql(&self) -> Result<mysql_async::Conn, mysql_async::Error> {
        self.mysql.get_conn().await
    }
}
