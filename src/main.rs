use std::error::Error;

use std::{thread, time};

mod update;
mod server;
mod client_message;
use client_message::*;

mod server_message;
use server_message::*;

mod client_data;
use client_data::*;

use futures::FutureExt;
use std::collections::HashMap;
use std::fs;

mod clients;
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

    let cd: ClientData = ClientData::new(broadcast.clone(), clients);

    let update_tx = update::setup_update_server().await?;
	let server_tx = server::setup_game_server(cd).await?;
	
	let mut client_ids : ClientList = ClientList::new();
	let mut clients : HashMap<u32,tokio::sync::mpsc::UnboundedSender<ServerMessage>> = HashMap::new();
	
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
				a = test1(testvar).fuse() => {
				}
				a = test2(testvar).fuse() => {
				}
                _ = tokio::signal::ctrl_c().fuse() => {
                    break;
                }
            }
        }

    println!("server: Server is shutting down");
    if let Err(e) = update_tx.send(0) {
        println!("server: Failed to signal the update server to shutdown {}", e);
    }
	if let Err(e) = server_tx.send(0) {
        println!("server: Failed to signal the server to shutdown {}", e);
    }
    thread::sleep(time::Duration::from_secs(1));
    println!("server: Server will now close");
    //return Err("some error".into());//
    Ok(())
}
