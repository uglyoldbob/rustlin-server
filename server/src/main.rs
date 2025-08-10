#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

#![feature(async_drop)]

//! The server for the game

mod client_message;
mod server;
mod update;
use client_message::*;

mod server_message;
use server_message::*;

mod character;
mod clients;
mod config;
use config::*;
mod user;
mod world;
use crate::clients::ClientList;

#[tokio::main]
async fn main() -> Result<(), String> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    common::do_stuff();
    log::info!("server: Game server is starting");

    let settings = load_config().unwrap();
    let mysql_pool = open_mysql(&settings).unwrap();
    log::info!("Trying to connect to database");
    let _mysql_conn = mysql_pool
        .get_conn()
        .await
        .expect("Failed to connect to mysql server");

    let mut tasks: tokio::task::JoinSet<Result<(), u32>> = tokio::task::JoinSet::new();

    let world = std::sync::Arc::new(world::World::new(mysql_pool));
    world.load_maps_data().await?;
    world.load_item_data().await?;

    let mut update_tx = Some(update::setup_update_server(&mut tasks, world.clone())
        .await
        .expect("Failed to setup update server"));
    let mut server_tx = Some(
        server::setup_game_server(&mut tasks, world.clone(), &settings.config)
            .await
            .expect("Failed to setup legacy server"),
    );

    let error;

    loop {
        tokio::select! {
            Some(r) = tasks.join_next() => {
                log::info!("A task exited {:?}, closing server in 5 seconds", r);
                error = Err(format!("A task exited {:?}, closing server in 5 seconds", r));
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                if let Some(tx) = server_tx.take() {
                    log::info!("Signal end of server");
                    tx.send(0);
                }
                if let Some(tx) = update_tx.take() {
                    log::info!("Signal end of update");
                    tx.send(0);
                }
            }
        }
    }

    log::info!("server: Server will now close");
    error
}
