use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::packet::{ServerPacket, ServerPacketSender};

use crate::{client_message::ClientMessage, player::Player, server::ClientError, server_message::ServerMessage};

/// Represents the world for a server
pub struct World {
    /// The users logged into the world
    users: Arc<Mutex<HashMap<u32, String>>>,
    /// The id generator for users
    client_ids: Arc<Mutex<crate::ClientList>>,
    /// Used to broadcast server messages to the entire server
    pub global_tx: tokio::sync::broadcast::Sender<crate::ServerMessage>,
    /// The connection to the database
    mysql: mysql_async::Pool,
}

impl World {
    /// Construct a new server world
    pub fn new(global_tx: tokio::sync::broadcast::Sender<crate::ServerMessage>, mysql: mysql_async::Pool,) -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
            global_tx,
            mysql,
        }
    }

    /// Get a connection to the database
    pub async fn get_mysql_conn(&self) -> Result<mysql_async::Conn, mysql_async::Error> {
        self.mysql.get_conn().await
    }

    /// Register a new user
    pub fn register_user(&self) -> u32 {
        let mut c = self.client_ids.lock().unwrap();
        c.new_entry()
    }

    pub fn get_broadcast_rx(&self) -> tokio::sync::broadcast::Receiver<crate::ServerMessage> {
        self.global_tx.subscribe()
    }

    /// Process a single message for the server.
    pub async fn handle_server_message(&self, p: ServerMessage, packet_writer: &mut ServerPacketSender) -> Result<u8, ClientError> {
        match p {
            ServerMessage::Disconnect => {
                packet_writer
                    .send_packet(ServerPacket::Disconnect.build())
                    .await?;
            }
            ServerMessage::SystemMessage(m) => {
                packet_writer
                    .send_packet(ServerPacket::SystemMessage(m).build())
                    .await?;
            }
            ServerMessage::NpcShout(m) => {
                packet_writer
                    .send_packet(ServerPacket::NpcShout(m).build())
                    .await?;
            }
            ServerMessage::RegularChat { id, msg } => {
                packet_writer
                    .send_packet(ServerPacket::RegularChat { id: id, msg: msg }.build())
                    .await?;
            }
            ServerMessage::WhisperChat(name, msg) => {
                packet_writer
                    .send_packet(
                        ServerPacket::WhisperChat {
                            name: name,
                            msg: msg,
                        }
                        .build(),
                    )
                    .await?;
            }
            ServerMessage::YellChat { id, msg, x, y } => {
                packet_writer
                    .send_packet(
                        ServerPacket::YellChat {
                            id: id,
                            msg: msg,
                            x: x,
                            y: y,
                        }
                        .build(),
                    )
                    .await?;
            }
            ServerMessage::GlobalChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::GlobalChat(m).build())
                    .await?;
            }
            ServerMessage::PledgeChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::PledgeChat(m).build())
                    .await?;
            }
            ServerMessage::PartyChat(m) => {
                packet_writer
                    .send_packet(ServerPacket::PartyChat(m).build())
                    .await?;
            }
            ServerMessage::CharacterCreateStatus(v) => match v {
                0 => {
                    packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(2).build())
                        .await?;
                }
                1 => {
                    packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(9).build())
                        .await?;
                }
                2 => {
                    packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(6).build())
                        .await?;
                }
                3 => {
                    packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(21).build())
                        .await?;
                }
                _ => {
                    log::info!("wrong char creation status");
                }
            },
            ServerMessage::NewCharacterDetails {
                name,
                pledge,
                class,
                gender,
                alignment,
                hp,
                mp,
                ac,
                level,
                strength,
                dexterity,
                constitution,
                wisdom,
                charisma,
                intelligence,
            } => {
                packet_writer
                    .send_packet(
                        ServerPacket::NewCharacterDetails {
                            name: name,
                            pledge: pledge,
                            class: class,
                            gender: gender,
                            alignment: alignment,
                            hp: hp,
                            mp: mp,
                            ac: ac,
                            level: level,
                            strength: strength,
                            dexterity: dexterity,
                            constitution: constitution,
                            wisdom: wisdom,
                            charisma: charisma,
                            intelligence: intelligence,
                        }
                        .build(),
                    )
                    .await?;
            }
        }
        Ok(0)
    }

    pub async fn send_message(&self, message: crate::client_message::ClientMessage, packet_writer: &mut ServerPacketSender) -> Result<(), crate::server::ClientError> {
        match message {
            ClientMessage::LoggedIn(id, account) => {
                let mut u = self.users.lock().unwrap();
                u.insert(id, account);
            }
            ClientMessage::NewCharacter {
                id,
                name,
                class,
                gender,
                strength,
                dexterity,
                constitution,
                wisdom,
                charisma,
                intelligence,
            } => {
                let a = {
                    let u = self.users.lock().unwrap();
                    u.get(&id).map(|e| e.to_owned())
                };
                if let Some(account) = a {
                    log::info!("{} wants to make a new character {}", account, name);
                    //TODO ensure player name does not already exist
                    //TODO validate that all stats are legitimately possible
                    //TODO validate count of characters for account
    
                    if !Player::valid_name(name.clone()) {
                        packet_writer.send_packet(ServerPacket::CharacterCreationStatus(1).build()).await?;
                    } else {
                        packet_writer.send_packet(ServerPacket::CharacterCreationStatus(0).build()).await?;
                        //TODO: populate the correct details
                        packet_writer.send_packet(ServerPacket::NewCharacterDetails {
                            name: name.clone(),
                            pledge: "".to_string(),
                            class: class,
                            gender: gender,
                            alignment: 32764,
                            hp: 234,
                            mp: 456,
                            ac: 12,
                            level: 1,
                            strength: strength,
                            dexterity: dexterity,
                            constitution: constitution,
                            wisdom: wisdom,
                            charisma: charisma,
                            intelligence: intelligence,
                        }.build()).await?;
                    }
                }
            }
            ClientMessage::DeleteCharacter { id, name } => {
                log::info!("{} wants to delete {}", id, name);
            }
            ClientMessage::RegularChat { id: _, msg } => {
                //TODO limit based on distance and map
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.global_tx.send(ServerMessage::RegularChat { id: 0, msg: amsg });
            }
            ClientMessage::YellChat { id: _, msg, x, y } => {
                //TODO limit based on distance and map
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.global_tx.send(ServerMessage::YellChat {
                    id: 0,
                    msg: amsg,
                    x,
                    y,
                });
            }
            ClientMessage::GlobalChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.global_tx.send(ServerMessage::GlobalChat(amsg));
            }
            ClientMessage::PledgeChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.global_tx.send(ServerMessage::PledgeChat(amsg));
            }
            ClientMessage::PartyChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.global_tx.send(ServerMessage::PartyChat(amsg));
            }
            ClientMessage::WhisperChat(_id, _person, msg) => {
                let _ = self.global_tx.send(ServerMessage::WhisperChat("unknown".to_string(), msg));
            }
        }
        Ok(())
    }

    /// Unregister a user
    pub fn unregister_user(&self, uid: u32) {
        let mut c = self.client_ids.lock().unwrap();
        c.remove_entry(uid);
        let mut d = self.users.lock().unwrap();
        d.remove(&uid);
    }

    /// Get the number of players currently in the world
    pub fn get_number_players(&self) -> u16 {
        let users = self.users.lock().unwrap();
        users.len() as u16
    }
}
