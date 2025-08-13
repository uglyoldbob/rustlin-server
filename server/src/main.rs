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

/// The reasons to shutdown the server
#[derive(Debug)]
enum ShutdownMode {
    /// A normal server shutdown
    Normal,
    /// A fatal error occurred during server operation
    Error(String),
    /// The server needs to be restarted, indicate this as an process exit code
    Restart,
    /// An abnormal or unexpected shutdown of the server
    Abnormal,
}

/// The main function of the server.
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

    let (iscs, mut iscr) = tokio::sync::mpsc::channel(5);

    let world = std::sync::Arc::new(
        world::World::new(mysql_pool, iscs)
            .await
            .map_err(|e| format!("{:?}", e))?,
    );
    world.spawn_monsters().await;

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

    let mut shutdown_mode = None;

    loop {
        tokio::select! {
            Some(r) = tasks.join_next() => {
                if let Ok(r2) = r {
                    if let Ok(r3) = r2 {
                        log::info!("A task exited {:?}, closing server in 5 seconds", r3);
                        if shutdown_mode.is_none() {
                            shutdown_mode = Some(ShutdownMode::Abnormal);
                        }
                    }
                    else {
                        log::error!("A task exited {:?}, closing server in 5 seconds", r2);
                        shutdown_mode = Some(ShutdownMode::Error(format!("A task exited {:?}, closing server now", r)));
                    }
                }
                else {
                    log::error!("A task exited {:?}, closing server in 5 seconds", r);
                    shutdown_mode = Some(ShutdownMode::Error(format!("A task exited {:?}, closing server now", r)))
                }
                break;
            }
            m = iscr.recv() => {
                if let Some(m) = m {
                    match m {
                        server_message::ServerShutdownMessage::Shutdown => {
                            shutdown_mode = Some(ShutdownMode::Normal);
                            if let Some(tx) = server_tx.take() {
                                log::info!("Signal end of server");
                                let _ = tx.send(0);
                            }
                            if let Some(tx) = update_tx.take() {
                                log::info!("Signal end of update");
                                let _ = tx.send(0);
                            }
                        }
                        server_message::ServerShutdownMessage::Restart => {
                            shutdown_mode = Some(ShutdownMode::Restart);
                            if let Some(tx) = server_tx.take() {
                                log::info!("Signal end of server");
                                let _ = tx.send(0);
                            }
                            if let Some(tx) = update_tx.take() {
                                log::info!("Signal end of update");
                                let _ = tx.send(0);
                            }
                        }
                    }
                } else {
                    log::error!("Internal channel for shutdown broken");
                    if let Some(tx) = server_tx.take() {
                        log::info!("Signal end of server");
                        let _ = tx.send(0);
                    }
                    if let Some(tx) = update_tx.take() {
                        log::info!("Signal end of update");
                        let _ = tx.send(0);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                shutdown_mode = Some(ShutdownMode::Normal);
                if let Some(tx) = server_tx.take() {
                    log::info!("Signal end of server");
                    let _ = tx.send(0);
                }
                if let Some(tx) = update_tx.take() {
                    log::info!("Signal end of update");
                    let _ = tx.send(0);
                }
            }
        }
    }
    log::info!("Waiting for main tasks to finish");
    tasks.join_all().await;

    log::info!(
        "server: Server will now close with status: {:?}",
        shutdown_mode
    );
    match shutdown_mode.unwrap() {
        ShutdownMode::Normal => Ok(()),
        ShutdownMode::Error(s) => Err(s),
        ShutdownMode::Restart => Err("Restarting".to_string()),
        ShutdownMode::Abnormal => Err("Abnormal shutdown".to_string()),
    }
}
