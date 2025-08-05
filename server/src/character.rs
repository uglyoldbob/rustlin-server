use common::packet::ServerPacket;
use mysql_async::Params;

/// Represents a playable character in the game
#[derive(Debug)]
pub struct Character {
    /// The account name for the character
    account_name: String,
    /// The name of the character
    name: String,
    /// The id of the character in the database
    id: u32,
    /// The alignment of the character
    alignment: i16,
    /// The level of the character
    level: u8,
    /// The pledge name of the character (empty string if no pledge)
    pledge: String,
    /// The class of character
    class: u8,
    /// The gender
    gender: u8,
    /// The current max hp
    hp_max: u16,
    /// The current mp max
    mp_max: u16,
    /// Current armor class
    ac: i8,
    /// Character strength
    strength: u8,
    /// Character dexterity
    dexterity: u8,
    /// Character constitution
    constitution: u8,
    /// Character wisdom
    wisdom: u8,
    /// Character charisma
    charisma: u8,
    /// Character intelligence
    intelligence: u8,
}

impl Character {
    pub const QUERY: &str = "SELECT account_name, char_name, objid, Lawful, level, Clanname, Class, Sex, MaxHp, MaxMp, Ac, Str, Dex, Con, Wis, Cha, Intel from characters WHERE account_name=?";

    /// Is the player name valid?
    pub fn valid_name(n: &str) -> bool {
        !n.is_empty()
    }

    /// Get the player name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the player alignment
    pub fn alignment(&self) -> i16 {
        self.alignment
    }

    /// Construct a details packet for informing the user of a character they can log in with
    pub fn get_details_packet(&self) -> ServerPacket {
        ServerPacket::LoginCharacterDetails {
            name: self.name.clone(),
            pledge: self.pledge.clone(),
            ctype: self.class,
            gender: self.gender,
            alignment: self.alignment,
            hp: self.hp_max,
            mp: self.mp_max,
            ac: self.ac,
            level: self.level,
            strength: self.strength,
            dexterity: self.dexterity,
            constitution: self.constitution,
            wisdom: self.wisdom,
            charisma: self.charisma,
            intelligence: self.intelligence,
        }
    }

    pub fn get_new_char_details_packet(&self) -> ServerPacket {
        ServerPacket::NewCharacterDetails {
            name: self.name().to_string(),
            pledge: self.pledge.to_string(),
            class: self.class,
            gender: self.gender,
            alignment: self.alignment,
            hp: self.hp_max,
            mp: self.mp_max,
            ac: self.ac,
            level: self.level,
            strength: self.strength,
            dexterity: self.dexterity,
            constitution: self.constitution,
            wisdom: self.wisdom,
            charisma: self.charisma,
            intelligence: self.intelligence,
        }
    }

    /// Save a new character into the database
    pub async fn save_new_to_db(&self, mysql: &mut mysql_async::Conn) -> Result<(), crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = "INSERT INTO characters SET account_name=?,objid=?,char_name=?,level=?,MaxHp=?,MaxMp=?,Class=?,Sex=?,Ac=?,Str=?,Dex=?,Con=?,Wis=?,Cha=?,Intel=?";
        let tq = mysql.exec_drop(
            query,
            self,
        );
        let err = tq.await;
        match err {
            Err(e) => {
                log::info!("error inserting character {}", e);
            }
            _ => {
                log::info!("chracter insertion is fine");
            }
        }
        Ok(())
    }

    /// Retrieve characters for user account from database
    pub async fn retrieve_chars(account_name: &String, mysql: &mut mysql_async::Conn) -> Result<Vec<crate::character::Character>, crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = crate::character::Character::QUERY;
        log::info!("Checking for account {}", account_name);
        let s = mysql.prep(query).await?;
        let asdf = mysql.exec_map(
            s,
            (
                account_name.clone(),
            ),
            |a: Character| {
                a
            },
        ).await?;
        Ok(asdf)
    }

    /// Calculate initial mp for a character with class and wisdom
    fn calc_initial_mp(class: u8, wisdom: u8) -> u16 {
        match class {
            0 => {
                match wisdom {
                    11 => 2,
                    12..=15 => 3,
                    16..=18 => 4,
                    _ => 2,
                }
            }
            1 => {
                2
            }
            2 => {
                3
            }
            3 => {
                4
            }
            4 => {
                5
            }
            5 => {
                6
            }
            6 => {
                7
            }
            _ => panic!("Invalid class"),
        }
    }

    /// Roll a new character
    pub fn new(account_name: String, id: u32, name: String, class: u8, gender: u8, str: u8, dex: u8, con: u8, wis: u8, cha: u8, int: u8) -> Option<Self> {
        if !Self::valid_name(&name) {
            return None;
        }
        Some(Self {
            account_name,
            name,
            pledge: "".to_string(),
            id,
            alignment: 0,
            level: 1,
            class,
            gender,
            hp_max: match class {
                0 => 14,
                1 => 16,
                2 => 15,
                3 => 12,
                4 => 12,
                5 => 15,
                6 => 15,
                _ => panic!(),
            },
            mp_max: Self::calc_initial_mp(class, wis),
            ac: 10,
            strength: str,
            dexterity: dex,
            constitution: con,
            wisdom: wis,
            charisma: cha,
            intelligence: int,
        })
    }
}

impl Into<Params> for &Character {
    fn into(self) -> Params {
        let mut p = Vec::new();
        p.push(self.account_name.clone().into());
        p.push(self.id.into());
        p.push(self.name.clone().into());
        p.push(self.level.into());
        p.push(self.hp_max.into());
        p.push(self.mp_max.into());
        p.push(self.class.into());
        p.push(self.gender.into());
        p.push(self.ac.into());
        p.push(self.strength.into());
        p.push(self.dexterity.into());
        p.push(self.constitution.into());
        p.push(self.wisdom.into());
        p.push(self.charisma.into());
        p.push(self.intelligence.into());
        Params::Positional(p)
    }
}

impl mysql_async::prelude::FromRow for Character {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
        where
            Self: Sized {
        Ok(Self {
            account_name: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            id: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            alignment: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            level: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            pledge: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            class: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            gender: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            hp_max: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            mp_max: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            ac: row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
            strength: row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?,
            dexterity: row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
            constitution: row.get(13).ok_or(mysql_async::FromRowError(row.clone()))?,
            wisdom: row.get(14).ok_or(mysql_async::FromRowError(row.clone()))?,
            charisma: row.get(15).ok_or(mysql_async::FromRowError(row.clone()))?,
            intelligence: row.get(16).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}