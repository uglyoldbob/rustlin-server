use std::error::Error;

use std::{thread, time};

mod client_message;
mod server;
mod update;
use client_message::*;

mod server_message;
use server_message::*;

mod client_data;
use client_data::*;

use futures::FutureExt;
use std::collections::HashMap;
use std::fs;

mod clients;
mod player;
mod user;
use crate::clients::ClientList;
use crate::player::Player;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    common::do_stuff();
    println!("server: Game server is starting");

    let (clients, mut clients_rx) = tokio::sync::mpsc::channel::<ClientMessage>(100);
    let (broadcast, _) = tokio::sync::broadcast::channel::<ServerMessage>(100);

    let settings_file = fs::read_to_string("./server-settings.ini")?;
    let mut settings = configparser::ini::Ini::new();
    settings.read(settings_file)?;

    let mysql_pw = settings
        .get("database", "password")
        .unwrap_or("invalid".to_string());
    let mysql_user = settings
        .get("database", "username")
        .unwrap_or("invalid".to_string());
    let mysql_dbname = settings
        .get("database", "name")
        .unwrap_or("none".to_string());
    let mysql_url = settings
        .get("database", "url")
        .unwrap_or("invalid".to_string());
    let mysql_conn_s = format!(
        "mysql://{}:{}@{}/{}",
        mysql_user, mysql_pw, mysql_url, mysql_dbname
    );
    let mysql_opt = mysql_async::Opts::from_url(mysql_conn_s.as_str()).unwrap();
    let mysql_pool = mysql_async::Pool::new(mysql_opt);
    println!("Trying to connecto to database");
    let mysql_conn = mysql_pool.get_conn().await?;

    let cd: ClientData = ClientData::new(broadcast.clone(), clients, mysql_pool);

    let update_tx = update::setup_update_server().await?;
    let server_tx = server::setup_game_server(cd).await?;

    let mut client_ids: ClientList = ClientList::new();
    let mut clients: HashMap<u32, tokio::sync::mpsc::UnboundedSender<ServerMessage>> =
        HashMap::new();
    let mut client_accounts: HashMap<u32, String> = HashMap::new();

    let mut testvar: u32;

    loop {
        futures::select! {
            res = clients_rx.recv().fuse() => {
                testvar = 1;
                match res.unwrap() {
                    ClientMessage::Register(tx) => {
                        let new_id = client_ids.new_entry();
                        clients.insert(new_id, tx.clone());
                        let resp = clients.get(&new_id).unwrap().send(ServerMessage::AssignId(new_id));
                        match resp {
                            Err(_) => {
                                clients.remove(&new_id);
                            }
                            Ok(()) => println!("server: New client {} just registered {}", new_id, testvar),
                        }
                    }
                    ClientMessage::Unregister(i) => {
                        println!("server: client {} is unregistering", i);
                        clients.remove(&i);
                        client_ids.remove_entry(i);
                    }
                    ClientMessage::LoggedIn(id, account) => {
                        client_accounts.insert(id, account);
                    }
                    ClientMessage::NewCharacter{id, name, class, gender, strength,
                        dexterity, constitution, wisdom, charisma, intelligence} => {
                        let a = client_accounts.get(&id);
                        let cid = clients.get(&id).unwrap();
                        if let Some(account) = &a {
                            println!("{} wants to make a new character {}", account.clone(), name.clone());
                            //TODO ensure player name does not already exist
                            //TODO validate that all stats are legitimately possible
                            //TODO validate count of characters for account

                            if !Player::valid_name(name.clone()) {
                                if let Err(e) = cid.send(ServerMessage::CharacterCreateStatus(1)) {
                                    println!("Failed to send char create status {} ", e);
                                }
                            }
                            else {
                                if let Err(e) = cid.send(ServerMessage::CharacterCreateStatus(0)) {
                                    println!("Failed to send char create status {} ", e);
                                }
                                //TODO: populate the correct details
                                if let Err(e) = cid.send(ServerMessage::NewCharacterDetails{
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
                                }) {
                                    println!("Failed to send new char details {}", e);
                                }
                            }
                        }
                    }
                    ClientMessage::DeleteCharacter{id, name} => {
                        let a = client_accounts.get(&id);
                        if let Some(account) = &a {
                            println!("{} wants to delete {}", account.clone(), name);
                        }
                    }
                    ClientMessage::RegularChat{id, msg} => {
                        //TODO limit based on distance and map
                        let amsg = format!("[{}] {}", "unknown", msg);
                        let _ = broadcast.send(ServerMessage::RegularChat{id:0, msg:amsg});
                    }
                    ClientMessage::YellChat{id, msg, x, y} => {
                        //TODO limit based on distance and map
                        let amsg = format!("[{}] {}", "unknown", msg);
                        let _ = broadcast.send(ServerMessage::YellChat{id:0, msg:amsg, x: y, y: y});
                    }
                    ClientMessage::GlobalChat(id, msg) => {
                        let amsg = format!("[{}] {}", "unknown", msg);
                        let _ = broadcast.send(ServerMessage::GlobalChat(amsg));
                    }
                    ClientMessage::PledgeChat(id, msg) => {
                        let amsg = format!("[{}] {}", "unknown", msg);
                        let _ = broadcast.send(ServerMessage::PledgeChat(amsg));
                    }
                    ClientMessage::PartyChat(id, msg) => {
                        let amsg = format!("[{}] {}", "unknown", msg);
                        let _ = broadcast.send(ServerMessage::PartyChat(amsg));
                    }
                    ClientMessage::WhisperChat(id, person, msg) => {
                        let _ = broadcast.send(ServerMessage::WhisperChat("unknown".to_string(), msg));
                    }
                }
            }
            _ = tokio::signal::ctrl_c().fuse() => {
                break;
            }
        }
    }

    let _ = broadcast.send(ServerMessage::Disconnect);

    thread::sleep(time::Duration::from_secs(5));

    println!("server: Server is shutting down");
    if let Err(e) = update_tx.send(0) {
        println!(
            "server: Failed to signal the update server to shutdown {}",
            e
        );
    }

    mysql_conn.disconnect().await?;

    if let Err(e) = server_tx.send(0) {
        println!("server: Failed to signal the server to shutdown {}", e);
    }
    thread::sleep(time::Duration::from_secs(1));
    println!("server: Server will now close");
    //return Err("some error".into());//
    Ok(())
}
