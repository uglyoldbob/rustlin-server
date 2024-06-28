mod config;
use std::{collections::HashSet, iter::FromIterator};

use config::*;
use mysql_async::prelude::{Query, Queryable, WithParams};

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("This is the server setup program");
    let settings = load_config().unwrap();
    let mysql_pool = open_mysql(&settings).unwrap();
    println!("Trying to connect to database");
    let mut mysql_conn = mysql_pool.get_conn().await.expect("Failed to connect to mysql server");
    println!("Ready to initialize database");
    let revision: Result<Vec<u32>, mysql_async::Error> = "SELECT revision FROM revision"
        .with(())
        .map(&mut mysql_conn, |revision| revision)
        .await;
    if let Err(mysql_async::Error::Server(mysql_async::ServerError{ code, message, state })) = &revision {
        if *code == 1146 {
            println!("Need to create revision table and initial dataset");
            let initial = tokio::fs::read("./database/initial.sql").await;
            let contents = String::from_utf8(initial.expect("Failed to open ./database/initial.sql")).unwrap();
            for statement in contents.lines() {
                println!("NEED TO APPLY {}", statement);
            }
        }
        else {
            println!("Unexpected error {:?}", revision);
            todo!();
        }
    }
    else {
        let revision_set: HashSet<u32> = HashSet::from_iter(revision.unwrap().into_iter());
        todo!();
    }
    Ok(())
}