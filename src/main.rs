use tokio::net::TcpListener;
use std::error::Error;

use std::{thread, time};

async fn process_update_client(socket: tokio::net::TcpStream) {
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Game server is starting");

    println!("Starting the update server");
    let update_listener = TcpListener::bind("0.0.0.0:2003").await?;

    let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();

    tokio::spawn(async move {
        println!("Update server things!");
        loop {
            tokio::select! {
                res = update_listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    println!("Received an update client from {}", addr);
                    tokio::spawn(async move { process_update_client(socket).await; });
                }
                _ = (&mut update_rx) => {
                    println!("Received a message to shut down");
                    break;
                }
            }
        }
        println!("Ending the update server thread!");
    });

    tokio::signal::ctrl_c().await.expect("Unable to listen for shutdown signal");
    println!("Server is shutting down");
    update_tx.send(0);
    thread::sleep(time::Duration::from_secs(10));
    println!("Server will now close");
    //return Err("some error".into());//
    Ok(())
}
