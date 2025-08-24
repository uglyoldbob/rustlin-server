//! Code for managing user accounts on the server

use mysql::prelude::Queryable;

use chrono::{TimeZone, Utc};
use crypto::digest::Digest;

/// A user account on the server
#[derive(Debug)]
pub struct UserAccount {
    /// The name for the account in the database
    name: String,
    ///the hashed password, don't be a moron
    password: String,
    /// Last time the account was active
    active: chrono::DateTime<chrono::Utc>,
    /// The access level of the account
    access: u32,
    /// The last login ip address
    ip: String,
    /// The host the account logged in from
    host: String,
    /// Is the account banned?
    banned: bool,
    /// The number of characters slots the account has?
    slot: u32,
}

/// Hash the password for the database
pub fn hash_password(name: &str, salt: &str, pw: &str) -> String {
    let mut md5 = crypto::md5::Md5::new();
    md5.input_str(name);
    let m = md5.result_str();
    let inp = format!("{}{}{}", salt, pw, m);
    let mut sha = crypto::sha2::Sha256::new();
    sha.input_str(&inp);
    sha.result_str()
}

/// Convert the mysql value to a usable date time
fn convert_date(d: mysql::Value) -> chrono::DateTime<chrono::Utc> {
    let dt = match d {
        mysql::Value::Date(y, m, d, h, min, s, _micro) => {
            Utc.with_ymd_and_hms(y as i32, m as u32, d as u32, h as u32, min as u32, s as u32)
        }
        _ => Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 1),
    };
    dt.single().or(dt.latest()).unwrap()
}

/// Get a user account from the db, if it exists
pub fn get_user_details(user: String, mysql: &mut mysql::PooledConn) -> Option<UserAccount> {
    let query = "SELECT login, password, access_level, ip, host, banned, character_slot, lastactive from accounts WHERE login=? LIMIT 1";
    let usertest = mysql.exec_map(
        query,
        (user,),
        |(a, pw, acc, ipa, h, b, slot, d): (
            String,
            String,
            u32,
            String,
            String,
            u32,
            u32,
            mysql::Value,
        )| {
            UserAccount {
                name: a,
                password: pw,
                active: convert_date(d),
                access: acc,
                ip: ipa,
                host: h,
                banned: b != 0,
                slot: slot,
            }
        },
    );
    usertest.unwrap().pop()
}

impl UserAccount {
    /// Check login to see if the password was correct
    pub fn check_login(&self, salt: &str, pw: &str) -> bool {
        let hash = hash_password(&self.name, salt, pw);
        hash == self.password
    }

    /// Construct a new user account
    pub fn new(name: String, pass: String, ip: String, salt: String) -> Self {
        let hashpass = hash_password(&name, &salt, &pass);
        Self {
            name: name.clone(),
            password: hashpass,
            active: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 1).single().unwrap(),
            access: 0,
            ip: ip.clone(),
            host: ip.clone(),
            banned: false,
            slot: 0,
        }
    }

    /// Get the account name
    pub fn account_name(&self) -> &str {
        &self.name
    }

    /// Retrieve characters for user account from database
    pub fn retrieve_chars(
        &self,
        mysql: &mut mysql::PooledConn,
    ) -> Result<Vec<crate::character::Character>, crate::server::ClientError> {
        crate::character::Character::retrieve_chars(&self.name, mysql)
    }

    /// Insert a new account into the database
    pub fn insert_into_db(&self, mysql: &mut mysql::PooledConn) {
        let query = "INSERT INTO accounts SET login=?,password=?,lastactive=?,access_level=?,ip=?,host=?,banned=?,character_slot=?";
        let tq = mysql.exec_drop(
            query,
            (
                self.name.clone(),
                self.password.clone(),
                mysql::Value::Date(2010, 3, 5, 4, 5, 6, 100),
                self.access,
                self.ip.clone(),
                self.host.clone(),
                if self.banned { 1 } else { 0 },
                self.slot,
            ),
        );
        match tq {
            Err(e) => {
                log::info!("error inserting account {}", e);
            }
            _ => {
                log::info!("account insertion is fine");
            }
        }
    }
}
