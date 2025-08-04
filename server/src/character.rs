use common::packet::ServerPacket;

/// Represents a playable character in the game
#[derive(Debug)]
pub struct Character {
    /// The account name for the character
    account_name: String,
    /// The name of the character
    name: String,
    /// The id of the character in the database
    id: u32,
    /// The alignmnet of the character
    alignment: i16,
}

pub type CharacterRowData = (String, String, u32, i16);

impl Character {
    /// Is the player name valid?
    pub fn valid_name(n: String) -> bool {
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
            pledge: "whocares".to_string(),
            ctype: 1,
            gender: 2,
            alignment: self.alignment,
            hp: 1234,
            mp: 95,
            ac: -12,
            level: 51,
            strength: 12,
            dexterity: 12,
            constitution: 12,
            wisdom: 12,
            charisma: 12,
            intelligence: 12,
        }
    }

    /// Save a new character into the database
    pub async fn save_new_to_db(&self, mysql: &mut mysql_async::Conn) -> Result<(), crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = "INSERT INTO characters SET account_name=?,objid=?,char_name=?";
        let tq = mysql.exec_drop(
            query,
            (
                self.account_name.clone(),
                self.id,
                self.name.clone(),
            ),
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

    /// Roll a new character
    pub fn new(account_name: String, id: u32, name: String, class: u8, gender: u8, str: u8, dex: u8, con: u8, wis: u8, cha: u8, int: u8) -> Option<Self> {
        Some(Self {
            account_name,
            name,
            id,
            alignment: 0,
        })
    }
}

impl From<CharacterRowData> for Character {
    fn from(value: CharacterRowData) -> Self {
        Self {
            account_name: value.0,
            name: value.1,
            id: value.2,
            alignment: value.3,
        }
    }
}

impl mysql_async::prelude::FromRow for Character {
    fn from_row(row: mysql_async::Row) -> Self
        where
            Self: Sized, {
        Self {
            account_name: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            id: row.get(2).unwrap(),
            alignment: row.get(3).unwrap(),
        }
    }

    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
        where
            Self: Sized {
        Ok(Self {
            account_name: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            id: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            alignment: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
        })
    }
}