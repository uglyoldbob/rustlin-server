#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! The server for the game.
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
    PacketError,
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
    fn from(_: PacketError) -> ClientError {
        ClientError::PacketError
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
) -> Result<u8, ClientError> {
    let (reader, writer) = socket.into_split();
    let packet_writer = ServerPacketSender::new(writer);

    let brd_rx: tokio::sync::broadcast::Receiver<ServerMessage> = cd.get_broadcast_rx();
    let server_tx = &cd.server_tx;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let _ = &server_tx.send(ClientMessage::Register(tx)).await?;

    let client_id: u32;

    let mysql = cd.get_mysql().await?;

    println!("client: Waiting to receive the id");
    loop {
        let msg = rx.recv().await;
        if let ServerMessage::AssignId(i) = msg.unwrap() {
            println!("client: assigned id of {} to self", i);
            client_id = i;
            break;
        }
    }

    let c = Client::new(packet_writer, brd_rx, rx, server_tx, client_id, mysql);

    if let Err(e) = c.event_loop(reader, world).await {
        println!("test: Client errored: {:?}", e);
    }

    server_tx.send(ClientMessage::Unregister(client_id)).await?;

    Ok(0)
}

/// Start the game
pub async fn setup_game_server(
    cd: ClientData,
    tasks: &mut tokio::task::JoinSet<Result<(), u32>>,
    world: std::sync::Arc<crate::world::World>,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    println!("server: Starting the game server");
    let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    tasks.spawn(async move {
        let mut f = futures::stream::FuturesUnordered::new();
        loop {
            use futures::stream::StreamExt;
            tokio::select! {
                Ok(res) = update_listener.accept() => {
                    let (socket, addr) = res;
                    println!("server: Received a client from {}", addr);
                    let cd2 = cd.clone();
                    let world2 = world.clone();
                    f.push(async move {
                        if let Err(e) = process_client(socket, cd2, world2).await {
                            println!("server: Client {} errored {:?}", addr, e);
                        }
                    });
                }
                Ok(Some(_)) = AssertUnwindSafe(f.next()).catch_unwind() => {}
                _ = (&mut update_rx) => {
                    println!("server: Received a message to shut down");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }
        println!("update: Ending the server thread!");
        Ok(())
    });

    Ok(update_tx)
}
