#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![feature(async_drop)]

//! The server for the game

mod client_message;
mod server;
mod update;
use client_message::*;

mod server_message;

mod character;
mod clients;
mod config;
use config::*;
mod user;
mod world;
use crate::clients::ClientList;

fn main() -> Result<(), String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(32 * 1024 * 1024)
        .build()
        .unwrap()
        .block_on(smain())
}

async fn smain() -> Result<(), String> {
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

    let world = std::sync::Arc::new(
        world::World::new(mysql_pool)
            .await
            .map_err(|e| format!("{:?}", e))?,
    );

    let mut update_tx = Some(
        update::setup_update_server(&mut tasks, world.clone())
            .await
            .expect("Failed to setup update server"),
    );
    let mut server_tx = Some(
        server::setup_game_server(&mut tasks, world.clone(), &settings.config)
            .await
            .expect("Failed to setup legacy server"),
    );

    let error;

    loop {
        tokio::select! {
            Some(r) = tasks.join_next() => {
                if let Ok(r2) = r {
                    if r.is_ok() {
                        log::info!("A task exited {:?}, closing server in 5 seconds", r2);
                    }
                    else {
                        log::error!("A task exited {:?}, closing server in 5 seconds", r2);
                    }
                }
                else {
                    log::error!("A task exited {:?}, closing server in 5 seconds", r);
                }
                error = Err(format!("A task exited {:?}, closing server now", r));
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
    log::info!("Waiting for main tasks to finish");
    tasks.join_all().await;

    log::info!("server: Server will now close");
    error
}
