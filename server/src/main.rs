mod client_message;
mod server;
mod update;
use client_message::*;

mod server_message;
use server_message::*;

mod client_data;
use client_data::*;

use std::collections::HashMap;

mod clients;
mod config;
use config::*;
mod player;
mod user;
mod world;
use crate::clients::ClientList;
use crate::player::Player;

/// Handle a message from a user
async fn handle_user_message(
    res: client_message::ClientMessage,
    testvar: &mut u32,
    client_ids: &mut ClientList,
    clients: &mut HashMap<u32, tokio::sync::mpsc::UnboundedSender<ServerMessage>>,
    client_accounts: &mut HashMap<u32, String>,
    broadcast: &tokio::sync::broadcast::Sender<ServerMessage>,
) {
    *testvar = 1;
    match res {
        ClientMessage::Register(tx) => {
            let new_id = client_ids.new_entry();
            clients.insert(new_id, tx.clone());
            let resp = clients
                .get(&new_id)
                .unwrap()
                .send(ServerMessage::AssignId(new_id));
            match resp {
                Err(_) => {
                    clients.remove(&new_id);
                }
                Ok(()) => println!("server: New client {} just registered {}", new_id, testvar),
            }
        }
        ClientMessage::Unregister(i) => {
            println!("server: client {} is unregistering", i);
            clients.remove(&i);
            client_ids.remove_entry(i);
        }
        ClientMessage::LoggedIn(id, account) => {
            client_accounts.insert(id, account);
        }
        ClientMessage::NewCharacter {
            id,
            name,
            class,
            gender,
            strength,
            dexterity,
            constitution,
            wisdom,
            charisma,
            intelligence,
        } => {
            let a = client_accounts.get(&id);
            let cid = clients.get(&id).unwrap();
            if let Some(account) = &a {
                println!("{} wants to make a new character {}", account, name);
                //TODO ensure player name does not already exist
                //TODO validate that all stats are legitimately possible
                //TODO validate count of characters for account

                if !Player::valid_name(name.clone()) {
                    if let Err(e) = cid.send(ServerMessage::CharacterCreateStatus(1)) {
                        println!("Failed to send char create status {} ", e);
                    }
                } else {
                    if let Err(e) = cid.send(ServerMessage::CharacterCreateStatus(0)) {
                        println!("Failed to send char create status {} ", e);
                    }
                    //TODO: populate the correct details
                    if let Err(e) = cid.send(ServerMessage::NewCharacterDetails {
                        name: name.clone(),
                        pledge: "".to_string(),
                        class: class,
                        gender: gender,
                        alignment: 32764,
                        hp: 234,
                        mp: 456,
                        ac: 12,
                        level: 1,
                        strength: strength,
                        dexterity: dexterity,
                        constitution: constitution,
                        wisdom: wisdom,
                        charisma: charisma,
                        intelligence: intelligence,
                    }) {
                        println!("Failed to send new char details {}", e);
                    }
                }
            }
        }
        ClientMessage::DeleteCharacter { id, name } => {
            let a = client_accounts.get(&id);
            if let Some(account) = &a {
                println!("{} wants to delete {}", account, name);
            }
        }
        ClientMessage::RegularChat { id: _, msg } => {
            //TODO limit based on distance and map
            let amsg = format!("[{}] {}", "unknown", msg);
            let _ = broadcast.send(ServerMessage::RegularChat { id: 0, msg: amsg });
        }
        ClientMessage::YellChat { id: _, msg, x, y } => {
            //TODO limit based on distance and map
            let amsg = format!("[{}] {}", "unknown", msg);
            let _ = broadcast.send(ServerMessage::YellChat {
                id: 0,
                msg: amsg,
                x,
                y,
            });
        }
        ClientMessage::GlobalChat(_id, msg) => {
            let amsg = format!("[{}] {}", "unknown", msg);
            let _ = broadcast.send(ServerMessage::GlobalChat(amsg));
        }
        ClientMessage::PledgeChat(_id, msg) => {
            let amsg = format!("[{}] {}", "unknown", msg);
            let _ = broadcast.send(ServerMessage::PledgeChat(amsg));
        }
        ClientMessage::PartyChat(_id, msg) => {
            let amsg = format!("[{}] {}", "unknown", msg);
            let _ = broadcast.send(ServerMessage::PartyChat(amsg));
        }
        ClientMessage::WhisperChat(_id, _person, msg) => {
            let _ = broadcast.send(ServerMessage::WhisperChat("unknown".to_string(), msg));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    common::do_stuff();
    println!("server: Game server is starting");

    let (clients, mut clients_rx) = tokio::sync::mpsc::channel::<ClientMessage>(100);
    let (broadcast, _) = tokio::sync::broadcast::channel::<ServerMessage>(100);

    let settings = load_config().unwrap();
    let mysql_pool = open_mysql(&settings).unwrap();
    println!("Trying to connect to database");
    let _mysql_conn = mysql_pool
        .get_conn()
        .await
        .expect("Failed to connect to mysql server");

    let cd: ClientData = ClientData::new(broadcast.clone(), clients, mysql_pool);

    let mut tasks: tokio::task::JoinSet<Result<(), u32>> = tokio::task::JoinSet::new();

    let world = std::sync::Arc::new(world::World::new());

    let update_tx = update::setup_update_server(&mut tasks, world.clone())
        .await
        .expect("Failed to setup update server");
    let server_tx = server::setup_game_server(cd, &mut tasks, world.clone())
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
                println!("A task exited {:?}, closing server in 5 seconds", r);
                error = Err(format!("A task exited {:?}, closing server in 5 seconds", r));
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                break;
            }
            Some(res) = clients_rx.recv() =>
                handle_user_message(
                    res,
                    &mut testvar,
                    &mut client_ids,
                    &mut clients,
                    &mut client_accounts,
                    &broadcast
                ).await,
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    let _ = broadcast.send(ServerMessage::Disconnect);

    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;

    println!("server: Server is shutting down");
    if let Err(e) = update_tx.send(0) {
        println!(
            "server: Failed to signal the update server to shutdown {}",
            e
        );
    }

    //mysql_conn.disconnect().await.expect("Failed to disconnect from mmysql server");

    if let Err(e) = server_tx.send(0) {
        println!("server: Failed to signal the server to shutdown {}", e);
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    println!("server: Server will now close");
    error
}
