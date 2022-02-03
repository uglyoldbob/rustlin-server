use std::error::Error;

use std::{thread, time};

mod update;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("server: Game server is starting");

    let update_tx = update::setup_update_server().await?;
	let server_tx = server::setup_game_server().await?;

    tokio::signal::ctrl_c().await.expect("server: Unable to listen for shutdown signal");
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
