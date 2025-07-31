//! Holds code for game clients

use crate::client_message::ClientMessage;
use crate::server::ClientError;
use crate::server_message::ServerMessage;
use crate::user::*;
use common::packet::*;

use futures::FutureExt;
use rand::Rng;

/// A game client for a game server
pub struct Client<'a> {
    /// Used to send packets to the server
    packet_writer: ServerPacketSender,
    /// How the client receives broadcast messages from the server
    brd_rx: tokio::sync::broadcast::Receiver<ServerMessage>,
    /// How the client receives messages from the server
    rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
    /// How the client sends messages to the server
    server_tx: &'a tokio::sync::mpsc::Sender<ClientMessage>,
    /// The id for the game client
    id: u32,
    /// Connection to the mysql server
    mysql: mysql_async::Conn,
    /// The world object
    world: std::sync::Arc<crate::world::World>,
}

impl<'a> Drop for Client<'a> {
    fn drop(&mut self) {
        //TODO send disconnect packet if applicable
        self.world.unregister_user(self.id);
    }
}

impl<'a> Client<'a> {
    /// Construct a new client
    pub fn new(
        packet_writer: ServerPacketSender,
        brd_rx: tokio::sync::broadcast::Receiver<ServerMessage>,
        rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
        server_tx: &'a tokio::sync::mpsc::Sender<ClientMessage>,
        mysql: mysql_async::Conn,
        world: std::sync::Arc<crate::world::World>,
    ) -> Self {
        Self {
            packet_writer,
            brd_rx,
            rx,
            server_tx,
            id: world.register_user(),
            mysql,
            world,
        }
    }

    /// Process a single message for the server.
    /// TODO move this functionality to world?
    pub async fn handle_server_message(&mut self, p: ServerMessage) -> Result<u8, ClientError> {
        match p {
            ServerMessage::Disconnect => {
                self.packet_writer
                    .send_packet(ServerPacket::Disconnect.build())
                    .await?;
            }
            ServerMessage::SystemMessage(m) => {
                self.packet_writer
                    .send_packet(ServerPacket::SystemMessage(m).build())
                    .await?;
            }
            ServerMessage::NpcShout(m) => {
                self.packet_writer
                    .send_packet(ServerPacket::NpcShout(m).build())
                    .await?;
            }
            ServerMessage::RegularChat { id, msg } => {
                self.packet_writer
                    .send_packet(ServerPacket::RegularChat { id: id, msg: msg }.build())
                    .await?;
            }
            ServerMessage::WhisperChat(name, msg) => {
                self.packet_writer
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
                self.packet_writer
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
            ServerMessage::CharacterCreateStatus(v) => match v {
                0 => {
                    self.packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(2).build())
                        .await?;
                }
                1 => {
                    self.packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(9).build())
                        .await?;
                }
                2 => {
                    self.packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(6).build())
                        .await?;
                }
                3 => {
                    self.packet_writer
                        .send_packet(ServerPacket::CharacterCreationStatus(21).build())
                        .await?;
                }
                _ => {
                    println!("wrong char creation status");
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
                self.packet_writer
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

    /// Performs packet testing
    pub async fn test1(&mut self) -> Result<(), ClientError> {
        let mut p = ServerPacket::Inventory {
            id: 1,
            i_type: 1,
            n_use: 1,
            icon: 1,
            blessing: common::packet::ItemBlessing::Normal,
            count: 1,
            identified: 0,
            description: " $1".to_string(),
            ed_used: 1,
        }
        .build();
        p.add_vec(&vec![1]);
        self.packet_writer.send_packet(p).await?;
        Ok(())
    }

    /// Process a single packet from the game client
    pub async fn process_packet(
        &mut self,
        p: Packet,
        peer: std::net::SocketAddr,
    ) -> Result<(), ClientError> {
        let c = p.convert();
        match c {
            ClientPacket::Version(a, b, c, d) => {
                println!("client: version {} {} {} {}", a, b, c, d);
                let response: Packet = ServerPacket::ServerVersion {
                    id: 2,
                    version: 0x27e9,
                    time: 3,
                    new_accounts: 1,
                    english: 1,
                    country: 0,
                }
                .build();
                self.packet_writer.send_packet(response).await?;
            }
            ClientPacket::Login(u, p, v1, v2, v3, v4, v5, v6, v7) => {
                println!(
                    "client: login attempt for {} {} {} {} {} {} {} {}",
                    &u, v1, v2, v3, v4, v5, v6, v7
                );
                let user = get_user_details(u.clone(), &mut self.mysql).await;
                match user {
                    Some(us) => {
                        println!("User {} exists", u.clone());
                        us.print();
                        //TODO un-hardcode the salt for the password hashing
                        let password_success = us.check_login("lineage".to_string(), p);
                        println!(
                            "User pw test {}",
                            hash_password(
                                "testtest".to_string(),
                                "lineage".to_string(),
                                "password".to_string()
                            )
                        );
                        println!("User login check is {}", password_success);
                        if password_success {
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 0 }.build())
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::News("This is the news".to_string()).build(),
                                )
                                .await?;
                            let _ = &self
                                .server_tx
                                .send(ClientMessage::LoggedIn(self.id, u))
                                .await?;
                        } else {
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 8 }.build())
                                .await?;
                        }
                    }
                    None => {
                        println!("User {} does not exist!", u.clone());
                        //TODO actually determine if auto account creation is enabled
                        if true {
                            //TODO un-hardcode the salt for the password hashing
                            let newaccount = UserAccount::new(
                                u.clone(),
                                p,
                                peer.to_string(),
                                "lineage".to_string(),
                            );
                            newaccount.insert_into_db(&mut self.mysql).await;
                            self.packet_writer
                                .send_packet(ServerPacket::LoginResult { code: 0 }.build())
                                .await?;
                            self.packet_writer
                                .send_packet(
                                    ServerPacket::News("This is the news".to_string()).build(),
                                )
                                .await?;
                            let _ = &self
                                .server_tx
                                .send(ClientMessage::LoggedIn(self.id, u.clone()))
                                .await?;
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
                let mut response = ServerPacket::NumberCharacters(1, 8).build();
                self.packet_writer.send_packet(response).await?;

                for _ in 0..1 {
                    response = ServerPacket::LoginCharacterDetails {
                        name: "whatever".to_string(),
                        pledge: "whocares".to_string(),
                        ctype: 1,
                        gender: 2,
                        alignment: 32767,
                        hp: 1234,
                        mp: 95,
                        ac: -12,
                        level: 51,
                        strength: 12,
                        dexterity: 12,
                        constitution: 12,
                        wisdom: 12,
                        charisma: 12,
                        intelligence: 12,
                    }
                    .build();
                    self.packet_writer.send_packet(response).await?;
                }
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
                let _ = &self
                    .server_tx
                    .send(ClientMessage::NewCharacter {
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
                let _ = &self
                    .server_tx
                    .send(ClientMessage::DeleteCharacter {
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
                println!("client: login with {}", name);
                let mut response = ServerPacket::StartGame(0).build();
                self.packet_writer.send_packet(response).await?;

                response = ServerPacket::CharacterDetails {
                    id: 1,
                    level: 5,
                    xp: 1234,
                    strength: 12,
                    dexterity: 12,
                    constitution: 13,
                    wisdom: 14,
                    charisma: 15,
                    intelligence: 16,
                    curr_hp: 123,
                    max_hp: 985,
                    curr_mp: 34,
                    max_mp: 345,
                    time: 1,
                    ac: -13,
                    food: 1.0,
                    weight: 0.5,
                    alignment: 32675,
                    fire_resist: 0,
                    water_resist: 1,
                    wind_resist: 2,
                    earth_resist: 3,
                }
                .build();
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
                            unknown: "".to_string(),
                            v1: 0,
                            hp_bar: 100,
                            v2: 0,
                            v3: 0,
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
                println!("Client window activate {}", v2);
            }
            ClientPacket::Save => {}
            ClientPacket::Move { x, y, heading } => {
                println!("client: moving to {} {} {}", x, y, heading);
            }
            ClientPacket::ChangeDirection(d) => {
                println!("client: change direction to {}", d);
            }
            ClientPacket::Chat(m) => {
                let _ = &self
                    .server_tx
                    .send(ClientMessage::RegularChat {
                        id: self.id,
                        msg: m,
                    })
                    .await?;
            }
            ClientPacket::YellChat(m) => {
                //TODO put in the correct coordinates for yelling
                let _ = &self
                    .server_tx
                    .send(ClientMessage::YellChat {
                        id: self.id,
                        msg: m,
                        x: 32768,
                        y: 32768,
                    })
                    .await?;
            }
            ClientPacket::PartyChat(m) => {
                let _ = &self
                    .server_tx
                    .send(ClientMessage::PartyChat(self.id, m))
                    .await?;
            }
            ClientPacket::PledgeChat(m) => {
                let _ = &self
                    .server_tx
                    .send(ClientMessage::PledgeChat(self.id, m))
                    .await?;
            }
            ClientPacket::WhisperChat(n, m) => {
                let _ = &self
                    .server_tx
                    .send(ClientMessage::WhisperChat(self.id, n, m))
                    .await?;
            }
            ClientPacket::GlobalChat(m) => {
                let _ = &self
                    .server_tx
                    .send(ClientMessage::GlobalChat(self.id, m))
                    .await?;
            }
            ClientPacket::CommandChat(m) => {
                println!("client: command chat {}", m);
                let mut words = m.split_whitespace();
                let first_word = words.next();
                if let Some(m) = first_word {
                    match m {
                        "asdf" => {
                            println!("A command called asdf");
                        }
                        "quit" => {
                            self.packet_writer
                                .send_packet(ServerPacket::Disconnect.build())
                                .await?;
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
                            println!("An unknown command {}", m);
                        }
                    }
                }
            }
            ClientPacket::SpecialCommandChat(m) => {
                println!("client: special command chat {}", m);
            }
            ClientPacket::ChangePassword {
                account,
                oldpass,
                newpass,
            } => {
                let user = get_user_details(account.clone(), &mut self.mysql).await;
                match user {
                    Some(us) => {
                        println!("User {} exists", account);
                        us.print();
                        let password_success = us.check_login("lineage".to_string(), oldpass);
                        println!("User login check is {}", password_success);
                        if password_success {
                            println!("User wants to change password and entered correct details");
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
                println!("client: received unknown packet {:x?}", d);
            }
        }
        Ok(())
    }

    /// The main event loop for a client in a server.
    pub async fn event_loop(
        mut self,
        reader: tokio::net::tcp::OwnedReadHalf,
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
                    self.process_packet(p, peer).await?;
                }
                msg = self.brd_rx.recv().fuse() => {
                    let p = msg.unwrap();
                    self.handle_server_message(p).await?;
                }
                msg = self.rx.recv().fuse() => {
                    match msg {
                        None => {}
                        Some(p) => {self.handle_server_message(p).await?;}
                    }
                }
            }
        }
        Ok(0)
    }
}
