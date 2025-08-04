use futures::FutureExt;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use tokio::net::TcpListener;

use crate::client_data::*;
use crate::ClientMessage;
use crate::ServerMessage;

mod client;
use crate::server::client::*;

use common::packet::*;

/// The errors that can occur when dealing with a game client
#[derive(Debug)]
pub enum ClientError {
    /// Some sort of packet error occurred
    PacketError(common::packet::PacketError),
    /// An io error occurred
    IoError(std::io::Error),
    /// Error receiving a broadcast
    ErrorReceivingBroadcast(tokio::sync::broadcast::error::RecvError),
    /// Error sending a ClientMessage
    ErrorSendingClientMessage(tokio::sync::mpsc::error::SendError<ClientMessage>),
    /// A mysql error occurred
    MysqlError(mysql_async::Error),
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

impl From<tokio::sync::broadcast::error::RecvError> for ClientError {
    fn from(a: tokio::sync::broadcast::error::RecvError) -> ClientError {
        ClientError::ErrorReceivingBroadcast(a)
    }
}

impl From<tokio::sync::mpsc::error::SendError<ClientMessage>> for ClientError {
    fn from(a: tokio::sync::mpsc::error::SendError<ClientMessage>) -> ClientError {
        ClientError::ErrorSendingClientMessage(a)
    }
}

impl From<mysql_async::Error> for ClientError {
    fn from(a: mysql_async::Error) -> ClientError {
        ClientError::MysqlError(a)
    }
}

/// Process a single client for the world in the server
async fn process_client(
    socket: tokio::net::TcpStream,
    cd: ClientData,
    world: std::sync::Arc<crate::world::World>,
    config: std::sync::Arc<crate::ServerConfiguration>,
) -> Result<u8, ClientError> {
    let (reader, writer) = socket.into_split();
    let packet_writer = ServerPacketSender::new(writer);

    let brd_rx: tokio::sync::broadcast::Receiver<ServerMessage> = cd.get_broadcast_rx();
    let server_tx = &cd.server_tx;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let mysql = cd.get_mysql().await?;

    let c = Client::new(packet_writer, brd_rx, rx, server_tx, mysql, world.clone());

    if let Err(e) = c.event_loop(reader, &config).await {
        log::info!("test: Client errored: {:?}", e);
    }

    Ok(0)
}

struct GameServer {
    /// Used to accept new connections from game clients
    listener: TcpListener,
    /// USed to send and receive server messages
    cd: ClientData,
    /// A reference to the server world
    world: std::sync::Arc<crate::world::World>,
    /// The server configuration
    config: std::sync::Arc<crate::ServerConfiguration>,
    /// Used to receive a message to end the server
    update_rx: tokio::sync::oneshot::Receiver<u32>,
}

impl Drop for GameServer {
    fn drop(&mut self) {}
}

impl GameServer {
    /// Run the server
    async fn run(mut self) -> Result<(), u32> {
        let mut f = futures::stream::FuturesUnordered::new();
        loop {
            use futures::stream::StreamExt;
            tokio::select! {
                Ok(res) = self.listener.accept() => {
                    let (socket, addr) = res;
                    log::info!("Received a client from {}", addr);
                    let cd2 = self.cd.clone();
                    let world2 = self.world.clone();
                    let config3 = self.config.clone();
                    f.push(async move {
                        if let Err(e) = process_client(socket, cd2, world2, config3).await {
                            log::warn!("Client {} errored {:?}", addr, e);
                        }
                    });
                }
                Ok(Some(_)) = AssertUnwindSafe(f.next()).catch_unwind() => {
                    log::info!("User disconnected");
                }
                Ok(a) = (&mut self.update_rx) => {
                    log::error!("Received a message {a} to shut down");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    log::info!("Caught ctrl c message");
                    break;
                }
            }
        }
        let _ = self.cd.global_tx.send(ServerMessage::Disconnect);
        f.clear();
        log::info!("Ending the server thread!");
        Ok(())
    }
}

/// Start the game
pub async fn setup_game_server(
    cd: ClientData,
    tasks: &mut tokio::task::JoinSet<Result<(), u32>>,
    world: std::sync::Arc<crate::world::World>,
    config: &crate::ServerConfiguration,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    log::info!("server: Starting the game server");
    let (update_tx, update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    let config = std::sync::Arc::new(config.clone());

    let server = GameServer {
        listener: update_listener,
        cd,
        world,
        config,
        update_rx,
    };
    tasks.spawn(server.run());

    Ok(update_tx)
}
