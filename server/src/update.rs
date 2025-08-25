//! Code specific to the update portion of the server
//! This update server operates for the classic update client

use std::collections::HashMap;
use std::error::Error;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// A list of files to send to the client
#[derive(Debug)]
struct UpdateFileSet {
    /// The list of files to send
    files: Vec<String>,
    /// The new cs field for the game client
    new_cs: u32,
}

/// The set of files to deliver for each possible version of client
#[derive(Debug)]
struct UpdateFiles {
    /// The files to deliver for each version
    versions: HashMap<u32, UpdateFileSet>,
}

/// An error occurred in the update process
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

/// Process a single client for the update server
async fn process_update_client(
    mut socket: tokio::net::TcpStream,
    ufiles: std::sync::Arc<UpdateFiles>,
) -> Result<u8, UpdateError> {
    //the AsyncReadExt trait is used
    let timestamp = socket.read_u32().await?;
    //timestamp is the contents of time.dat
    log::info!(" Client checksum is {}", timestamp);

    if let Some(uf) = ufiles.versions.get(&timestamp) {
        //socket.write_i32_le(-2).await?;
        let number_files = uf.files.len() as u32;
        log::info!("Sending {} files to client", number_files);
        socket.write_u32(number_files).await?;
        for fname in &uf.files {
            let fname2 = format!("sprite/{}", fname);
            let pb = std::path::PathBuf::from(format!("./update-files/{}/{}.gz", timestamp, fname));
            log::info!("Sending {}", pb.display());
            let mut f = tokio::fs::File::open(pb).await.unwrap();
            let mut b = Vec::new();
            f.read_to_end(&mut b).await.unwrap();
            socket.write_u8(fname2.len() as u8).await.unwrap();
            socket.write_all(fname2.as_bytes()).await.unwrap();
            log::info!("Sending a file of length {}", b.len());
            socket.write_u32(b.len() as u32).await.unwrap();
            socket.write_all(b.as_slice()).await.unwrap();
            socket.write_u32(uf.new_cs).await.unwrap();
        }

        log::info!("Done sending files");
        socket.write_u32(0).await?; //unsure if necessary
        log::info!("Sending number of servers");
        socket.write_u32(1).await?; //number of servers
        let number_players = 1; // TODO update this?
        log::info!("Sending number of players");
        socket.write_u16(number_players).await?;
    } else {
        log::info!("update: client is using an invalid timestamp");
        if true {
            //socket.write_i32_le(-2).await?;
            let number_files = 0u32;
            log::info!("Sending {} files to client", number_files);
            socket.write_u32(number_files).await?;
            socket.write_u32(0).await?; //unsure if necessary
            log::info!("Sending number of servers");
            socket.write_u32(1).await?; //number of servers
            let number_players = 1; //TODO update this value?
            log::info!("Sending number of players");
            socket.write_u16(number_players).await?;
        } else {
            return Err(UpdateError);
        }
    }
    log::info!("Reading restime from client");
    let restime = socket.read_u32().await?; //loaded from restime.dat
    log::info!(" Client restime is {}", restime);

    Ok(0)
}

/// The update server
struct UpdateServer {
    /// The socket for listening for new update clients
    update_listener: TcpListener,
    /// Used to receive a message to end the update server
    update_rx: tokio::sync::oneshot::Receiver<u32>,
    /// The list of updates to deploy to clients
    updates: std::sync::Arc<UpdateFiles>,
}

impl UpdateServer {
    /// Run the update server
    async fn run(mut self) -> Result<(), u32> {
        loop {
            tokio::select! {
                Ok(res) = self.update_listener.accept() => {
                    let (socket, addr) = res;
                    log::info!("update: Received an update client from {}", addr);
                    let updates2 = self.updates.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = process_update_client(socket, updates2).await {
                            log::info!("update: Client {} errored during the update process {}", addr, e);
                        }
                    });
                }
                a = (&mut self.update_rx) => {
                    if let Ok(a) = a {
                        log::info!("update: Received a message {:?} to shut down", a);
                        break;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }
        log::info!("update: Ending the update server thread!");
        Ok(())
    }
}

/// Setup and start the update portion of the server
pub async fn setup_update_server(
    tasks: &mut tokio::task::JoinSet<Result<(), u32>>,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    log::info!("update: Starting the update server");
    let (update_tx, update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2003").await?;

    let mut updates = UpdateFiles {
        versions: HashMap::new(),
    };

    for f in (std::fs::read_dir("./update-files")?).flatten() {
        if f.path().is_dir() {
            let name = f.file_name().into_string().unwrap();
            let oldcs: u32 = name.parse().unwrap();
            log::info!("Found a entry for checksum {}", oldcs);
            let mut flist = Vec::new();
            let mut cs = None;
            for f2 in (std::fs::read_dir(f.path())?).flatten() {
                if f2.path().is_file() {
                    let update_file = f2.file_name().into_string().unwrap();
                    if update_file.ends_with(".gz") {
                        let newname = update_file.trim_end_matches(".gz").to_string();
                        log::info!("Found file {}", newname);
                        flist.push(newname);
                    }
                    if update_file == "newcs" {
                        let mut fcon = String::new();
                        let mut f3 = tokio::fs::File::open(f2.path()).await.unwrap();
                        f3.read_to_string(&mut fcon).await.unwrap();
                        let newcs2: u32 = fcon.parse().unwrap();
                        log::info!("Found a newcs file");
                        cs = Some(newcs2);
                    }
                }
            }
            if let Some(newcs) = cs {
                log::info!("Inserting entry for cs {} -> {}", oldcs, newcs);
                updates.versions.insert(
                    oldcs,
                    UpdateFileSet {
                        files: flist,
                        new_cs: newcs,
                    },
                );
            }
        }
    }

    let updates = std::sync::Arc::new(updates);

    let update_server = UpdateServer {
        update_listener,
        update_rx,
        updates,
    };

    tasks.spawn(update_server.run());

    Ok(update_tx)
}
