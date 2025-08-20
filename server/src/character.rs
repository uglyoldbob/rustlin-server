//! Code for representing characters played by users

use std::{collections::HashMap, convert::TryInto};

use common::packet::{ServerPacket, ServerPacketSender};
use mysql::{prelude::Queryable, Params};

use crate::{
    server::ClientError,
    world::{
        item::{ItemInstanceWithoutDefinition, ItemUsage},
        object::ObjectList,
        Map, WorldObjectId, WorldResponse,
    },
};

/// Represents a complete playable character in the game
#[derive(Debug)]
pub struct FullCharacter {
    /// The account name for the character
    account_name: String,
    /// The name of the character
    pub name: String,
    /// The access level of the character, 200 = Admin, 100 = monitor
    access_level: u16,
    /// The id of the character in the database
    id: u32,
    /// The world id of the character
    world_id: super::world::WorldObjectId,
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
    items: HashMap<u32, crate::world::item::ItemInstance>,
    /// The known objects for the character
    known_objects: ObjectList,
    /// How to send messages to the async task for this character
    sender: Option<tokio::sync::mpsc::Sender<crate::world::WorldResponse>>,
}

/// Represents a partial playable character in the game
#[derive(Debug)]
pub struct PartialCharacter {
    /// The account name for the character
    account_name: String,
    /// The name of the character
    pub name: String,
    /// The access level of the character, 200 = Admin, 100 = monitor
    access_level: u16,
    /// The world id of the character
    world_id: super::world::WorldObjectId,
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
    items: HashMap<u32, crate::world::item::ItemInstanceWithoutDefinition>,
}

impl PartialCharacter {
    /// Convert into a full character, returning a FullCharacter
    pub fn into_full(self, item_table: &HashMap<u32, crate::world::item::Item>) -> FullCharacter {
        let mut items = HashMap::new();
        for (k, i) in self.items.into_iter() {
            if let Some(i) = i.populate_item_definition(item_table) {
                items.insert(k, i);
            }
        }
        FullCharacter {
            account_name: self.account_name,
            name: self.name,
            access_level: self.access_level,
            id: self.id,
            world_id: self.world_id,
            alignment: self.alignment,
            level: self.level,
            pledge: self.pledge,
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
            details: self.details,
            items,
            known_objects: ObjectList::new(),
            sender: None,
        }
    }
}

impl crate::world::object::ObjectTrait for FullCharacter {
    fn get_location(&self) -> crate::character::Location {
        self.details.location
    }

    fn get_prev_location(&self) -> crate::character::Location {
        self.details.old_location.unwrap_or(self.details.location)
    }

    fn can_shutdown(&self) -> bool {
        self.access_level == 200
    }

    fn set_location(&mut self, l: crate::character::Location) {
        self.details.old_location = Some(self.details.location);
        self.details.location = l;
    }

    fn id(&self) -> super::world::WorldObjectId {
        self.world_id
    }

    fn add_object(&mut self, o: WorldObjectId) {
        self.known_objects.add_object(o);
    }

    fn remove_object(&mut self, o: crate::world::WorldObjectId) {
        self.known_objects.remove_object(o);
    }

    fn get_known_objects(&self) -> Option<&ObjectList> {
        Some(&self.known_objects)
    }

    fn player_name(&self) -> Option<String> {
        Some(self.name.clone())
    }

    fn get_items(&self) -> Option<&HashMap<u32, crate::world::item::ItemInstance>> {
        Some(&self.items)
    }

    fn items_mut(&mut self) -> Option<&mut HashMap<u32, crate::world::item::ItemInstance>> {
        Some(&mut self.items)
    }

    fn sender(&self) -> Option<tokio::sync::mpsc::Sender<crate::world::WorldResponse>> {
        self.sender.clone()
    }

    fn build_put_object_packet(&self) -> common::packet::ServerPacket {
        ServerPacket::PutObject {
            x: self.details.location.x,
            y: self.details.location.y,
            id: self.world_id.get_u32(),
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
    }
}

impl FullCharacter {
    /// Get a reference to the location of the character
    pub fn location_ref(&self) -> &Location {
        &self.details.location
    }

    pub fn add_sender(&mut self, s: tokio::sync::mpsc::Sender<crate::world::WorldResponse>) {
        self.sender = Some(s);
    }

    /// Use the specified item
    pub fn use_item(
        &mut self,
        id: &u32,
        p2: &mut crate::world::ItemUseData,
        map: &Map,
    ) -> Result<(), ClientError> {
        if let Some(item) = self.items.get_mut(id) {
            ////TODO check to see if item delay in effect for the item being used, return None if it is
            ////TODO Check to see if there is a delay timer in effect for the item being used
            if crate::world::item::ItemUsage::None == item.usage() {
                p2.packets.push(ServerPacket::Message {
                    ty: 74,
                    msgs: vec![item.name()],
                });
                return Ok(());
            }
            // Or when the map you are on disallows item usage
            if !map.can_use_items() {
                p2.packets.push(ServerPacket::Message {
                    ty: 563,
                    msgs: vec![],
                });
                return Ok(());
            }
            let mut poly_name = None;
            let mut item_target = None;
            match item.usage() {
                ItemUsage::Polymorph => {
                    poly_name = Some(p2.p.pull_string());
                }
                ItemUsage::EnchantArmorScroll
                | ItemUsage::EnchantWeaponScroll
                | ItemUsage::IdentifyScroll
                | ItemUsage::WithTarget => {
                    item_target = Some(p2.p.pull_u32());
                }
                ItemUsage::Normal => {
                    //item 41048..=41057 glued logbook page is a pull u32
                }
                ItemUsage::None => {
                    //item 40956,40957 probably need to be fixed in db to item_type choice, it is a pull_u32
                    //items 41255..=41259 probably need to be fixed in db, they are a pull_u8, pull_u8
                }
                ItemUsage::TeleportScroll1 | ItemUsage::BlessedTeleport => {
                    item_target = Some(p2.p.pull_u32());
                }
                ItemUsage::BlankScroll => {
                    let spell_id = p2.p.pull_u8();
                }
                ItemUsage::SpellBuff => {
                    //item 40870 and 40879 change to choice in db?
                    let spell_id = p2.p.pull_u32();
                }
                ItemUsage::WandNearbyTarget | ItemUsage::WandWithTarget => {
                    let spell_id = p2.p.pull_u32();
                    let spell_x = p2.p.pull_u16();
                    let spell_y = p2.p.pull_u16();
                }
                ItemUsage::ResurrectScroll => {
                    let id = p2.p.pull_u32();
                }
                ItemUsage::Letter
                | ItemUsage::Letter2
                | ItemUsage::ChristmasCard
                | ItemUsage::ChristmasCard
                | ItemUsage::ValentinesCard
                | ItemUsage::ValentinesCard2
                | ItemUsage::WhiteDayCard
                | ItemUsage::WhiteDayCard2 => {
                    let code = p2.p.pull_u16();
                    let receiver = p2.p.pull_string();
                    let body = p2.p.pull_u8();
                }
                ItemUsage::FishingRod => {
                    let fishx = p2.p.pull_u16();
                    let fishy = p2.p.pull_u16();
                }
                ItemUsage::Armor => {
                    log::info!("Need to check armor equipping");
                    item.toggle_equip();
                    {
                        let packet: ServerPacket = ServerPacket::InventoryMod(item.update_packet());
                        p2.packets.push(packet);
                        p2.packets.push(item.update_description_packet());
                    }
                }
                ItemUsage::Weapon => {
                    log::info!("Need to check weapon equipping");
                    item.toggle_equip();
                    {
                        let packet: ServerPacket = ServerPacket::InventoryMod(item.update_packet());
                        p2.packets.push(packet);
                        //p2.packets.push(item.update_description_packet());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Get the current hp of the player
    pub fn curr_hp(&self) -> u16 {
        self.details.curr_hp
    }

    /// Send all items the player has to the user
    pub async fn send_all_items(
        &self,
        s: &mut tokio::sync::mpsc::Sender<WorldResponse>,
    ) -> Result<(), crate::server::ClientError> {
        let mut elements = Vec::new();
        {
            for i in self.items.values() {
                elements.push(i.inventory_element());
            }
        }
        s.send(WorldResponse::ServerPacket(
            common::packet::ServerPacket::InventoryVec(elements),
        )).await;
        Ok(())
    }

    /// Get a mutable reference to the location of the character
    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.details.location
    }

    /// Get the details packet for sending to the user
    pub fn details_packet(&self) -> ServerPacket {
        ServerPacket::CharacterDetails {
            id: self.world_id.get_u32(),
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
            id: self.world_id.get_u32(),
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
#[derive(Copy, Clone, Debug, PartialEq)]
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

impl Location {
    /// Compute the location for a move packet
    pub fn compute_for_move(&self) -> Self {
        match self.direction {
            0 => Self {
                x: self.x,
                y: self.y + 1,
                map: self.map,
                direction: self.direction,
            },
            1 => Self {
                x: self.x - 1,
                y: self.y + 1,
                map: self.map,
                direction: self.direction,
            },
            2 => Self {
                x: self.x - 1,
                y: self.y,
                map: self.map,
                direction: self.direction,
            },
            3 => Self {
                x: self.x - 1,
                y: self.y - 1,
                map: self.map,
                direction: self.direction,
            },
            4 => Self {
                x: self.x,
                y: self.y - 1,
                map: self.map,
                direction: self.direction,
            },
            5 => Self {
                x: self.x + 1,
                y: self.y - 1,
                map: self.map,
                direction: self.direction,
            },
            6 => Self {
                x: self.x + 1,
                y: self.y,
                map: self.map,
                direction: self.direction,
            },
            7 => Self {
                x: self.x + 1,
                y: self.y + 1,
                map: self.map,
                direction: self.direction,
            },
            _ => *self,
        }
    }

    /// Get the linear distance between the location of this object and the specified location (as the crow flies).
    /// This assumes the objects are already on the same map
    pub fn linear_distance(&self, l2: &Self) -> f32 {
        let deltax = self.x.abs_diff(l2.x);
        let deltay = self.y.abs_diff(l2.y);
        let sum = ((deltax as u32) * (deltax as u32) + (deltay as u32) * (deltay as u32)) as f32;
        sum.sqrt()
    }

    /// Calculates the manhattan distance between two map points
    pub fn manhattan_distance(&self, l2: &Self) -> u16 {
        let d1 = u16::abs_diff(self.x, l2.x);
        let d2 = u16::abs_diff(self.y, l2.y);
        d1 + d2
    }
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
    /// The last place the npc was
    old_location: Option<crate::character::Location>,
}

impl mysql::prelude::FromRow for ExtraCharacterDetails {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            exp: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            curr_hp: row.get(1).ok_or(mysql::FromRowError(row.clone()))?,
            curr_mp: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            time: row.get(3).ok_or(mysql::FromRowError(row.clone()))?,
            food: row.get(4).ok_or(mysql::FromRowError(row.clone()))?,
            weight: row.get(5).ok_or(mysql::FromRowError(row.clone()))?,
            fire_resist: row.get(6).ok_or(mysql::FromRowError(row.clone()))?,
            water_resist: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            wind_resist: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            earth_resist: row.get(9).ok_or(mysql::FromRowError(row.clone()))?,
            location: Location {
                x: row.get(10).ok_or(mysql::FromRowError(row.clone()))?,
                y: row.get(11).ok_or(mysql::FromRowError(row.clone()))?,
                map: row.get(12).ok_or(mysql::FromRowError(row.clone()))?,
                direction: 5,
            },
            old_location: None,
        })
    }
}

/// Represents a playable character in the game
#[derive(Debug)]
pub struct Character {
    /// The account name for the character
    account_name: String,
    /// The access level of the character, 200 = Admin, 100 = monitor
    access_level: u16,
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
    /// A query for selecting characters in a player account
    pub const QUERY: &str = "SELECT account_name, char_name, objid, Lawful, level, Clanname, Class, Sex, MaxHp, MaxMp, Ac, Str, Dex, Con, Wis, Cha, Intel, AccessLevel from characters WHERE account_name=?";

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
    fn get_items(
        &self,
        mysql: &mut mysql::PooledConn,
    ) -> Result<Vec<ItemInstanceWithoutDefinition>, crate::server::ClientError> {
        let query = "SELECT * from character_items WHERE char_id=?";
        let s = mysql.prep(query)?;
        let params = Params::Positional(vec![self.id.into()]);
        let details = mysql.exec_map(s, params, |a: ItemInstanceWithoutDefinition| a)?;
        Ok(details)
    }

    /// Retrieve all gameplay details of the character from the database, some of the elements need to be looked up to finish the character.
    pub fn get_partial_details(
        &self,
        new_id: super::world::WorldObjectId,
        mysql: &mut mysql::PooledConn,
    ) -> Result<PartialCharacter, crate::server::ClientError> {
        use mysql::prelude::Queryable;
        let query = "SELECT Exp, CurHp, CurMp, 1, Food, 32, 1, 2, 3, 4, LocX, LocY, MapID from characters WHERE account_name=? and char_name=?";
        log::info!(
            "Checking for account {} -  player {}",
            self.account_name,
            self.name
        );
        let s = mysql.prep(query)?;
        let details = mysql.exec_map(
            s,
            (&self.account_name, &self.name),
            |a: ExtraCharacterDetails| a,
        )?;
        let details = details[0];
        let items = self.get_items(mysql)?;
        let mut item_map = HashMap::new();
        for i in items {
            item_map.insert(i.id(), i);
        }
        Ok(PartialCharacter {
            account_name: self.account_name.clone(),
            name: self.name.clone(),
            access_level: self.access_level,
            id: self.id,
            world_id: new_id,
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
            items: item_map,
        })
    }

    /// Save a new character into the database, updating the id of the character to a new valid id
    pub fn save_new_to_db(
        &mut self,
        mysql: &mut mysql::PooledConn,
    ) -> Result<(), crate::server::ClientError> {
        use mysql::prelude::Queryable;
        let mut t = mysql.start_transaction(mysql::TxOpts::default())?;
        let id = crate::world::World::get_new_id(&mut t)?;
        if let Some(id) = id {
            self.id = id;
        } else {
            self.id = 2;
        }
        let query = "INSERT INTO characters SET account_name=?,objid=?,char_name=?,level=?,MaxHp=?,MaxMp=?,Class=?,Sex=?,Ac=?,Str=?,Dex=?,Con=?,Wis=?,Cha=?,Intel=?";
        t.exec_drop(query, self)?;
        t.commit()?;
        Ok(())
    }

    /// Delete the character from the database
    pub fn delete_char(&self, mysql: &mut mysql::PooledConn) -> Result<(), mysql::Error> {
        let query = "DELETE FROM characters WHERE account_name=? AND char_name=?";
        mysql.exec_drop(query, (self.account_name.clone(), self.name.clone()))?;
        Ok(())
    }

    /// Retrieve characters for user account from database
    pub fn retrieve_chars(
        account_name: &String,
        mysql: &mut mysql::PooledConn,
    ) -> Result<Vec<crate::character::Character>, crate::server::ClientError> {
        use mysql::prelude::Queryable;
        let query = crate::character::Character::QUERY;
        log::info!("Checking for account {}", account_name);
        let s = mysql.prep(query)?;
        let asdf = mysql.exec_map(s, (account_name.clone(),), |a: Character| a)?;
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
            access_level: 0,
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

impl From<&mut Character> for Params {
    fn from(value: &mut Character) -> Self {
        let p = vec![
            value.account_name.clone().into(),
            value.id.into(),
            value.name.clone().into(),
            value.level.into(),
            value.hp_max.into(),
            value.mp_max.into(),
            (value.class as u16).into(),
            value.gender.into(),
            value.ac.into(),
            value.strength.into(),
            value.dexterity.into(),
            value.constitution.into(),
            value.wisdom.into(),
            value.charisma.into(),
            value.intelligence.into(),
        ];
        Params::Positional(p)
    }
}

impl mysql::prelude::FromRow for Character {
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        let c: u16 = row.get(6).ok_or(mysql::FromRowError(row.clone()))?;
        Ok(Self {
            account_name: row.get(0).ok_or(mysql::FromRowError(row.clone()))?,
            name: row.get(1).ok_or(mysql::FromRowError(row.clone()))?,
            id: row.get(2).ok_or(mysql::FromRowError(row.clone()))?,
            alignment: row.get(3).ok_or(mysql::FromRowError(row.clone()))?,
            level: row.get(4).ok_or(mysql::FromRowError(row.clone()))?,
            pledge: row.get(5).ok_or(mysql::FromRowError(row.clone()))?,
            class: c.try_into().map_err(|_| mysql::FromRowError(row.clone()))?,
            gender: row.get(7).ok_or(mysql::FromRowError(row.clone()))?,
            hp_max: row.get(8).ok_or(mysql::FromRowError(row.clone()))?,
            mp_max: row.get(9).ok_or(mysql::FromRowError(row.clone()))?,
            ac: row.get(10).ok_or(mysql::FromRowError(row.clone()))?,
            strength: row.get(11).ok_or(mysql::FromRowError(row.clone()))?,
            dexterity: row.get(12).ok_or(mysql::FromRowError(row.clone()))?,
            constitution: row.get(13).ok_or(mysql::FromRowError(row.clone()))?,
            wisdom: row.get(14).ok_or(mysql::FromRowError(row.clone()))?,
            charisma: row.get(15).ok_or(mysql::FromRowError(row.clone()))?,
            intelligence: row.get(16).ok_or(mysql::FromRowError(row.clone()))?,
            access_level: row.get(17).ok_or(mysql::FromRowError(row.clone()))?,
        })
    }
}
