#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! The server for the game

mod client_message;
mod server;
mod update;
use client_message::*;

mod server_message;
use server_message::*;

use std::collections::HashMap;

mod character;
mod clients;
mod config;
use config::*;
mod user;
mod world;
use crate::clients::ClientList;
use crate::character::Character;

#[tokio::main]
async fn main() -> Result<(), String> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    common::do_stuff();
    log::info!("server: Game server is starting");

    let (broadcast, _) = tokio::sync::broadcast::channel::<ServerMessage>(100);

    let settings = load_config().unwrap();
    let mysql_pool = open_mysql(&settings).unwrap();
    log::info!("Trying to connect to database");
    let _mysql_conn = mysql_pool
        .get_conn()
        .await
        .expect("Failed to connect to mysql server");

    let mut tasks: tokio::task::JoinSet<Result<(), u32>> = tokio::task::JoinSet::new();

    let world = std::sync::Arc::new(world::World::new(broadcast, mysql_pool));

    let update_tx = update::setup_update_server(&mut tasks, world.clone())
        .await
        .expect("Failed to setup update server");
    let server_tx = server::setup_game_server(&mut tasks, world.clone(), &settings.config)
        .await
        .expect("Failed to setup legacy server");

    let mut client_ids: ClientList = ClientList::new();
    let mut clients: HashMap<u32, tokio::sync::mpsc::UnboundedSender<ServerMessage>> =
        HashMap::new();
    let mut client_accounts: HashMap<u32, String> = HashMap::new();

    let mut testvar: u32 = 0;

    let mut error = Ok(());

    loop {
        tokio::select! {
            Some(r) = tasks.join_next() => {
                log::info!("A task exited {:?}, closing server in 5 seconds", r);
                error = Err(format!("A task exited {:?}, closing server in 5 seconds", r));
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    let _ = world.global_tx.send(ServerMessage::Disconnect);

    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;

    log::info!("server: Server is shutting down");
    if let Err(e) = update_tx.send(0) {
        log::info!(
            "server: Failed to signal the update server to shutdown {}",
            e
        );
    }

    //mysql_conn.disconnect().await.expect("Failed to disconnect from mmysql server");

    if let Err(e) = server_tx.send(0) {
        log::info!("server: Failed to signal the server to shutdown {}", e);
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    log::info!("server: Server will now close");
    error
}
