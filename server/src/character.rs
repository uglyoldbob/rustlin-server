use std::{collections::HashMap, convert::TryInto};

use common::packet::{ServerPacket, ServerPacketSender};
use mysql_async::{prelude::Queryable, Params};

use crate::world::item::{ItemInstance, ItemTrait};

/// Represents a complete playable character in the game
#[derive(Clone, Debug)]
pub struct FullCharacter {
    /// The account name for the character
    account_name: String,
    /// The name of the character
    pub name: String,
    /// The id of the character in the database
    id: u32,
    /// The alignment of the character
    alignment: i16,
    /// The level of the character
    level: u8,
    /// The pledge name of the character (empty string if no pledge)
    pledge: String,
    /// The class of character
    class: Class,
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
    /// Extra details
    details: ExtraCharacterDetails,
    /// All the items the character holds
    items: Vec<crate::world::item::ItemInstance>,
}

impl crate::world::object::ObjectTrait for FullCharacter {
    fn get_location(&self) -> crate::character::Location {
        self.details.location
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn get_items(&self) -> Option<Vec<crate::world::item::ItemInstance>> {
        None
    }

    fn items_mut(&mut self) -> Option<&mut Vec<crate::world::item::ItemInstance>> {
        Some(&mut self.items)
    }

    fn build_put_object_packet(&self) -> common::packet::Packet {
        ServerPacket::PutObject {
            x: self.details.location.x,
            y: self.details.location.y,
            id: self.id,
            icon: 29,
            status: 0,
            direction: 1,
            light: 7,
            speed: 50,
            xp: self.details.exp,
            alignment: self.alignment,
            name: self.name.clone(),
            title: "".to_string(),
            status2: 0,
            pledgeid: 0,
            pledgename: self.pledge.clone(),
            owner_name: "".to_string(),
            v1: 0,
            hp_bar: 255,
            v2: 0,
            level: self.level,
        }
        .build()
    }
}

impl FullCharacter {
    /// Get a reference to the location of the character
    pub fn location_ref(&self) -> &Location {
        &self.details.location
    }

    /// Send all items the player has to the user
    pub async fn send_all_items(
        &mut self,
        w: &crate::world::World,
        packet_writer: &mut ServerPacketSender,
    ) -> Result<(), crate::server::ClientError> {
        let mut packets = Vec::new();
        {
            let item_table = w.item_table.lock().unwrap();
            for i in &mut self.items {
                if let Some(p) = i.inventory_packet(&item_table) {
                    packets.push(p);
                }
            }
        }
        for p in packets {
            packet_writer.send_packet(p).await?;
        }
        Ok(())
    }

    /// Get a mutable reference to the location of the character
    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.details.location
    }

    /// Get the details packet for sending to the user
    pub fn details_packet(&self) -> ServerPacket {
        ServerPacket::CharacterDetails {
            id: self.id,
            level: self.level,
            xp: self.details.exp,
            strength: self.strength,
            dexterity: self.strength,
            constitution: self.constitution,
            wisdom: self.wisdom,
            charisma: self.charisma,
            intelligence: self.intelligence,
            curr_hp: self.hp_max,
            max_hp: self.hp_max,
            curr_mp: self.details.curr_mp,
            max_mp: self.mp_max,
            time: 1,
            ac: self.ac,
            food: 1.0,
            weight: 0.5,
            alignment: self.alignment,
            fire_resist: self.details.fire_resist,
            water_resist: self.details.water_resist,
            wind_resist: self.details.wind_resist,
            earth_resist: self.details.earth_resist,
        }
    }

    /// Get a put object packet
    pub fn get_object_packet(&self) -> ServerPacket {
        ServerPacket::PutObject {
            x: self.details.location.x,
            y: self.details.location.y,
            id: self.id,
            icon: 1,
            status: 0,
            direction: 0,
            light: 5,
            speed: 50,
            xp: self.details.exp,
            alignment: self.alignment,
            name: self.name.clone(),
            title: "i am groot".to_string(),
            status2: 0,
            pledgeid: 0,
            pledgename: self.pledge.clone(),
            owner_name: "".to_string(),
            v1: 0,
            hp_bar: ((self.details.curr_hp as f32 / self.hp_max as f32) * 100.0) as u8,
            v2: 0,
            level: self.level,
        }
    }

    /// Get a map location packet
    pub fn get_map_packet(&self) -> ServerPacket {
        ServerPacket::MapId(self.details.location.map, 0)
    }
}

/// The location on a specific map for an object
#[derive(Copy, Clone, Debug)]
pub struct Location {
    /// The x coordinate
    pub x: u16,
    /// The y coordinate
    pub y: u16,
    /// The map id
    pub map: u16,
    /// The direction that is being faced
    pub direction: u8,
}

/// The extra details for a character to go from Character to FullCharacter
#[derive(Copy, Clone, Debug)]
pub struct ExtraCharacterDetails {
    /// Character experience
    exp: u32,
    /// Current hitpoint amount
    curr_hp: u16,
    /// Current mana point amount
    curr_mp: u16,
    /// time?
    time: u32,
    /// Food level
    food: u8,
    /// Amount of weight the player is carrying
    weight: u8,
    /// Fire resistance
    fire_resist: u8,
    /// Water resistance
    water_resist: u8,
    /// Wind resistance
    wind_resist: u8,
    /// Earth resist
    earth_resist: u8,
    /// Location
    location: Location,
}

impl mysql_async::prelude::FromRow for ExtraCharacterDetails {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            exp: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            curr_hp: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            curr_mp: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            time: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            food: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            weight: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            fire_resist: row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?,
            water_resist: row.get(7).ok_or(mysql_async::FromRowError(row.clone()))?,
            wind_resist: row.get(8).ok_or(mysql_async::FromRowError(row.clone()))?,
            earth_resist: row.get(9).ok_or(mysql_async::FromRowError(row.clone()))?,
            location: Location {
                x: row.get(10).ok_or(mysql_async::FromRowError(row.clone()))?,
                y: row.get(11).ok_or(mysql_async::FromRowError(row.clone()))?,
                map: row.get(12).ok_or(mysql_async::FromRowError(row.clone()))?,
                direction: 5,
            },
        })
    }
}

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
    class: Class,
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

/// The possible classes for a character
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Class {
    // Prince/princess
    Royal = 0,
    /// Knight
    Knight = 1,
    /// Elf
    Elf = 2,
    /// Wizard
    Wizard = 3,
    /// Dark Elf
    DarkElf = 4,
    /// Dragon Knight
    DragonKnight = 5,
    /// Illusionist
    Illusionist = 6,
}

impl Class {
    /// Get the initial hp for classes, maybe this should depend on initial constitution?
    fn initial_hp(&self, _con: u8) -> u16 {
        match self {
            Class::Royal => 14,
            Class::Knight => 16,
            Class::Elf => 15,
            Class::Wizard => 12,
            Class::DarkElf => 12,
            Class::DragonKnight => 15,
            Class::Illusionist => 15,
        }
    }

    /// Get the initial mp
    fn initial_mp(&self, wisdom: u8) -> u16 {
        match self {
            Class::Royal => match wisdom {
                12..=15 => 3,
                16..=18 => 4,
                _ => 2,
            },
            Class::Knight => match wisdom {
                12..=13 => 2,
                _ => 1,
            },
            Class::Elf => match wisdom {
                16..=18 => 6,
                _ => 4,
            },
            Class::Wizard => match wisdom {
                16..=18 => 8,
                _ => 6,
            },
            Class::DarkElf => match wisdom {
                12..=15 => 4,
                16..=18 => 6,
                _ => 3,
            },
            Class::DragonKnight => match wisdom {
                16..=18 => 6,
                _ => 4,
            },
            Class::Illusionist => match wisdom {
                16..=18 => 6,
                _ => 4,
            },
        }
    }
}

impl std::convert::TryFrom<u16> for Class {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 | 1 => Ok(Self::Royal),
            48 | 61 => Ok(Self::Knight),
            37 | 138 => Ok(Self::Elf),
            734 | 1186 => Ok(Self::Wizard),
            2786 | 2796 => Ok(Self::DarkElf),
            6658 | 6661 => Ok(Self::DragonKnight),
            6650 | 6671 => Ok(Self::Illusionist),
            _ => Err(()),
        }
    }
}

impl std::convert::TryFrom<u8> for Class {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Royal),
            1 => Ok(Self::Knight),
            2 => Ok(Self::Elf),
            3 => Ok(Self::Wizard),
            4 => Ok(Self::DarkElf),
            5 => Ok(Self::DragonKnight),
            6 => Ok(Self::Illusionist),
            _ => Err(()),
        }
    }
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

    /// Does there need to be a waiting period to delete the character?
    pub fn needs_delete_waiting(&self) -> bool {
        self.level >= 30
    }

    /// Construct a details packet for informing the user of a character they can log in with
    pub fn get_details_packet(&self) -> ServerPacket {
        ServerPacket::LoginCharacterDetails {
            name: self.name.clone(),
            pledge: self.pledge.clone(),
            ctype: self.class as u8,
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

    /// Get a new character details packet, used when creating new characters
    pub fn get_new_char_details_packet(&self) -> ServerPacket {
        ServerPacket::NewCharacterDetails {
            name: self.name().to_string(),
            pledge: self.pledge.to_string(),
            class: self.class as u8,
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

    /// Retrieve all items for the character
    async fn get_items(&self, mysql: &mut mysql_async::Conn) -> Result<Vec<ItemInstance>, crate::server::ClientError> {
        let query = "SELECT * from character_items WHERE char_id=?";
        let s = mysql.prep(query).await?;
        let params = Params::Positional(vec![self.id.into()]);
        let details = mysql
            .exec_map(
                s,
                params,
                |a: ItemInstance| a,
            )
            .await?;
        Ok(details)
    }

    /// Retrieve all gameplay details of the character
    pub async fn get_full_details(
        &self,
        mysql: &mut mysql_async::Conn,
    ) -> Result<FullCharacter, crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = "SELECT Exp, CurHp, CurMp, 1, Food, 32, 1, 2, 3, 4, LocX, LocY, MapID from characters WHERE account_name=? and char_name=?";
        log::info!(
            "Checking for account {} -  player {}",
            self.account_name,
            self.name
        );
        let s = mysql.prep(query).await?;
        let details = mysql
            .exec_map(
                s,
                (&self.account_name, &self.name),
                |a: ExtraCharacterDetails| a,
            )
            .await?;
        let details = details[0];
        let items = self.get_items(mysql).await?;
        Ok(FullCharacter {
            account_name: self.account_name.clone(),
            name: self.name.clone(),
            id: self.id,
            alignment: self.alignment,
            level: self.level,
            pledge: self.pledge.clone(),
            class: self.class,
            gender: self.gender,
            hp_max: self.hp_max,
            mp_max: self.mp_max,
            ac: self.ac,
            strength: self.strength,
            dexterity: self.dexterity,
            constitution: self.constitution,
            wisdom: self.wisdom,
            charisma: self.charisma,
            intelligence: self.intelligence,
            details,
            items,
        })
    }

    /// Save a new character into the database, updating the id of the character to a new valid id
    pub async fn save_new_to_db(
        &mut self,
        mysql: &mut mysql_async::Conn,
    ) -> Result<(), crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let mut t = mysql.start_transaction(mysql_async::TxOpts::new()).await?;
        let id = crate::world::World::get_new_id(&mut t).await?;
        if let Some(id) = id {
            self.id = id;
        } else {
            self.id = 2;
        }
        let query = "INSERT INTO characters SET account_name=?,objid=?,char_name=?,level=?,MaxHp=?,MaxMp=?,Class=?,Sex=?,Ac=?,Str=?,Dex=?,Con=?,Wis=?,Cha=?,Intel=?";
        t.exec_drop(query, self).await?;
        t.commit().await?;
        Ok(())
    }

    /// Delete the character from the database
    pub async fn delete_char(
        &self,
        mysql: &mut mysql_async::Conn,
    ) -> Result<(), mysql_async::Error> {
        let query = "DELETE FROM characters WHERE account_name=? AND char_name=?";
        mysql
            .exec_drop(query, (self.account_name.clone(), self.name.clone()))
            .await?;
        Ok(())
    }

    /// Retrieve characters for user account from database
    pub async fn retrieve_chars(
        account_name: &String,
        mysql: &mut mysql_async::Conn,
    ) -> Result<Vec<crate::character::Character>, crate::server::ClientError> {
        use mysql_async::prelude::Queryable;
        let query = crate::character::Character::QUERY;
        log::info!("Checking for account {}", account_name);
        let s = mysql.prep(query).await?;
        let asdf = mysql
            .exec_map(s, (account_name.clone(),), |a: Character| a)
            .await?;
        Ok(asdf)
    }

    /// Roll a new character
    pub fn new(
        account_name: String,
        id: u32,
        name: String,
        class: u8,
        gender: u8,
        str: u8,
        dex: u8,
        con: u8,
        wis: u8,
        cha: u8,
        int: u8,
    ) -> Option<Self> {
        if !Self::valid_name(&name) {
            return None;
        }
        let class: Class = std::convert::TryInto::try_into(class).ok()?;
        Some(Self {
            account_name,
            name,
            pledge: "".to_string(),
            id,
            alignment: 0,
            level: 1,
            class,
            gender,
            hp_max: class.initial_hp(con),
            mp_max: class.initial_mp(wis),
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

impl Into<Params> for &mut Character {
    fn into(self) -> Params {
        let mut p = Vec::new();
        p.push(self.account_name.clone().into());
        p.push(self.id.into());
        p.push(self.name.clone().into());
        p.push(self.level.into());
        p.push(self.hp_max.into());
        p.push(self.mp_max.into());
        p.push((self.class as u16).into());
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
        Self: Sized,
    {
        let c: u16 = row.get(6).ok_or(mysql_async::FromRowError(row.clone()))?;
        Ok(Self {
            account_name: row.get(0).ok_or(mysql_async::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql_async::FromRowError(row.clone()))?,
            id: row.get(2).ok_or(mysql_async::FromRowError(row.clone()))?,
            alignment: row.get(3).ok_or(mysql_async::FromRowError(row.clone()))?,
            level: row.get(4).ok_or(mysql_async::FromRowError(row.clone()))?,
            pledge: row.get(5).ok_or(mysql_async::FromRowError(row.clone()))?,
            class: c
                .try_into()
                .map_err(|_| mysql_async::FromRowError(row.clone()))?,
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
