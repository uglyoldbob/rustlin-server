use futures::FutureExt;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::panic::AssertUnwindSafe;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
struct UpdateFileSet {
    files: Vec<String>,
    new_cs: u32,
}

#[derive(Debug)]
struct UpdateFiles {
    versions: HashMap<u32, UpdateFileSet>,
}

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
    ufiles: std::sync::Arc<UpdateFiles>,
) -> Result<u8, UpdateError> {
    //the AsyncReadExt trait is used
    let timestamp = socket.read_u32().await?;
    //timestamp is the contents of time.dat
    println!(" Client checksum is {}", timestamp);

    if let Some(uf) = ufiles.versions.get(&timestamp) {
        //socket.write_i32_le(-2).await?;
        let number_files = uf.files.len() as u32;
        println!("Sending {} files to client", number_files);
        socket.write_u32(number_files).await?;
        for fname in &uf.files {
            let fname2 = format!("sprite/{}", fname);
            let pb = std::path::PathBuf::from(format!("./update-files/{}/{}.gz", timestamp, fname));
            println!("Sending {}", pb.display());
            let mut f = tokio::fs::File::open(pb).await.unwrap();
            let mut b = Vec::new();
            f.read_to_end(&mut b).await.unwrap();
            socket.write_u8(fname2.len() as u8).await.unwrap();
            socket.write_all(fname2.as_bytes()).await.unwrap();
            println!("Sending a file of length {}", b.len());
            socket.write_u32(b.len() as u32).await.unwrap();
            socket.write_all(b.as_slice()).await.unwrap();
            socket.write_u32(uf.new_cs).await.unwrap();
        }

        println!("Done sending files");
        socket.write_u32(0).await?; //unsure if necessary
        println!("Sending number of servers");
        socket.write_u32(1).await?; //number of servers
        let number_players = world.get_number_players();
        println!("Sending number of players");
        socket.write_u16(number_players).await?;
    } else {
        println!("update: client is using an invalid timestamp");
        if true {
            //socket.write_i32_le(-2).await?;
            let number_files = 0u32;
            println!("Sending {} files to client", number_files);
            socket.write_u32(number_files).await?;
            socket.write_u32(0).await?; //unsure if necessary
            println!("Sending number of servers");
            socket.write_u32(1).await?; //number of servers
            let number_players = world.get_number_players();
            println!("Sending number of players");
            socket.write_u16(number_players).await?;
        } else {
            return Err(UpdateError);
        }
    }
    println!("Reading restime from client");
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

    let mut updates = UpdateFiles {
        versions: HashMap::new(),
    };

    for f in std::fs::read_dir("./update-files")? {
        if let Ok(f) = f {
            if f.path().is_dir() {
                let name = f.file_name().into_string().unwrap();
                let oldcs: u32 = name.parse().unwrap();
                println!("Found a entry for checksum {}", oldcs);
                let mut flist = Vec::new();
                let mut cs = None;
                for f2 in std::fs::read_dir(f.path())? {
                    if let Ok(f2) = f2 {
                        if f2.path().is_file() {
                            let update_file = f2.file_name().into_string().unwrap();
                            if update_file.ends_with(".gz") {
                                let newname = update_file.trim_end_matches(".gz").to_string();
                                println!("Found file {}", newname);
                                flist.push(newname);
                            }
                            if update_file == "newcs" {
                                let mut fcon = String::new();
                                let mut f3 = tokio::fs::File::open(f2.path()).await.unwrap();
                                f3.read_to_string(&mut fcon).await.unwrap();
                                let newcs2: u32 = fcon.parse().unwrap();
                                println!("Found a newcs file");
                                cs = Some(newcs2);
                            }
                        }
                    }
                }
                if let Some(newcs) = cs {
                    println!("Inserting entry for cs {} -> {}", oldcs, newcs);
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
    }

    let updates = std::sync::Arc::new(updates);

    tasks.spawn(async move {
        let mut f = futures::stream::FuturesUnordered::new();
        loop {
            use futures::stream::StreamExt;
            tokio::select! {
                Ok(res) = update_listener.accept() => {
                    let (socket, addr) = res;
                    println!("update: Received an update client from {}", addr);
                    let world2 = world.clone();
                    let updates2 = updates.clone();
                    f.push(async move {
                        if let Err(e) = process_update_client(socket, world2, updates2).await {
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
