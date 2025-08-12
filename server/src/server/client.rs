//! Holds code for game clients

use crate::client_message::ClientMessage;
use crate::server::ClientError;
use crate::server_message::ServerMessage;
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
            .send_packet(ServerPacket::Disconnect.build())
            .await;
        self.world.unregister_user(self.id);
    }
}

pub struct ItemUseData {
    pub p: Packet,
    pub packets: Vec<Packet>,
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
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerMessage::RegularChat { id: 0, msg: amsg })
                        })
                        .await;
                    if let Some(m) = m {
                        let _ = self
                            .world
                            .with_mut_objects_near_me_do(
                                &r,
                                30.0,
                                true,
                                &mut self.packet_writer,
                                async move |o: &mut crate::world::object::Object, pw| {
                                    if let Some(sender) = o.sender() {
                                        let _ = sender.send(m.clone()).await;
                                    }
                                    Ok::<(), String>(())
                                },
                            )
                            .await;
                    }
                }
            }
            ClientMessage::YellChat { id: _, msg, x, y } => {
                //TODO limit based on distance and map
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerMessage::YellChat {
                                id: 0,
                                msg: amsg,
                                x,
                                y,
                            })
                        })
                        .await;
                    if let Some(m) = m {
                        let _ = self
                            .world
                            .with_mut_objects_near_me_do(
                                &r,
                                60.0,
                                true,
                                &mut self.packet_writer,
                                async move |o: &mut crate::world::object::Object, pw| {
                                    if let Some(sender) = o.sender() {
                                        let _ = sender.send(m.clone()).await;
                                    }
                                    Ok::<(), String>(())
                                },
                            )
                            .await;
                    }
                }
            }
            ClientMessage::GlobalChat(_id, msg) => {
                if let Some(r) = self.char_ref {
                    let m = self
                        .world
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerMessage::GlobalChat(amsg))
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
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerMessage::PledgeChat(amsg))
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
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            let amsg = format!("[{}] {}", fc.name, msg);
                            Some(ServerMessage::PartyChat(amsg))
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
                        .with_player_ref_do(r, &mut self.packet_writer, async move |fc, pw, _| {
                            Some(ServerMessage::WhisperChat(fc.name.clone(), msg.clone()))
                        })
                        .await;
                    if let Some(m) = m {
                        if let Err(e) = self.world.send_whisper_to(person.as_str(), m).await {
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::Message {
                                        ty: 73,
                                        msgs: vec![person],
                                    }
                                    .build(),
                                )
                                .await?;
                        }
                    }
                }
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
        self.packet_writer
            .send_packet(
                ServerPacket::Message {
                    ty: 74,
                    msgs: vec!["stuff".to_string()],
                }
                .build(),
            )
            .await?;
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
        sender: &tokio::sync::mpsc::Sender<ServerMessage>,
    ) -> Result<(), ClientError> {
        let c = p.convert();
        match c {
            ClientPacket::UseItem { id, remainder } => {
                log::info!("User wants to use item {}: {:X?}", id, remainder);
                let p = common::packet::Packet::raw_packet(remainder);
                let mut p2 = ItemUseData {
                    p,
                    packets: Vec::new(),
                };
                if let Some(r) = self.char_ref {
                    self.world
                        .with_player_mut_do(r, &mut p2, async move |fc, p2, map| {
                            // Can't use items whe you are dead
                            if fc.curr_hp() == 0 {
                                return None;
                            }
                            fc.use_item(&id, p2, map).await.ok()?;
                            Some(42)
                        })
                        .await;
                }
                for p in p2.packets {
                    self.packet_writer.send_packet(p).await?;
                }
            }
            ClientPacket::Ping(v) => {
                log::info!("The user pinged us {v}");
            }
            ClientPacket::Restart => {
                log::info!("Player restarts");
                self.world.remove_player(&mut self.char_ref).await;
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
                let c = self.find_char(&n);
                if let Some(c) = c {
                    let c = &self.chars[c];
                    if c.needs_delete_waiting() {
                        //TODO implement the actual delete in a scheduled async task
                        self.packet_writer
                            .send_packet(ServerPacket::DeleteCharacterWait.build())
                            .await?;
                    } else {
                        self.send_message(ClientMessage::DeleteCharacter {
                            id: self.id,
                            name: n,
                        })
                        .await?;
                        self.packet_writer
                            .send_packet(ServerPacket::DeleteCharacterOk.build())
                            .await?;
                    }
                }
            }
            ClientPacket::CharacterSelect { name } => {
                log::info!("login with {}", name);
                let c = self
                    .find_char(&name)
                    .ok_or(ClientError::InvalidCharSelection)?;

                self.packet_writer
                    .send_packet(ServerPacket::StartGame(0).build())
                    .await?;
                let mut mysql = self.world.get_mysql_conn().await?;
                let c = self.chars[c]
                    .get_partial_details(self.world.new_object_id(), &mut mysql)
                    .await?;
                self.char_ref = {
                    let c = {
                        let item_table = self.world.item_table.lock().unwrap();
                        c.to_full(&item_table, sender.clone())
                    };
                    self.world.add_player(c, &mut self.packet_writer).await
                };

                if let Some(r) = self.char_ref {
                    self.world
                        .with_player_ref_do(r, &mut self.packet_writer, async |c, pw, _| {
                            pw.send_packet(c.details_packet().build()).await.ok()?;
                            pw.send_packet(c.get_map_packet().build()).await.ok()?;
                            pw.send_packet(c.get_object_packet().build()).await.ok()?;
                            c.send_all_items(pw).await.ok()?;
                            Some(42)
                        })
                        .await;
                }

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
                            &mut self.packet_writer,
                        )
                        .await?;
                }
            }
            ClientPacket::ChangeDirection(d) => {
                if let Some(r) = self.char_ref {
                    self.world
                        .with_player_mut_do(r, &mut 42, async move |fc, _, _| {
                            let l = fc.location_mut();
                            l.direction = d;
                            Some(42)
                        })
                        .await;
                }
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
                                .send_packet(ServerPacket::LoginResult { code: 0x30 }.build())
                                .await?;
                        } else {
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                                .await?;
                        }
                    }
                    _ => {
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

    /// Process a message received from the server
    async fn process_server_message(&mut self, p: ServerMessage) -> Result<(), ClientError> {
        match p {
            ServerMessage::AddObject { id, location } => {
                log::info!("Adding object {:?} at {:?}", id, location);
                self.world
                    .send_new_object(location, id, &mut self.packet_writer)
                    .await?;
            }
            ServerMessage::RemoveObject { id } => {
                self.packet_writer
                    .send_packet(ServerPacket::RemoveObject(id.into()).build())
                    .await?;
            }
            ServerMessage::Disconnect => {
                self.packet_writer
                    .send_packet(ServerPacket::Disconnect.build())
                    .await?;
            }
            ServerMessage::RegularChat { id, msg } => {
                self.packet_writer
                    .send_packet(ServerPacket::RegularChat { id, msg }.build())
                    .await?;
            }
            ServerMessage::YellChat { id, msg, x, y } => {
                self.packet_writer
                    .send_packet(ServerPacket::YellChat { id, msg, x, y }.build())
                    .await?;
            }
            ServerMessage::GlobalChat(m) => {
                self.packet_writer
                    .send_packet(ServerPacket::GlobalChat(m).build())
                    .await?;
            }
            ServerMessage::PledgeChat(m) => {
                self.packet_writer
                    .send_packet(ServerPacket::PledgeChat(m).build())
                    .await?;
            }
            ServerMessage::PartyChat(m) => {
                self.packet_writer
                    .send_packet(ServerPacket::PartyChat(m).build())
                    .await?;
            }
            ServerMessage::WhisperChat(u, m) => {
                self.packet_writer
                    .send_packet(ServerPacket::WhisperChat { name: u, msg: m }.build())
                    .await?;
            }
        }
        Ok(())
    }

    /// The main event loop for a client in a server.
    pub async fn event_loop(
        mut self,
        reader: tokio::net::tcp::OwnedReadHalf,
        mut receiver: tokio::sync::mpsc::Receiver<ServerMessage>,
        sender: tokio::sync::mpsc::Sender<ServerMessage>,
        config: &std::sync::Arc<crate::ServerConfiguration>,
        mut end_rx: tokio::sync::mpsc::Receiver<u32>,
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
                    self.process_packet(p, peer, config, &sender).await?;
                }
                msg = receiver.recv().fuse() => {
                    let p = msg.unwrap();
                    self.process_server_message(p).await?;
                }
                _ = end_rx.recv().fuse() => {
                    break;
                }
            }
        }
        Ok(0)
    }
}
