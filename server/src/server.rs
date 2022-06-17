use futures::FutureExt;
use std::error::Error;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use std::convert::TryInto;
use std::vec::Vec;

use rand::Rng;

use crate::client_data::*;
use crate::user::*;
use crate::ClientMessage;
use crate::ServerMessage;

mod client;
use crate::server::client::*;

use common::packet::*;

#[derive(Debug, Clone)]
pub struct ClientError;
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client error")
    }
}

impl From<PacketError> for ClientError {
    fn from(_: PacketError) -> ClientError {
        ClientError {}
    }
}

impl From<std::io::Error> for ClientError {
    fn from(_: std::io::Error) -> ClientError {
        ClientError {}
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for ClientError {
    fn from(_: tokio::sync::broadcast::error::RecvError) -> ClientError {
        ClientError {}
    }
}

impl From<tokio::sync::mpsc::error::SendError<ClientMessage>> for ClientError {
    fn from(_: tokio::sync::mpsc::error::SendError<ClientMessage>) -> ClientError {
        ClientError {}
    }
}

impl From<mysql_async::Error> for ClientError {
    fn from(_: mysql_async::Error) -> ClientError {
        ClientError {}
    }
}



async fn process_client(socket: tokio::net::TcpStream, cd: ClientData) -> Result<u8, ClientError> {
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
        match msg.unwrap() {
            ServerMessage::AssignId(i) => {
                println!("client: assigned id of {} to self", i);
                client_id = i;
                break;
            }
            _ => {}
        }
    }

    let c = Client::new(packet_writer,
        brd_rx,
        rx,
        &server_tx,
        client_id,
        mysql,);

    if let Err(_) = c.event_loop(reader).await
    {
        println!("test: Client errored");
    }

    server_tx.send(ClientMessage::Unregister(client_id)).await?;

    Ok(0)
}

pub async fn setup_game_server(
    cd: ClientData,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    println!("server: Starting the game server");
    let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = update_listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    println!("server: Received a client from {}", addr);
                    let cd2 = cd.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process_client(socket, cd2).await {
                            println!("server: Client {} errored {}", addr, e);
                        }
                    });
                }
                _ = (&mut update_rx) => {
                    println!("server: Received a message to shut down");
                    break;
                }
            }
        }
        println!("update: Ending the server thread!");
    });

    Ok(update_tx)
}
