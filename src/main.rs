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
mod user;
use crate::clients::ClientList;

async fn test1(mut c: u32) -> String {
    c = c + 1;
    "asdf".to_string()
}

async fn test2(mut c: u32) -> String {
    c = c * 2;
    "asdffdsa".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server: Game server is starting");

    let (clients, mut clients_rx) = tokio::sync::mpsc::channel::<ClientMessage>(100);
    let (broadcast, _) = tokio::sync::broadcast::channel::<ServerMessage>(100);

    let settings_file = fs::read_to_string("./settings.ini")?;
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
    let mut mysql_conn = mysql_pool.get_conn().await?;

    let cd: ClientData = ClientData::new(broadcast.clone(), clients, mysql_pool);

    let update_tx = update::setup_update_server().await?;
    let server_tx = server::setup_game_server(cd).await?;

    let mut client_ids: ClientList = ClientList::new();
    let mut clients: HashMap<u32, tokio::sync::mpsc::UnboundedSender<ServerMessage>> =
        HashMap::new();

    let mut testvar: u32 = 5;

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
                    ClientMessage::RegularChat{id, msg} => {
                        broadcast.send(ServerMessage::RegularChat{id:0, msg:msg});
                    }
                    ClientMessage::YellChat{id, msg, x, y} => {
                    }
                    ClientMessage::GlobalChat(id, msg) => {
                    }
                    ClientMessage::PledgeChat(id, msg) => {
                    }
                    ClientMessage::PartyChat(id, msg) => {
                    }
                    ClientMessage::WhisperChat(id, person, msg) => {
                    }
                }
            }
            _ = tokio::signal::ctrl_c().fuse() => {
                break;
            }
        }
    }

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
