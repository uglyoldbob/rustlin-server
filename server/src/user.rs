use mysql_async::prelude::Queryable;

use chrono::{TimeZone, Utc};
use crypto::digest::Digest;

pub struct UserAccount {
    name: String,
    ///the hashed password, don't be a moron
    password: String,
    active: chrono::DateTime<chrono::Utc>,
    access: u32,
    ip: String,
    host: String,
    banned: bool,
    slot: u32,
}

pub fn hash_password(name: String, salt: String, pw: String) -> String {
    let mut md5 = crypto::md5::Md5::new();
    md5.input_str(&name);
    let m = md5.result_str();
    let inp = format!("{}{}{}", salt, pw, m);
    let mut sha = crypto::sha2::Sha256::new();
    sha.input_str(&inp);
    sha.result_str()
}

fn convert_date(d: mysql_async::Value) -> chrono::DateTime<chrono::Utc> {
    match d {
        mysql_async::Value::Date(y, m, d, h, min, s, micro) => Utc
            .ymd(y as i32, m as u32, d as u32)
            .and_hms_milli(h as u32, min as u32, s as u32, micro as u32),
        _ => Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444),
    }
}

pub async fn get_user_details(user: String, mysql: &mut mysql_async::Conn) -> Option<UserAccount> {
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
            mysql_async::Value,
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
    usertest.await.unwrap().pop()
}

impl UserAccount {
    pub fn check_login(&self, salt: String, pw: String) -> bool {
        let hash = hash_password(self.name.clone(), salt, pw);
        hash == self.password
    }

    pub fn new(name: String, pass: String, ip: String, salt: String) -> Self {
        let hashpass = hash_password(name.clone(), salt, pass);
        Self {
            name: name.clone(),
            password: hashpass,
            active: Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444),
            access: 0,
            ip: ip.clone(),
            host: ip.clone(),
            banned: false,
            slot: 0,
        }
    }

    pub async fn insert_into_db(&self, mysql: &mut mysql_async::Conn) {
        let query = "INSERT INTO accounts SET login=?,password=?,lastactive=?,access_level=?,ip=?,host=?,banned=?,character_slot=?";
        let tq = mysql.exec_drop(
            query,
            (
                self.name.clone(),
                self.password.clone(),
                mysql_async::Value::Date(2010, 03, 05, 04, 05, 06, 100),
                self.access,
                self.ip.clone(),
                self.host.clone(),
                if self.banned { 0 } else { 1 },
                self.slot,
            ),
        );
        let err = tq.await;
        match err {
            Err(e) => {
                println!("error inserting account {}", e);
            }
            _ => {
                println!("account insertion is fine");
            }
        }
    }

    pub fn print(&self) -> () {
        println!(
            "User details: {} {} {} {} {} {} {} {}",
            self.name,
            self.password,
            self.active,
            self.access,
            self.ip,
            self.host,
            self.banned,
            self.slot
        );
    }
}
