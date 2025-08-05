use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::packet::{ServerPacket, ServerPacketSender};
use mysql_async::prelude::Queryable;

use crate::{
    character::Character, client_message::ClientMessage, server::ClientError,
    server_message::ServerMessage,
};

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
    pub fn new(
        global_tx: tokio::sync::broadcast::Sender<crate::ServerMessage>,
        mysql: mysql_async::Pool,
    ) -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            client_ids: Arc::new(Mutex::new(crate::ClientList::new())),
            global_tx,
            mysql,
        }
    }

    /// Insert a character id into the world
    pub async fn insert_id(&self, id: u32, account: String) -> Result<(), ClientError> {
        let mut u = self.users.lock().unwrap();
        u.insert(id, account);
        Ok(())
    }

    pub async fn lookup_id(&self, id: u32) -> Option<String> {
        let u = self.users.lock().unwrap();
        u.get(&id).map(|e| e.to_owned())
    }

    /// Get a new object id as part of a transaction.
    /// This prevents atomicity problems where two threads can get the same new id, and try to insert the same id into the database.
    /// # Arguments:
    /// * t - The transaction object
    pub async fn get_new_id(
        t: &mut mysql_async::Transaction<'_>,
    ) -> Result<Option<u32>, mysql_async::Error> {
        use mysql_async::prelude::Queryable;
        let query = "select max(id)+1 as nextid from (select id from character_items union all select id from character_teleport union all select id from character_warehouse union all select id from character_elf_warehouse union all select objid as id from characters union all select clan_id as id from clan_data union all select id from clan_warehouse union all select objid as id from pets) t";
        let a: Vec<u32> = t.exec(query, ()).await?;
        let r = if let Some(a) = a.first() {
            Ok(Some(*a))
        } else {
            Ok(None)
        };
        r
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
    pub async fn handle_server_message(
        &self,
        p: ServerMessage,
        packet_writer: &mut ServerPacketSender,
    ) -> Result<u8, ClientError> {
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

    /// Save a new character into the database
    pub async fn save_new_character(&self, c: &mut Character) -> Result<(), ClientError> {
        let mut conn = self.get_mysql_conn().await?;
        c.save_new_to_db(&mut conn).await
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
