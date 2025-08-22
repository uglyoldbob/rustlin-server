//! Holds code for game clients

use crate::server::ClientError;
use crate::user::*;
use crate::world::{ObjectRef, WorldMessage, WorldResponse};
use common::packet::*;

use futures::FutureExt;
use rand::Rng;

/// A game client for a game server
pub struct Client {
    /// Used to send packets to the server
    packet_writer: ServerPacketSender,
    /// The id for the game client
    id: Option<u32>,
    /// The account for the client
    account: Option<UserAccount>,
    /// The possible characters for the client
    chars: Vec<crate::character::Character>,
    /// The player reference
    char_ref: Option<ObjectRef>,
    /// The world object sender
    world_sender: tokio::sync::mpsc::Sender<WorldMessage>,
    /// The remote peer
    peer: std::net::SocketAddr,
}

impl Drop for Client {
    fn drop(&mut self) {
        log::info!("Running sync drop on client");
    }
}

impl Client {
    /// Construct a new client
    pub fn new(
        packet_writer: ServerPacketSender,
        world_sender: tokio::sync::mpsc::Sender<WorldMessage>,
        peer: std::net::SocketAddr,
    ) -> Self {
        Self {
            packet_writer,
            account: None,
            chars: Vec::new(),
            char_ref: None,
            world_sender,
            id: None,
            peer,
        }
    }

    pub async fn end(&mut self) {
        log::info!("Running end on client");
        let _ = self.packet_writer.queue_packet(ServerPacket::Disconnect);
        self.packet_writer.send_all_current_packets(None).await;
        if let Some(id) = self.id {
            self.world_sender
                .send(WorldMessage {
                    data: crate::world::WorldMessageData::UnregisterClient(id),
                    peer: self.peer,
                    sender: Some(id),
                })
                .await;
        }
    }

    /// Send a packet to the client
    pub fn queue_packet(&mut self, data: ServerPacket) {
        self.packet_writer.queue_packet(data)
    }

    /// Delete the character with the specified name
    pub fn delete_char(
        &mut self,
        name: &str,
        mysql: &mut mysql::PooledConn,
    ) -> Result<(), mysql::Error> {
        let mut i = None;
        for (index, c) in self.chars.iter_mut().enumerate() {
            if c.name() == name {
                c.delete_char(mysql)?;
                i = Some(index);
                break;
            }
        }
        if let Some(i) = i {
            self.chars.remove(i);
        }
        Ok(())
    }

    /// find a character by name, returning the character index
    pub fn find_char(&self, name: &str) -> Option<usize> {
        for (i, c) in self.chars.iter().enumerate() {
            if c.name() == name {
                return Some(i);
            }
        }
        None
    }

    /// Process a single packet from the game client
    pub async fn process_packet(&mut self, p: Packet) -> Result<(), ClientError> {
        let c = p.convert();
        log::info!("Processing client packet {:?}", c);
        self.world_sender
            .send(WorldMessage {
                data: crate::world::WorldMessageData::ClientPacket(c),
                sender: self.id,
                peer: self.peer,
            })
            .await;
        Ok(())
    }

    /// The main event loop for a client in a server.
    pub async fn event_loop(
        mut self,
        reader: tokio::net::tcp::OwnedReadHalf,
        mut receiver: tokio::sync::mpsc::Receiver<WorldResponse>,
        sender: tokio::sync::mpsc::Sender<WorldResponse>,
        mut end_rx: tokio::sync::mpsc::Receiver<u32>,
    ) -> Result<u8, ClientError> {
        let encryption_key: u32 = rand::thread_rng().gen();
        self.packet_writer.set_future_encryption_key(encryption_key);
        let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

        self.world_sender
            .send(WorldMessage {
                data: crate::world::WorldMessageData::RegisterSender(sender),
                sender: self.id,
                peer: self.peer,
            })
            .await;
        loop {
            futures::select! {
                packet = packet_reader.read_packet().fuse() => {
                    let p = packet?;
                    log::info!("Processing a packet {:?}", p);
                    self.process_packet(p).await?;
                    self.packet_writer.send_all_current_packets(Some(&mut packet_reader)).await?;
                }
                msg = receiver.recv().fuse() => {
                    let p = msg.unwrap();
                    log::info!("Got a async packet to send to client: {:?}", p);
                    match p {
                        WorldResponse::ServerPacket(p) => {
                            self.packet_writer.queue_packet(p);
                            self.packet_writer.send_all_current_packets(Some(&mut packet_reader)).await?;
                        }
                        WorldResponse::NewClientId(id) => {
                            log::info!("Got a client id {}", id);
                            self.id = Some(id);
                            self.packet_writer.send_all_current_packets(Some(&mut packet_reader)).await?;
                        }
                    }

                }
                _ = end_rx.recv().fuse() => {
                    break;
                }
            }
        }
        Ok(0)
    }
}
