//! Code for the main point of the server. This handles the main game portion of the server.

use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub mod client;
use crate::server::client::*;
use crate::world::WorldMessage;

use common::packet::*;

/// The errors that can occur when dealing with a game client
#[derive(Debug)]
pub enum ClientError {
    /// Some sort of packet error occurred
    PacketError(common::packet::PacketError),
    /// An io error occurred
    IoError(std::io::Error),
    /// A mysql error occurred
    MysqlError(mysql::Error),
    /// The user selected an invalid character
    InvalidCharSelection,
}

impl From<PacketError> for ClientError {
    fn from(p: PacketError) -> ClientError {
        ClientError::PacketError(p)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(a: std::io::Error) -> ClientError {
        ClientError::IoError(a)
    }
}

impl From<mysql::Error> for ClientError {
    fn from(a: mysql::Error) -> ClientError {
        ClientError::MysqlError(a)
    }
}

/// Process a single client for the world in the server
async fn process_client(
    socket: tokio::net::TcpStream,
    world_sender: tokio::sync::mpsc::Sender<crate::world::WorldMessage>,
    end_rx: tokio::sync::mpsc::Receiver<u32>,
) -> Result<u8, ClientError> {
    log::info!("Processing a client");
    let (reader, writer) = socket.into_split();
    let packet_writer = ServerPacketSender::new(writer);

    let (t_s, t_r) = tokio::sync::mpsc::channel(100);
    let peer = reader.peer_addr()?;
    let mut c = Client::new(packet_writer, world_sender, peer);
    let _ = c.event_loop(reader, t_r, t_s, end_rx).await;
    c.end().await;
    Ok(0)
}

/// The main struct for the game server
struct GameServer {
    /// Used to accept new connections from game clients
    listener: TcpListener,
    /// The server configuration
    config: std::sync::Arc<crate::ServerConfiguration>,
    /// Used to receive a message to end the server
    update_rx: tokio::sync::oneshot::Receiver<u32>,
    /// kill triggers
    kill: Arc<tokio::sync::Mutex<HashMap<SocketAddr, tokio::sync::mpsc::Sender<u32>>>>,
    /// task list of all clients
    clients: Option<tokio::task::JoinSet<()>>,
}

impl Drop for GameServer {
    fn drop(&mut self) {}
}

impl GameServer {
    /// End the game server gracefully
    pub async fn end(&mut self) {
        {
            let k = self.kill.lock().await;
            for (addr, k) in k.iter() {
                log::info!("Sending kill to {:?}", addr);
                let _ = k.send(0).await;
            }
        }
        log::info!("Waiting for all clients to finish");
        if let Some(t) = self.clients.take() {
            t.join_all().await;
        }
        log::info!("Ending the server thread!");
    }

    /// Run the server
    async fn run(&mut self, sender: tokio::sync::mpsc::Sender<WorldMessage>) -> Result<(), u32> {
        loop {
            tokio::select! {
                Ok((socket, addr)) = self.listener.accept() => {
                    log::info!("Received a client from {}", addr);
                    let sender2 = sender.clone();
                    let (kill_s, kill_r) = tokio::sync::mpsc::channel(5);
                    let kills2 = self.kill.clone();
                    if let Some(c) = &mut self.clients {
                        c.spawn(async move {
                            {
                                let mut k = kills2.lock().await;
                                k.insert(addr, kill_s);
                            }
                            if let Err(e) = process_client(socket, sender2, kill_r).await {
                                log::warn!("Client {} errored {:?}", addr, e);
                            }
                            {
                                let mut k = kills2.lock().await;
                                k.remove(&addr);
                            }
                            log::info!("Exiting client task");
                        });
                    }
                }
                Ok(a) = (&mut self.update_rx) => {
                    log::error!("Received a message {a} to shut down");
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Start the game
pub async fn setup_game_server(
    tasks: &mut tokio::task::JoinSet<Result<(), u32>>,
    config: &crate::ServerConfiguration,
    sender: tokio::sync::mpsc::Sender<WorldMessage>,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    log::info!("server: Starting the game server");
    let (update_tx, update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    let config = std::sync::Arc::new(config.clone());

    let mut server = GameServer {
        listener: update_listener,
        config,
        update_rx,
        clients: Some(tokio::task::JoinSet::new()),
        kill: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    };
    tasks.spawn(async move {
        server.run(sender).await;
        server.end().await;
        Ok(())
    });

    Ok(update_tx)
}
