use futures::FutureExt;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
struct UpdateError;
impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client failed to update")
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(_: std::io::Error) -> UpdateError {
        UpdateError {}
    }
}

async fn process_update_client(
    mut socket: tokio::net::TcpStream,
    world: std::sync::Arc<crate::world::World>,
) -> Result<u8, UpdateError> {
    //the AsyncReadExt trait is used
    let timestamp = socket.read_u32().await?;
    //timestamp is the contents of time.dat
    println!(" Client checksum is {}", timestamp);

    //find relevant entry for files with timestamp
    //if there is a match
    let located = true;
    if located {
        socket.write_i32_le(-2).await?;
        let number_files = 0;
        socket.write_u32_le(number_files).await?;
        //TODO: send all files
        socket.write_u32_le(0).await?;
        socket.write_u32_le(1).await?;
        let number_players = world.get_number_players();
        socket.write_u16_le(number_players).await?;
    } else {
        println!("update: client is using an invalid timestamp");
        return Err(UpdateError);
    }

    let restime = socket.read_u32().await?; //loaded from restime.dat
    println!(" Client restime is {}", restime);

    Ok(0)
}

pub async fn setup_update_server(
    tasks: &mut tokio::task::JoinSet<Result<(), u32>>,
    world: std::sync::Arc<crate::world::World>,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    println!("update: Starting the update server");
    let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2003").await?;

    tasks.spawn(async move {
        let mut f = futures::stream::FuturesUnordered::new();
        loop {
            use futures::stream::StreamExt;
            tokio::select! {
                Ok(res) = update_listener.accept() => {
                    let (socket, addr) = res;
                    println!("update: Received an update client from {}", addr);
                    let world2 = world.clone();
                    f.push(async move {
                        if let Err(e) = process_update_client(socket, world2).await {
                            println!("update: Client {} errored during the update process {}", addr, e);
                        }
                    });
                }
                Ok(Some(_)) = AssertUnwindSafe(f.next()).catch_unwind() => {}
                _ = (&mut update_rx) => {
                    println!("update: Received a message to shut down");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }
        println!("update: Ending the update server thread!");
        Ok(())
    });

    Ok(update_tx)
}
