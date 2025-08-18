//! Holds code for game clients

use crate::client_message::ClientMessage;
use crate::server::ClientError;
use crate::user::*;
use crate::world::object::ObjectTrait;
use crate::world::ObjectRef;
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
    /// The player reference
    char_ref: Option<ObjectRef>,
    /// The world object
    world: std::sync::Arc<crate::world::World>,
}

impl Drop for Client {
    fn drop(&mut self) {
        log::info!("Running sync drop on client");
    }
}

impl std::future::AsyncDrop for Client {
    async fn drop(mut self: std::pin::Pin<&mut Self>) {
        log::info!("Running async drop on client");
        let _ = self
            .packet_writer
            .queue_packet(ServerPacket::Disconnect);
        self.packet_writer.send_all_current_packets().await;
        self.world.unregister_user(self.id);
    }
}

/// Data to convey when the user wants to use an item
pub struct ItemUseData {
    /// the packet to process the item use command with
    pub p: Packet,
    /// The packets to send
    pub packets: Vec<ServerPacket>,
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
            char_ref: None,
            id: world.register_user(),
            world,
        }
    }

    /// Process a message from the client
    pub async fn process_client_message(
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
                        self.queue_packet(ServerPacket::CharacterCreationStatus(0));
                        self.queue_packet(c.get_new_char_details_packet());
                    } else {
                        self.queue_packet(ServerPacket::CharacterCreationStatus(1));
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
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerPacket::RegularChat { id: 0, msg: amsg })
                        })
                        .await;
                    if let Some(m) = m {
                        let mut p = Vec::new();
                        let _ = self
                            .world
                            .with_mut_objects_near_me_do(
                                &r,
                                30.0,
                                true,
                                &mut p,
                                move |o: &mut crate::world::object::Object, pw| {
                                    if let Some(sender) = o.sender() {
                                        pw.push((sender, m.clone()));
                                    }
                                    Ok::<(), String>(())
                                },
                            )
                            .await;
                        for (sender, p) in p {
                            let _ = sender.send(p).await;
                        }
                    }
                }
            }
            ClientMessage::YellChat { id: _, msg, x, y } => {
                //TODO limit based on distance and map
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerPacket::YellChat {
                                id: 0,
                                msg: amsg,
                                x,
                                y,
                            })
                        })
                        .await;
                    if let Some(m) = m {
                        let mut p = Vec::new();
                        let _ = self
                            .world
                            .with_mut_objects_near_me_do(
                                &r,
                                30.0,
                                true,
                                &mut p,
                                move |o: &mut crate::world::object::Object, pw| {
                                    if let Some(sender) = o.sender() {
                                        pw.push((sender, m.clone()));
                                    }
                                    Ok::<(), String>(())
                                },
                            )
                            .await;
                        for (sender, p) in p {
                            let _ = sender.send(p).await;
                        }
                    }
                }
            }
            ClientMessage::GlobalChat(_id, msg) => {
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerPacket::GlobalChat(amsg))
                        })
                        .await;
                    if let Some(m) = m {
                        self.world.send_global_chat(m).await;
                    }
                }
            }
            ClientMessage::PledgeChat(_id, msg) => {
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerPacket::PledgeChat(amsg))
                        })
                        .await;
                    if let Some(m) = m {
                        self.world.send_global_chat(m).await;
                    }
                }
            }
            ClientMessage::PartyChat(_id, msg) => {
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerPacket::PartyChat(amsg))
                        })
                        .await;
                    if let Some(m) = m {
                        self.world.send_global_chat(m).await;
                    }
                }
            }
            ClientMessage::WhisperChat(_id, person, msg) => {
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, move |fc, pw, _| {
                            Some(ServerPacket::WhisperChat {
                                name: fc.name.clone(),
                                msg: msg.clone(),
                            })
                        })
                        .await;
                    if let Some(m) = m {
                        if let Err(e) = self.world.send_packet_to(person.as_str(), m).await {
                            self.packet_writer
                                .queue_packet(ServerPacket::Message {
                                    ty: 73,
                                    msgs: vec![person],
                                });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Send a packet to the client
    pub fn queue_packet(&mut self, data: ServerPacket) {
        self.packet_writer.queue_packet(data)
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
    pub fn test1(&mut self) -> Result<(), ClientError> {
        self.packet_writer
            .queue_packet(ServerPacket::Message {
                ty: 74,
                msgs: vec!["stuff".to_string()],
            });
        Ok(())
    }

    /// Performs packet testing
    pub fn test2(&mut self) -> Result<(), ClientError> {
        self.packet_writer
            .queue_packet(ServerPacket::CloneObject {
                id: 2,
                speed: 255,
                poly_id: 2001,
                alignment: -32767,
                poly_action: 0,
                title: "Evil dragon 3".to_string(),
            });
        Ok(())
    }

    /// Send the client details that happens after the news (if there was any news at all)
    /// This still should be called even if there was no news.
    async fn after_news(&mut self) -> Result<(), ClientError> {
        let response = ServerPacket::NumberCharacters(self.chars.len() as u8, 8);
        self.packet_writer.queue_packet(response);

        for c in &self.chars {
            let response = c.get_details_packet();
            self.packet_writer.queue_packet(response);
        }
        Ok(())
    }

    /// Run the login process, delivering news if applicable
    async fn login_with_news(
        &mut self,
        config: &std::sync::Arc<crate::ServerConfiguration>,
        username: String,
    ) -> Result<(), ClientError> {
        self.packet_writer
            .queue_packet(ServerPacket::LoginResult { code: 0 });
        let news = config.get_news();
        if news.is_empty() {
            self.after_news().await?;
        } else {
            self.packet_writer
                .queue_packet(ServerPacket::News(news));
        }
        self.process_client_message(ClientMessage::LoggedIn(self.id, username))
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
        sender: &tokio::sync::mpsc::Sender<ServerPacket>,
    ) -> Result<(), ClientError> {
        let c = p.convert();
        let mut list = crate::world::map_info::SendsToAnotherObject::new();
        log::info!("Processing client packet {:?}", c);
        match c {
            ClientPacket::AttackObject { id, x, y } => {
                if let Some(r) = self.char_ref {
                    self.packet_writer
                        .queue_packet(ServerPacket::Attack {
                            attack_type: 3,
                            id: r.world_id().get_u32(),
                            id2: id,
                            impact: 1,
                            direction: 2,
                            effect: None,
                        });
                }
            }
            ClientPacket::UseItem { id, remainder } => {
                log::info!("User wants to use item {}: {:X?}", id, remainder);
                let p = common::packet::Packet::raw_packet(remainder);
                let mut p2 = ItemUseData {
                    p,
                    packets: Vec::new(),
                };
                if let Some(r) = self.char_ref {
                    self.world
                        .with_player_mut_do(r, &mut p2, move |fc, p2, map| {
                            // Can't use items whe you are dead
                            if fc.curr_hp() == 0 {
                                return None;
                            }
                            fc.use_item(&id, p2, map).ok()?;
                            Some(42)
                        })
                        .await;
                }
                for p in p2.packets {
                    self.packet_writer.queue_packet(p);
                }
            }
            ClientPacket::Ping(v) => {
                log::info!("The user pinged us {v}");
            }
            ClientPacket::Restart => {
                log::info!("Player restarts");
                self.world.remove_player(&mut self.char_ref).await;
                self.packet_writer
                    .queue_packet(ServerPacket::BackToCharacterSelect);
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
                let response = ServerPacket::ServerVersion {
                    id: 2,
                    version: 0x16009,
                    time: 3,
                    new_accounts: 1,
                    english: 1,
                    country: 0,
                };
                self.packet_writer.queue_packet(response);
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
                                .queue_packet(ServerPacket::LoginResult { code: 8 });
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
                                .queue_packet(ServerPacket::LoginResult { code: 8 });
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
                self.process_client_message(ClientMessage::NewCharacter {
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
                let c = self.find_char(&n);
                if let Some(c) = c {
                    let c = &self.chars[c];
                    if c.needs_delete_waiting() {
                        //TODO implement the actual delete in a scheduled async task
                        self.packet_writer
                            .queue_packet(ServerPacket::DeleteCharacterWait);
                    } else {
                        self.process_client_message(ClientMessage::DeleteCharacter {
                            id: self.id,
                            name: n,
                        })
                        .await?;
                        self.packet_writer
                            .queue_packet(ServerPacket::DeleteCharacterOk);
                    }
                }
            }
            ClientPacket::CharacterSelect { name } => {
                log::info!("login with {}", name);
                let c = self
                    .find_char(&name)
                    .ok_or(ClientError::InvalidCharSelection)?;

                self.packet_writer
                    .queue_packet(ServerPacket::StartGame(0));
                let mut mysql = self.world.get_mysql_conn().await?;
                let c = self.chars[c]
                    .get_partial_details(self.world.new_object_id(), &mut mysql)
                    .await?;
                self.char_ref = {
                    let c = {
                        let item_table = self.world.item_table.lock();
                        c.into_full(&item_table, sender.clone())
                    };
                    self.world.add_player(c, &mut self.packet_writer, &mut list)
                };
                self.test1()?;
                log::error!("Character select 5");
            }
            ClientPacket::KeepAlive => {}
            ClientPacket::GameInitDone => {}
            ClientPacket::WindowActivate(v2) => {
                log::info!("Client window activate {}", v2);
            }
            ClientPacket::Save => {}
            ClientPacket::MoveFrom { x, y, heading } => {
                let (x2, y2) = match heading {
                    0 => (x, y - 1),
                    1 => (x + 1, y - 1),
                    2 => (x + 1, y),
                    3 => (x + 1, y + 1),
                    4 => (x, y + 1),
                    5 => (x - 1, y + 1),
                    6 => (x - 1, y),
                    7 => (x - 1, y - 1),
                    _ => (x, y),
                };
                if let Some(r) = self.char_ref {
                    self.world
                        .move_object(
                            r,
                            crate::character::Location {
                                map: r.map(),
                                x: x2,
                                y: y2,
                                direction: heading,
                            },
                            Some(&mut self.packet_writer),
                            &mut list,
                        )
                        .await?;
                }
            }
            ClientPacket::ChangeDirection(d) => {
                if let Some(r) = self.char_ref {
                    self.world
                        .with_player_mut_do(r, &mut 42, move |fc, _, _| {
                            let l = fc.location_mut();
                            l.direction = d;
                            Some(42)
                        })
                        .await;
                }
            }
            ClientPacket::Chat(m) => {
                self.process_client_message(ClientMessage::RegularChat {
                    id: self.id,
                    msg: m,
                })
                .await?;
            }
            ClientPacket::YellChat(m) => {
                //TODO put in the correct coordinates for yelling
                self.process_client_message(ClientMessage::YellChat {
                    id: self.id,
                    msg: m,
                    x: 32768,
                    y: 32768,
                })
                .await?;
            }
            ClientPacket::PartyChat(m) => {
                self.process_client_message(ClientMessage::PartyChat(self.id, m))
                    .await?;
            }
            ClientPacket::PledgeChat(m) => {
                self.process_client_message(ClientMessage::PledgeChat(self.id, m))
                    .await?;
            }
            ClientPacket::WhisperChat(n, m) => {
                self.process_client_message(ClientMessage::WhisperChat(self.id, n, m))
                    .await?;
            }
            ClientPacket::GlobalChat(m) => {
                self.process_client_message(ClientMessage::GlobalChat(self.id, m))
                    .await?;
            }
            ClientPacket::CommandChat(m) => {
                let mut words = m.split_whitespace();
                let first_word = words.next();
                if let Some(m) = first_word {
                    /// TODO replace with a hashmap converting strings to functions
                    match m {
                        "asdf" => {
                            log::info!("A command called asdf");
                        }
                        "shutdown" => {
                            log::info!("A shutdown command was received");
                            if let Some(r) = &self.char_ref {
                                self.world.shutdown(r).await;
                            }
                        }
                        "restart" => {
                            log::info!("A restart command was received");
                            if let Some(r) = &self.char_ref {
                                self.world.restart(r).await;
                            }
                        }
                        "quit" => {
                            self.packet_writer.queue_packet(ServerPacket::Disconnect);
                        }
                        "test" => {
                            self.test2()?;
                        }
                        "chat" => {
                            self.packet_writer.queue_packet(ServerPacket::SystemMessage(
                                "This is a test of the system message".to_string(),
                            ));
                            self.packet_writer
                                .queue_packet(ServerPacket::NpcShout("NPC Shout test".to_string()));

                            self.packet_writer.queue_packet(ServerPacket::RegularChat {
                                id: 0,
                                msg: "regular chat".to_string(),
                            });
                            self.packet_writer.queue_packet(ServerPacket::YellChat {
                                id: 0,
                                msg: "yelling".to_string(),
                                x: 32768,
                                y: 32768,
                            });
                            self.packet_writer
                                .queue_packet(ServerPacket::GlobalChat("global chat".to_string()));
                            self.packet_writer
                                .queue_packet(ServerPacket::PledgeChat("pledge chat".to_string()));
                            self.packet_writer
                                .queue_packet(ServerPacket::PartyChat("party chat".to_string()));
                            self.packet_writer.queue_packet(ServerPacket::WhisperChat {
                                name: "test".to_string(),
                                msg: "whisper message".to_string(),
                            });
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
                newpass: _,
            } => {
                let mut mysql = self.world.get_mysql_conn().await?;
                let user = get_user_details(account.clone(), &mut mysql).await;
                match user {
                    Some(us) => {
                        let password_success =
                            us.check_login(&config.account_creation_salt, &oldpass);
                        if password_success {
                            log::info!("User wants to change password and entered correct details");
                            self.packet_writer
                                .queue_packet(ServerPacket::LoginResult { code: 0x30 });
                        } else {
                            self.packet_writer
                                .queue_packet(ServerPacket::LoginResult { code: 8 });
                        }
                    }
                    _ => {
                        self.packet_writer
                            .queue_packet(ServerPacket::LoginResult { code: 8 });
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
        mut receiver: tokio::sync::mpsc::Receiver<common::packet::ServerPacket>,
        sender: tokio::sync::mpsc::Sender<common::packet::ServerPacket>,
        config: &std::sync::Arc<crate::ServerConfiguration>,
        mut end_rx: tokio::sync::mpsc::Receiver<u32>,
    ) -> Result<u8, ClientError> {
        let encryption_key: u32 = rand::thread_rng().gen();
        let peer = reader.peer_addr()?;
        let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

        self.packet_writer
            .queue_packet(ServerPacket::EncryptionKey(encryption_key));
        self.packet_writer.send_all_current_packets().await?;
        self.packet_writer
            .set_encryption_key(packet_reader.get_key());
        loop {
            futures::select! {
                packet = packet_reader.read_packet().fuse() => {
                    let p = packet?;
                    self.process_packet(p, peer, config, &sender).await?;
                    self.packet_writer.send_all_current_packets().await?;
                }
                msg = receiver.recv().fuse() => {
                    let p = msg.unwrap();
                    log::info!("Got a async packet to send to client: {:?}", p);
                    self.packet_writer.queue_packet(p);
                    self.packet_writer.send_all_current_packets().await?;
                }
                _ = end_rx.recv().fuse() => {
                    break;
                }
            }
        }
        Ok(0)
    }
}
