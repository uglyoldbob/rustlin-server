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

use std::collections::HashMap;
use std::fs;

mod clients;
use crate::clients::ClientList;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server: Game server is starting");

    let (clients, mut clients_rx) = tokio::sync::mpsc::unbounded_channel::<ClientMessage>();
    let (broadcast, mut clients_rx_broadcast) = tokio::sync::broadcast::channel::<ServerMessage>(100);
	
	let settings_file = fs::read_to_string("./settings.ini")?;
	let mut settings = configparser::ini::Ini::new();
    settings.read(settings_file)?;

    let cd: ClientData = ClientData::new(broadcast, clients);

    let update_tx = update::setup_update_server().await?;
	let server_tx = server::setup_game_server(cd).await?;
	
	let mut client_ids : ClientList = ClientList::new();
	let mut clients : HashMap<u32,tokio::sync::mpsc::UnboundedSender<ServerMessage>> = HashMap::new();

    loop {
            tokio::select! {
                res = clients_rx.recv() => {
                    println!("server: received a message from a client");
					match res.unwrap() {
						ClientMessage::Register(rx) => {
							println!("Received a register request from a client");
							
						}
					}
                }
                _ = tokio::signal::ctrl_c() => {
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
