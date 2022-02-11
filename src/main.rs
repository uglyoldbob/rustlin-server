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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server: Game server is starting");

    let (clients, mut clients_rx) = tokio::sync::mpsc::unbounded_channel::<ClientMessage>();
    let (broadcast, mut clients_rx_broadcast) = tokio::sync::broadcast::channel::<ServerMessage>(100);

    let cd: ClientData = ClientData::new(broadcast, clients);

    let update_tx = update::setup_update_server().await?;
	let server_tx = server::setup_game_server(cd).await?;


    loop {
            tokio::select! {
                res = clients_rx.recv() => {
                    println!("server: received a message from a client");
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
