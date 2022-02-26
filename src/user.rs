use mysql_async::prelude::Queryable;

use chrono::{Utc, TimeZone};
use crypto::digest::Digest;

pub struct UserAccount {
    name: String,
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
		mysql_async::Value::Date(y,m,d,h,min,s,micro) => {
			Utc.ymd(y as i32,m as u32,d as u32).and_hms_milli(h as u32,min as u32,s as u32,micro as u32)
		}
		_ => Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444)
	}
}

pub async fn get_user_details(user: String, mysql: &mut mysql_async::Conn) -> Option<UserAccount> {
    let query = "SELECT login, password, access_level, ip, host, banned, character_slot, lastactive from accounts WHERE login=? LIMIT 1";
    let usertest = mysql.exec_map(
        query,
        (user,),
        |(a, pw, acc, ipa, h, b, slot, d): (String, String, u32, String, String, u32, u32, mysql_async::Value)| {
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
