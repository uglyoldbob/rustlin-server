//! Holds code for game clients

use crate::client_message::ClientMessage;
use crate::server::ClientError;
use crate::server_message::ServerMessage;
use crate::user::{self, *};
use common::packet::*;

use futures::FutureExt;
use rand::Rng;

/// A game client for a game server
pub struct Client {
    /// Used to send packets to the server
    packet_writer: ServerPacketSender,
    /// The id for the game client
    id: u32,
    /// The account for the client
    account: Option<UserAccount>,
    /// The possible characters for the client
    chars: Vec<crate::character::Character>,
    /// The world object
    world: std::sync::Arc<crate::world::World>,
}

impl Drop for Client {
    fn drop(&mut self) {
        //TODO send disconnect packet if applicable (needs async drop support first)
        self.world.unregister_user(self.id);
    }
}

impl Client {
    /// Construct a new client
    pub fn new(
        packet_writer: ServerPacketSender,
        world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            packet_writer,
            account: None,
            chars: Vec::new(),
            id: world.register_user(),
            world,
        }
    }

    pub async fn send_message(
        &mut self,
        message: crate::client_message::ClientMessage,
    ) -> Result<(), crate::server::ClientError> {
        match message {
            ClientMessage::LoggedIn(id, account) => {
                self.world.insert_id(id, account).await?;
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
                let a = self.world.lookup_id(id).await;
                if let Some(account) = a {
                    log::info!("{} wants to make a new character {}", account, name);
                    //TODO validate count of characters for account
                    //TODO validate that all stats are legitimately possible

                    if let Some(mut c) = crate::character::Character::new(
                        account,
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
                    ) {
                        self.world.save_new_character(&mut c).await?;
                        self.send_packet(ServerPacket::CharacterCreationStatus(0).build())
                            .await?;
                        self.send_packet(c.get_new_char_details_packet().build())
                            .await?;
                    } else {
                        self.send_packet(ServerPacket::CharacterCreationStatus(1).build())
                            .await?;
                    }
                }
            }
            ClientMessage::DeleteCharacter { id, name } => {
                log::info!("{} wants to delete {}", id, name);
                let mut m = self.world.get_mysql_conn().await?;
                self.delete_char(&name, &mut m).await?;
            }
            ClientMessage::RegularChat { id: _, msg } => {
                //TODO limit based on distance and map
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self
                    .world
                    .global_tx
                    .send(ServerMessage::RegularChat { id: 0, msg: amsg });
            }
            ClientMessage::YellChat { id: _, msg, x, y } => {
                //TODO limit based on distance and map
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.world.global_tx.send(ServerMessage::YellChat {
                    id: 0,
                    msg: amsg,
                    x,
                    y,
                });
            }
            ClientMessage::GlobalChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.world.global_tx.send(ServerMessage::GlobalChat(amsg));
            }
            ClientMessage::PledgeChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.world.global_tx.send(ServerMessage::PledgeChat(amsg));
            }
            ClientMessage::PartyChat(_id, msg) => {
                let amsg = format!("[{}] {}", "unknown", msg);
                let _ = self.world.global_tx.send(ServerMessage::PartyChat(amsg));
            }
            ClientMessage::WhisperChat(_id, _person, msg) => {
                let _ = self
                    .world
                    .global_tx
                    .send(ServerMessage::WhisperChat("unknown".to_string(), msg));
            }
        }
        Ok(())
    }

    /// Send a packet to the client
    pub async fn send_packet(&mut self, data: Packet) -> Result<(), PacketError> {
        self.packet_writer.send_packet(data).await
    }

    /// Delete the character with the specified name
    pub async fn delete_char(
        &mut self,
        name: &str,
        mysql: &mut mysql_async::Conn,
    ) -> Result<(), mysql_async::Error> {
        let mut i = None;
        for (index, c) in self.chars.iter_mut().enumerate() {
            if c.name() == name {
                c.delete_char(mysql).await?;
                i = Some(index);
                break;
            }
        }
        if let Some(i) = i {
            self.chars.remove(i);
        }
        Ok(())
    }

    /// Performs packet testing
    pub async fn test1(&mut self) -> Result<(), ClientError> {
        for i in 1..=16 {
            self.packet_writer
                .send_packet(
                    ServerPacket::Inventory {
                        id: 1,
                        i_type: 1,
                        n_use: 1,
                        icon: 1,
                        blessing: common::packet::ItemBlessing::Normal,
                        count: 1,
                        identified: 0,
                        description: " $1".to_string(),
                        ed: vec![23, i, i, 0, 0, 0],
                    }
                    .build(),
                )
                .await?;
        }
        for j in 0..12 {
            for i in 0..12 {
                self.packet_writer
                    .send_packet(
                        ServerPacket::PutObject {
                            x: 33435 + i,
                            y: 32816 - 2 * j,
                            id: 2 + 12 * j as u32 + i as u32,
                            icon: 29,
                            status: 0,
                            direction: 1,
                            light: 7,
                            speed: 50,
                            xp: 1235,
                            alignment: 2767,
                            name: "steve".to_string(),
                            title: "".to_string(),
                            status2: 0,
                            pledgeid: 0,
                            pledgename: "".to_string(),
                            owner_name: "".to_string(),
                            v1: 0,
                            hp_bar: 12,
                            v2: 0,
                            level: 0,
                        }
                        .build(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    /// Performs packet testing
    pub async fn test2(&mut self) -> Result<(), ClientError> {
        self.packet_writer
            .send_packet(
                ServerPacket::CloneObject {
                    id: 2,
                    speed: 255,
                    poly_id: 2001,
                    alignment: -32767,
                    poly_action: 0,
                    title: "Evil dragon 3".to_string(),
                }
                .build(),
            )
            .await?;
        Ok(())
    }

    async fn after_news(&mut self) -> Result<(), ClientError> {
        let response = ServerPacket::NumberCharacters(self.chars.len() as u8, 8).build();
        self.packet_writer.send_packet(response).await?;

        for c in &self.chars {
            let response = c.get_details_packet().build();
            self.packet_writer.send_packet(response).await?;
        }
        Ok(())
    }

    async fn login_with_news(
        &mut self,
        config: &std::sync::Arc<crate::ServerConfiguration>,
        username: String,
    ) -> Result<(), ClientError> {
        self.packet_writer
            .send_packet(ServerPacket::LoginResult { code: 0 }.build())
            .await?;
        let news = config.get_news();
        if news.is_empty() {
            self.after_news().await?;
        } else {
            self.packet_writer
                .send_packet(ServerPacket::News(news).build())
                .await?;
        }
        self.send_message(ClientMessage::LoggedIn(self.id, username))
            .await?;
        if let Some(account) = &self.account {
            let mut conn = self.world.get_mysql_conn().await?;
            self.chars = account.retrieve_chars(&mut conn).await?;
            log::info!("Characters are {:?}", self.chars);
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
    pub async fn process_packet(
        &mut self,
        p: Packet,
        peer: std::net::SocketAddr,
        config: &std::sync::Arc<crate::ServerConfiguration>,
    ) -> Result<(), ClientError> {
        let c = p.convert();
        match c {
            ClientPacket::Ping(v) => {
                log::info!("The user pinged us {v}");
            }
            ClientPacket::Restart => {
                log::info!("Player restarts");
                self.packet_writer
                    .send_packet(ServerPacket::BackToCharacterSelect.build())
                    .await?;
            }
            ClientPacket::RemoveFriend(name) => {
                log::info!("User used the remove friend command with {name}");
            }
            ClientPacket::AddFriend(name) => {
                log::info!("User used the add friend command with {name}");
            }
            ClientPacket::WhoCommand(name) => {
                log::info!("User used the who command on {name}");
            }
            ClientPacket::CreateBookmark(n) => {
                log::info!("User wants to create a bookmark named {n}");
            }
            ClientPacket::Version(a, b, c, d) => {
                log::info!("version {} {} {} {}", a, b, c, d);
                let response: Packet = ServerPacket::ServerVersion {
                    id: 2,
                    version: 0x16009,
                    time: 3,
                    new_accounts: 1,
                    english: 1,
                    country: 0,
                }
                .build();
                self.packet_writer.send_packet(response).await?;
            }
            ClientPacket::Login(u, p, v1, v2, v3, v4, v5, v6, v7) => {
                log::info!(
                    "login attempt for {} {} {} {} {} {} {} {}",
                    &u,
                    v1,
                    v2,
                    v3,
                    v4,
                    v5,
                    v6,
                    v7
                );
                let mut mysql_conn = self.world.get_mysql_conn().await?;
                let user = get_user_details(u.clone(), &mut mysql_conn).await;
                match user {
                    Some(us) => {
                        log::info!("User {} exists: {:?}", u.clone(), us);
                        let password_success = us.check_login(&config.account_creation_salt, &p);
                        log::info!("User login check is {}", password_success);
                        if password_success {
                            self.account = Some(us);
                            self.login_with_news(config, u).await?;
                        } else {
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                                .await?;
                        }
                    }
                    None => {
                        log::info!("User {} does not exist!", u.clone());
                        if config.automatic_account_creation {
                            let newaccount = UserAccount::new(
                                u.clone(),
                                p,
                                peer.to_string(),
                                config.account_creation_salt.clone(),
                            );
                            let mut mysql = self.world.get_mysql_conn().await?;
                            newaccount.insert_into_db(&mut mysql).await;
                            self.account = Some(newaccount);
                            self.login_with_news(config, u).await?;
                        } else {
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                                .await?;
                        }
                    }
                }
            }
            ClientPacket::NewsDone => {
                //send number of characters the player has
                self.after_news().await?;
            }
            ClientPacket::NewCharacter {
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
                self.send_message(ClientMessage::NewCharacter {
                    id: self.id,
                    name,
                    class,
                    gender,
                    strength,
                    dexterity,
                    constitution,
                    wisdom,
                    charisma,
                    intelligence,
                })
                .await?;
            }
            ClientPacket::DeleteCharacter(n) => {
                self.send_message(ClientMessage::DeleteCharacter {
                    id: self.id,
                    name: n,
                })
                .await?;
                //TODO determine if character level is 30 or higher
                //TODO send DeleteCharacterWait if level is 30 or higher
                self.packet_writer
                    .send_packet(ServerPacket::DeleteCharacterOk.build())
                    .await?;
            }
            ClientPacket::CharacterSelect { name } => {
                log::info!("login with {}", name);
                let c = self
                    .find_char(&name)
                    .ok_or(ClientError::InvalidCharSelection)?;

                let mut response = ServerPacket::StartGame(0).build();
                self.packet_writer.send_packet(response).await?;
                let mut mysql = self.world.get_mysql_conn().await?;
                let mut c = self.chars[c].get_full_details(&mut mysql).await?;
                c.main_character();
                response = c.details_packet().build();
                self.packet_writer.send_packet(response).await?;

                self.packet_writer
                    .send_packet(ServerPacket::MapId(4, 0).build())
                    .await?;

                self.packet_writer
                    .send_packet(
                        ServerPacket::PutObject {
                            x: 33430,
                            y: 32815,
                            id: 1,
                            icon: 1,
                            status: 0,
                            direction: 0,
                            light: 5,
                            speed: 50,
                            xp: 1234,
                            alignment: 32767,
                            name: "testing".to_string(),
                            title: "i am groot".to_string(),
                            status2: 0,
                            pledgeid: 0,
                            pledgename: "avengers".to_string(),
                            owner_name: "".to_string(),
                            v1: 0,
                            hp_bar: 100,
                            v2: 0,
                            level: 0,
                        }
                        .build(),
                    )
                    .await?;

                self.packet_writer
                    .send_packet(ServerPacket::CharSpMrBonus { sp: 0, mr: 0 }.build())
                    .await?;

                self.packet_writer
                    .send_packet(ServerPacket::Weather(0).build())
                    .await?;

                //TODO send owncharstatus packet
                self.test1().await?;
            }
            ClientPacket::KeepAlive => {}
            ClientPacket::GameInitDone => {}
            ClientPacket::WindowActivate(v2) => {
                log::info!("Client window activate {}", v2);
            }
            ClientPacket::Save => {}
            ClientPacket::Move { x, y, heading } => {
                log::info!("moving to {} {} {}", x, y, heading);
            }
            ClientPacket::ChangeDirection(d) => {
                log::info!("change direction to {}", d);
            }
            ClientPacket::Chat(m) => {
                self.send_message(ClientMessage::RegularChat {
                    id: self.id,
                    msg: m,
                })
                .await?;
            }
            ClientPacket::YellChat(m) => {
                //TODO put in the correct coordinates for yelling
                self.send_message(ClientMessage::YellChat {
                    id: self.id,
                    msg: m,
                    x: 32768,
                    y: 32768,
                })
                .await?;
            }
            ClientPacket::PartyChat(m) => {
                self.send_message(ClientMessage::PartyChat(self.id, m))
                    .await?;
            }
            ClientPacket::PledgeChat(m) => {
                self.send_message(ClientMessage::PledgeChat(self.id, m))
                    .await?;
            }
            ClientPacket::WhisperChat(n, m) => {
                self.send_message(ClientMessage::WhisperChat(self.id, n, m))
                    .await?;
            }
            ClientPacket::GlobalChat(m) => {
                self.send_message(ClientMessage::GlobalChat(self.id, m))
                    .await?;
            }
            ClientPacket::CommandChat(m) => {
                log::info!("command chat {}", m);
                let mut words = m.split_whitespace();
                let first_word = words.next();
                if let Some(m) = first_word {
                    match m {
                        "asdf" => {
                            log::info!("A command called asdf");
                        }
                        "quit" => {
                            self.packet_writer
                                .send_packet(ServerPacket::Disconnect.build())
                                .await?;
                        }
                        "test" => {
                            self.test2().await?;
                        }
                        "chat" => {
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::SystemMessage(
                                        "This is a test of the system message".to_string(),
                                    )
                                    .build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::NpcShout("NPC Shout test".to_string()).build(),
                                )
                                .await?;

                            self.packet_writer
                                .send_packet(
                                    ServerPacket::RegularChat {
                                        id: 0,
                                        msg: "regular chat".to_string(),
                                    }
                                    .build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::YellChat {
                                        id: 0,
                                        msg: "yelling".to_string(),
                                        x: 32768,
                                        y: 32768,
                                    }
                                    .build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::GlobalChat("global chat".to_string()).build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::PledgeChat("pledge chat".to_string()).build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::PartyChat("party chat".to_string()).build(),
                                )
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::WhisperChat {
                                        name: "test".to_string(),
                                        msg: "whisper message".to_string(),
                                    }
                                    .build(),
                                )
                                .await?;
                        }
                        _ => {
                            log::info!("An unknown command {}", m);
                        }
                    }
                }
            }
            ClientPacket::SpecialCommandChat(m) => {
                log::info!("special command chat {}", m);
            }
            ClientPacket::ChangePassword {
                account,
                oldpass,
                newpass,
            } => {
                let mut mysql = self.world.get_mysql_conn().await?;
                let user = get_user_details(account.clone(), &mut mysql).await;
                match user {
                    Some(us) => {
                        log::info!("User {} exists: {:?}", account, us);
                        let password_success =
                            us.check_login(&config.account_creation_salt, &oldpass);
                        log::info!("User login check is {}", password_success);
                        if password_success {
                            log::info!("User wants to change password and entered correct details");
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 0x30 }.build())
                                .await?;
                        } else {
                            let mut p = Packet::new();
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                                .await?;
                        }
                    }
                    _ => {
                        let mut p = Packet::new();
                        self.packet_writer
                            .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                            .await?;
                    }
                }
            }
            ClientPacket::Unknown(d) => {
                log::info!("received unknown packet {:x?}", d);
            }
        }
        Ok(())
    }

    /// The main event loop for a client in a server.
    pub async fn event_loop(
        mut self,
        reader: tokio::net::tcp::OwnedReadHalf,
        mut brd_rx: tokio::sync::broadcast::Receiver<ServerMessage>,
        mut rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
        config: &std::sync::Arc<crate::ServerConfiguration>,
    ) -> Result<u8, ClientError> {
        let encryption_key: u32 = rand::thread_rng().gen();
        let peer = reader.peer_addr()?;
        let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

        let mut key_packet = Packet::new();
        key_packet.add_u8(65).add_u32(encryption_key);
        self.packet_writer.send_packet(key_packet).await?;
        self.packet_writer
            .set_encryption_key(packet_reader.get_key());
        loop {
            futures::select! {
                packet = packet_reader.read_packet().fuse() => {
                    let p = packet?;
                    self.process_packet(p, peer, config).await?;
                }
                msg = brd_rx.recv().fuse() => {
                    let p = msg.unwrap();
                    self.world.handle_server_message(p, &mut self.packet_writer).await?;
                }
                msg = rx.recv().fuse() => {
                    match msg {
                        None => {}
                        Some(p) => {self.world.handle_server_message(p, &mut self.packet_writer).await?;}
                    }
                }
            }
        }
        Ok(0)
    }
}
