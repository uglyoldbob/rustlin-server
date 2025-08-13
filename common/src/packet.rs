use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use std::convert::TryInto;
use std::vec::Vec;

#[derive(Debug)]
pub enum PacketError {
    IoError,
}

impl From<std::io::Error> for PacketError {
    fn from(_: std::io::Error) -> PacketError {
        PacketError::IoError
    }
}

/// The 'ClientPacket' type. Represents packets sent by the client
#[derive(Debug)]
pub enum ClientPacket {
    Version(u16, u32, u8, u32),
    Login(String, String, u32, u32, u32, u32, u32, u32, u32),
    CharacterSelect {
        name: String,
    },
    /// The player is done "reading" the news
    NewsDone,
    KeepAlive,
    GameInitDone,
    WindowActivate(u8),
    Save,
    MoveFrom {
        x: u16,
        y: u16,
        heading: u8,
    },
    ChangeDirection(u8),
    Chat(String),
    YellChat(String),
    PartyChat(String),
    PledgeChat(String),
    WhisperChat(String, String),
    GlobalChat(String),
    /// chat messages starting with - in the client
    CommandChat(String),
    /// chat message starting with . in the client
    SpecialCommandChat(String),
    ChangePassword {
        account: String,
        oldpass: String,
        newpass: String,
    },
    NewCharacter {
        name: String,
        class: u8,
        gender: u8,
        strength: u8,
        dexterity: u8,
        constitution: u8,
        wisdom: u8,
        charisma: u8,
        intelligence: u8,
    },
    DeleteCharacter(String),
    /// The user wants to create a bookmark
    CreateBookmark(String),
    /// The user used the /who command
    WhoCommand(String),
    /// The user wants to add a friend
    AddFriend(String),
    /// The user wants to remove a friend
    RemoveFriend(String),
    /// A ping from a user
    Ping(u8),
    /// The player wants to restart with another character
    Restart,
    /// The player is using an item
    UseItem {
        /// Item id of item being used
        id: u32,
        /// The rest of the packet
        remainder: Vec<u8>,
    },
    Unknown(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct InventoryElement {
    /// Item id
    pub id: u32,
    /// Item type
    pub i_type: i8,
    /// usage?
    pub n_use: u8,
    /// icon
    pub icon: i16,
    /// bless status
    pub blessing: ItemBlessing,
    /// item count
    pub count: u32,
    /// identified
    pub identified: u8,
    /// description, $numeric references use a string from the stringtable of the game client, as long as the $ is not the first character
    pub description: String,
    /// extended description.
    /// # opcodes
    /// * 1 - Hit value small+?/large+?, small: u8, large: u8, material: u8, weight: u32, if followed by 2, the next u8 is the + value for large and small
    /// * 3 - Damage d, d: u8
    /// * 4 - Two handed Weapon
    /// * 5 - Hit bonus +d, d: u8
    /// * 6 - Damage bonus +d, d: u8
    /// * 7 - Usable: bitmask: bitmask: u8 - 1 = Prince/Princess, 2 = Knight, 4 = Elf, 8 = Wizard, 16 = Dark Elf, 32 = Dragon Knight, 64 = Illusionist, 128 = High Pet, 127 = All Classes
    /// * 8 - STR +d: d: u8
    /// * 9 - DEX +d: d: u8
    /// * 10 - CON +d: d: u8
    /// * 11 - WIS +d: d: u8
    /// * 12 - INT +d: d: u8
    /// * 13 - CHA +d: d: u8
    /// * 14 - Maximum HP +d: d: u16
    /// * 15 - Magic Defense +d: d: u16
    /// * 16 - Mana Absorption
    /// * 17 - Spell Power +d: d: u8
    /// * 18 - Maintains haste state when held
    /// * 19 - AC ac+?: ac: u8, grade: u8, material: u8, weight: u32, if followed by 2, the next u8 is the + value (grade: high = 0, medium = 1, low = 2)
    /// * 20 - Luck +d: d: u8
    /// * 21 - Nutrition - nutrition: nitrition: u16, material: u8, weight: u32
    /// * 22 - Lightness d: d: u16, material: u8, weight: u32
    /// * 23 - Nothing : material: u8, weight: u32
    /// * 24 - Bow hit bonus +d: d: u8
    /// * 25 - Class $d: d: u16, unsure of valid values
    /// * 26 - Level d: d: u16
    /// * 27 - Fire Elemental d: d: u8
    /// * 28 - Water Elemental d: d: u8
    /// * 29 - Wind Elementa d: d: u8
    /// * 30 - Earth Elemental d: d: u8
    /// * 31 - Maximum HP d: d: i16
    /// * 32 - Maximum MP +d: d: i8
    /// * 33 - Modifies magic defense with v, must come after a opcode 15: v: u8 (1 = Freeze Resistance, 2 = Petrify Resistance, 3 = Sleep Resistance, 4 = Darkness Resistance, 5 = Stun Resistance, 6 = Hold Resistance, 7 = None)
    /// * 34 - Life Suction
    /// * 35 - Bow Damage +d: d: u8
    /// * 36 - dummy for branch d: d: u8
    /// * 37 - Healing Rate d: d: u8
    /// * 38 - Mana Healing Rate d: d: u8
    /// * 39 - plain string: null terminated string
    /// * 40 - nothing?: unknown: u8
    /// * others: immediately finish processing
    /// # Materials 1-22
    /// * 1 - Liquid
    /// * 2 - Web
    /// * 3 - Vegetation
    /// * 4 - Animal Matter
    /// * 5 - Paper
    /// * 6 - Cloth
    /// * 7 - Leather
    /// * 8 - Wood
    /// * 9 - Bone
    /// * 10 - Dragon Scale
    /// * 11 - Iron
    /// * 12 - Metal
    /// * 13 - Copper
    /// * 14 - Silver
    /// * 15 - Gold
    /// * 16 - Platinum
    /// * 17 - Mithril
    /// * 18 - Black Mithril
    /// * 19 - Glass
    /// * 20 - Gemstone
    /// * 21 - Mineral
    /// * 22 - Oriharukon
    pub ed: Vec<u8>,
}

impl InventoryElement {
    fn add(&self, p: &mut Packet) {
        p.add_u32(self.id)
            .add_i8(self.i_type)
            .add_u8(self.n_use)
            .add_i16(self.icon)
            .add_u8(self.blessing as u8)
            .add_u32(self.count)
            .add_u8(self.identified)
            .add_string(&self.description)
            .add_u8(self.ed.len() as u8);
        if self.ed.len() > 0 {
            p.add_vec(&self.ed);
        }
    }
}

/// USed to create a inventory update packet
#[derive(Clone, Debug)]
pub struct InventoryUpdate {
    /// Item id
    pub id: u32,
    /// description
    pub description: String,
    /// count
    pub count: u32,
    /// See ed from InventoryElement
    pub ed: Vec<u8>,
}

//TODO create enums for the option values

/// Represents packets sent to the client, from the server
#[derive(Clone, Debug)]
pub enum ServerPacket {
    ServerVersion {
        id: u8,
        version: u32,
        time: u32,
        new_accounts: u8,
        english: u8,
        country: u8,
    },
    Disconnect,
    LoginResult {
        code: u8,
    },
    News(String),
    CharacterCreationStatus(u8),
    NewCharacterDetails {
        name: String,
        pledge: String,
        class: u8,
        gender: u8,
        alignment: i16,
        hp: u16,
        mp: u16,
        ac: i8,
        level: u8,
        strength: u8,
        dexterity: u8,
        constitution: u8,
        wisdom: u8,
        charisma: u8,
        intelligence: u8,
    },
    DeleteCharacterOk,
    DeleteCharacterWait,
    NumberCharacters(u8, u8),
    LoginCharacterDetails {
        name: String,
        pledge: String,
        /// The character type
        ctype: u8,
        gender: u8,
        alignment: i16,
        hp: u16,
        mp: u16,
        ac: i8,
        level: u8,
        strength: u8,
        dexterity: u8,
        constitution: u8,
        wisdom: u8,
        charisma: u8,
        intelligence: u8,
    },
    StartGame(u32),
    CharacterDetails {
        id: u32,
        level: u8,
        xp: u32,
        strength: u8,
        dexterity: u8,
        constitution: u8,
        wisdom: u8,
        charisma: u8,
        intelligence: u8,
        curr_hp: u16,
        max_hp: u16,
        curr_mp: u16,
        max_mp: u16,
        ac: i8,
        time: u32,
        food: f32,
        weight: f32,
        alignment: i16,
        fire_resist: u8,
        water_resist: u8,
        wind_resist: u8,
        earth_resist: u8,
    },
    MapId(u16, u8),
    /// put a new object on the user map
    PutObject {
        x: u16,
        y: u16,
        id: u32,
        icon: u16,
        status: u8,
        direction: u8,
        light: u8,
        speed: u8,
        xp: u32,
        alignment: i16,
        name: String,
        title: String,
        status2: u8,
        pledgeid: u32,
        pledgename: String,
        owner_name: String,
        /// Upper 4 bits is pledge rank, lower 4 bits is altitude
        v1: u8,
        /// Represents the hp percentage 0-100, 255 means no bar
        hp_bar: u8,
        /// 8 = drunken, not 8 = speed becomes 2
        v2: u8,
        level: u8,
    },
    /// Move an object
    MoveObject {
        id: u32,
        x: u16,
        y: u16,
        direction: u8,
    },
    CharSpMrBonus {
        sp: u8,
        mr: u8,
    },
    Weather(u8),
    SystemMessage(String),
    NpcShout(String),
    RegularChat {
        id: u32,
        ///msg = "player name: message"
        msg: String,
    },
    YellChat {
        id: u32,
        msg: String,
        x: u16,
        y: u16,
    },
    WhisperChat {
        name: String,
        ///msg = "<player name> message"
        msg: String,
    },
    GlobalChat(String),
    ///msg = "[player name] message"
    PledgeChat(String),
    PartyChat(String),
    /// Adds a bunch of inventory items
    InventoryVec(Vec<InventoryElement>),
    /// Add an inventory item
    Inventory(InventoryElement),
    /// Modify an existing inventory item
    InventoryMod(InventoryUpdate),
    /// Update item description
    InventoryDescriptionUpdate {
        /// The item id to update
        id: u32,
        /// The new item description
        description: String,
    },
    /// change direction packet
    ChangeDirection {
        /// The id of the userobject
        id: u32,
        /// The direction for the userobject
        direction: u8,
    },
    /// Updates title, alignment, speed, and polymorph, resulting object seems to not move
    CloneObject {
        /// The id of the userobject
        id: u32,
        /// new object speed
        speed: u32,
        /// the poly id?
        poly_id: u16,
        /// alignment
        alignment: i16,
        /// the polymorph action
        poly_action: u8,
        /// title
        title: String,
    },
    /// Sets the criminal count for an object
    SetCriminalCount {
        /// The id of the userobject
        id: u32,
        /// The count
        count: u8,
    },
    /// Remove an object
    RemoveObject(u32),
    /// Send the player back to the character select screen
    BackToCharacterSelect,
    /// A message presented to the user
    Message {
        /// message type
        ty: u16,
        /// message strings
        msgs: Vec<String>,
    },
}

/// Potential bless status for an item
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum ItemBlessing {
    /// The item is blessed
    Blessed = 0,
    /// The item is normal
    Normal = 1,
    /// The item is cursed
    Cursed = 2,
    /// The item is unknown
    Unidentified = 3,
}

impl ServerPacket {
    /// Build a `Packet` from the ServerPacket
    pub fn build(self) -> Packet {
        let mut p = Packet::new();
        match self {
            ServerPacket::Message { ty, msgs } => {
                p.add_u8(87).add_u16(ty).add_u8(msgs.len() as u8);
                for m in msgs {
                    p.add_string(&m);
                }
            }
            ServerPacket::MoveObject {
                id,
                x,
                y,
                direction,
            } => {
                p.add_u8(61)
                    .add_u32(id)
                    .add_u16(x)
                    .add_u16(y)
                    .add_u8(direction);
            }
            ServerPacket::BackToCharacterSelect => {
                p.add_u8(107).add_u8(42);
            }
            ServerPacket::RemoveObject(id) => {
                p.add_u8(9).add_u32(id);
            }
            ServerPacket::SetCriminalCount { id, count } => {
                p.add_u8(121).add_u32(id).add_u8(count);
            }
            ServerPacket::CloneObject {
                id,
                speed,
                poly_id,
                alignment,
                poly_action: poly_arg,
                title,
            } => {
                p.add_u8(126)
                    .add_u32(id)
                    .add_u32(speed)
                    .add_u16(poly_id)
                    .add_i16(alignment)
                    .add_u8(poly_arg)
                    .add_string(&title);
            }
            ServerPacket::ServerVersion {
                id,
                version,
                time,
                new_accounts,
                english,
                country,
            } => {
                p.add_u8(10)
                    .add_u8(0)
                    .add_u8(id)
                    .add_u32(version)
                    .add_u32(version)
                    .add_u32(version)
                    .add_u32(version)
                    .add_u32(time)
                    .add_u8(new_accounts)
                    .add_u8(english)
                    .add_u8(country);
            }
            //TODO sometimes the client crashes when sending this packet, after they click ok
            ServerPacket::Disconnect => {
                p.add_u8(18).add_u16(500).add_u32(0);
            }
            ServerPacket::LoginResult { code } => {
                p.add_u8(21).add_u8(code).add_u32(0);
            }
            ServerPacket::News(news) => {
                p.add_u8(90).add_string(&news);
            }
            //TODO verify this
            ServerPacket::CharacterCreationStatus(v) => {
                p.add_u8(106).add_u8(v).add_u32(0).add_u32(0);
            }
            ServerPacket::NewCharacterDetails {
                name,
                pledge,
                class,
                gender,
                alignment,
                hp,
                mp,
                ac,
                level,
                strength,
                dexterity,
                constitution,
                wisdom,
                charisma,
                intelligence,
            } => {
                p.add_u8(98)
                    .add_string(&name)
                    .add_string(&pledge)
                    .add_u8(class)
                    .add_u8(gender)
                    .add_i16(alignment)
                    .add_u16(hp)
                    .add_u16(mp)
                    .add_i8(ac)
                    .add_u8(level)
                    .add_u8(strength)
                    .add_u8(dexterity)
                    .add_u8(constitution)
                    .add_u8(wisdom)
                    .add_u8(charisma)
                    .add_u8(intelligence)
                    .add_u8(1)
                    .add_u8(2)
                    .add_u32(3);
            }
            ServerPacket::DeleteCharacterOk => {
                p.add_u8(33).add_u8(0x05);
            }
            ServerPacket::DeleteCharacterWait => {
                p.add_u8(33).add_u8(0x51);
            }
            ServerPacket::NumberCharacters(num, max) => {
                p.add_u8(113)
                    .add_u8(num) //number of characters
                    .add_u8(max); //number of slots
            }
            ServerPacket::LoginCharacterDetails {
                name,
                pledge,
                ctype,
                gender,
                alignment,
                hp,
                mp,
                ac,
                level,
                strength,
                dexterity,
                constitution,
                wisdom,
                charisma,
                intelligence,
            } => {
                p.add_u8(99)
                    .add_string(&name)
                    .add_string(&pledge)
                    .add_u8(ctype)
                    .add_u8(gender)
                    .add_i16(alignment)
                    .add_u16(hp)
                    .add_u16(mp)
                    .add_i8(ac)
                    .add_u8(level)
                    .add_u8(strength)
                    .add_u8(dexterity)
                    .add_u8(constitution)
                    .add_u8(wisdom)
                    .add_u8(charisma)
                    .add_u8(intelligence)
                    .add_u8(0) //isBuilder
                    .add_u32(0); //birthday
            }
            ServerPacket::CharacterDetails {
                id,
                level,
                xp,
                strength,
                dexterity,
                constitution,
                wisdom,
                charisma,
                intelligence,
                curr_hp,
                max_hp,
                curr_mp,
                max_mp,
                ac,
                time,
                food,
                weight,
                alignment,
                fire_resist,
                water_resist,
                wind_resist,
                earth_resist,
            } => {
                p.add_u8(69)
                    .add_u32(id)
                    .add_u8(level)
                    .add_u32(xp)
                    .add_u8(strength)
                    .add_u8(intelligence)
                    .add_u8(wisdom)
                    .add_u8(dexterity)
                    .add_u8(constitution)
                    .add_u8(charisma)
                    .add_u16(curr_hp)
                    .add_u16(max_hp)
                    .add_u16(curr_mp)
                    .add_u16(max_mp)
                    .add_i8(ac)
                    .add_u32(time)
                    .add_u8((food * 29.0).round() as u8)
                    .add_u8((weight * 29.0).round() as u8)
                    .add_i16(alignment)
                    .add_u8(fire_resist)
                    .add_u8(water_resist)
                    .add_u8(wind_resist)
                    .add_u8(earth_resist);
            }
            ServerPacket::StartGame(i) => {
                p.add_u8(63).add_u8(3).add_u32(i);
            }
            ServerPacket::MapId(map, underwater) => {
                p.add_u8(76).add_u16(map).add_u8(underwater);
            }
            ServerPacket::PutObject {
                x,
                y,
                id,
                icon,
                status,
                direction,
                light,
                speed,
                xp,
                alignment,
                name,
                title,
                status2,
                pledgeid,
                pledgename,
                owner_name,
                v1,
                hp_bar,
                v2,
                level: v3,
            } => {
                p.add_u8(64)
                    .add_u16(x)
                    .add_u16(y)
                    .add_u32(id)
                    .add_u16(icon)
                    .add_u8(status)
                    .add_u8(direction)
                    .add_u8(light)
                    .add_u8(speed)
                    .add_u32(xp)
                    .add_i16(alignment)
                    .add_string(&name)
                    .add_string(&title)
                    .add_u8(status2)
                    .add_u32(pledgeid)
                    .add_string(&pledgename)
                    .add_string(&owner_name)
                    .add_u8(v1)
                    .add_u8(hp_bar)
                    .add_u8(v2)
                    .add_u8(v3);
            }
            ServerPacket::CharSpMrBonus { sp, mr } => {
                p.add_u8(80).add_u8(sp).add_u8(mr);
            }
            ServerPacket::Weather(w) => {
                p.add_u8(83).add_u8(w);
            }
            ServerPacket::SystemMessage(m) => {
                p.add_u8(105).add_u8(9).add_string(&m);
            }
            ServerPacket::NpcShout(m) => {
                p.add_u8(42)
                    .add_u8(2)
                    .add_u32(0)
                    .add_string(&m)
                    .add_u16(1)
                    .add_u16(2);
            }
            ServerPacket::RegularChat { id, msg } => {
                p.add_u8(8).add_u8(0).add_u32(id).add_string(&msg);
            }
            ServerPacket::YellChat { id, msg, x, y } => {
                p.add_u8(8)
                    .add_u8(2)
                    .add_u32(id)
                    .add_string(&msg)
                    .add_u16(x)
                    .add_u16(y);
            }
            ServerPacket::GlobalChat(msg) => {
                p.add_u8(105).add_u8(3).add_string(&msg);
            }
            ServerPacket::PledgeChat(msg) => {
                p.add_u8(105).add_u8(4).add_string(&msg);
            }
            ServerPacket::PartyChat(msg) => {
                p.add_u8(105).add_u8(11).add_string(&msg);
            }
            ServerPacket::WhisperChat { name, msg } => {
                p.add_u8(91).add_string(&name).add_string(&msg);
            }
            ServerPacket::InventoryVec(v) => {
                p.add_u8(47);
                p.add_u8(v.len() as u8);
                for e in v {
                    e.add(&mut p);
                }
            }
            ServerPacket::Inventory(e) => {
                p.add_u8(6);
                e.add(&mut p);
            }
            ServerPacket::ChangeDirection { id, direction } => {
                p.add_u32(id).add_u8(direction);
            }
            ServerPacket::InventoryMod(m) => {
                p.add_u8(43)
                    .add_u32(m.id)
                    .add_string(&m.description)
                    .add_u32(m.count)
                    .add_u8(m.ed.len() as u8);
                if !m.ed.is_empty() {
                    p.add_vec(&m.ed);
                }
            }
            ServerPacket::InventoryDescriptionUpdate { id, description } => {
                p.add_u8(29).add_u32(id).add_string(&description);
            }
        }
        p
    }
}

/// Change the key to the next key with the given data
pub fn change_key(k: u64, v: u32) -> u64 {
    let d: u32 = u32::from_be(v);
    let mut little: u32 = u32::from_be((k & 0xFFFFFFFF).try_into().unwrap());
    let mut little64 = little as u64;

    little64 += 0x287effc3;
    little64 &= 0xffffffff;

    little = little64.try_into().unwrap();
    little = u32::from_be(little);
    let mut nk = (k ^ ((d as u64) << 32)) & 0xFFFFFFFF00000000;
    nk |= little as u64;
    nk
}

/// Initialize an encryption key based on an initial seed
pub fn key_init(k: u32) -> u64 {
    let key: u32 = 0x930fd7e2;
    let rotr: u32 = k ^ 0x9c30d539;
    let big0: u32 = rotr >> 13 | rotr << 19;
    let big1: u32 = big0 ^ key ^ 0x7c72e993;
    let mut keyvec2 = big0.to_be_bytes().to_vec();
    let mut keyvec = big1.to_be_bytes().to_vec();
    keyvec.append(&mut keyvec2);
    u64::from_ne_bytes(keyvec.try_into().unwrap())
}

/// A packet of data that can be sent or received over a network connection
#[derive(Clone, Debug)]
pub struct Packet {
    data: Vec<u8>,
    read: usize,
}

/// Used for converting u8 to i8 and vice versa
union U8Converter {
    u: u8,
    i: i8,
}

/// Used for converting u16 to i16 and vice versa
union U16Converter {
    u: u16,
    i: i16,
}

impl Packet {
    /// Construct an empty packet
    pub fn new() -> Packet {
        Packet {
            data: Vec::new(),
            read: 0,
        }
    }

    /// Construct a raw packet from a vector
    pub fn raw_packet(d: Vec<u8>) -> Self {
        Packet { data: d, read: 0 }
    }

    /// Convert the packet to a `ClientPacket`
    pub fn convert(mut self) -> ClientPacket {
        let opcode: u8 = self.pull_u8();
        match opcode {
            1 => ClientPacket::UseItem {
                id: self.pull_u32(),
                remainder: self.pull_remainder(),
            },
            12 => ClientPacket::Login(
                self.pull_string(),
                self.pull_string(),
                self.pull_u32(),
                self.pull_u32(),
                self.pull_u32(),
                self.pull_u32(),
                self.pull_u32(),
                self.pull_u32(),
                self.pull_u32(),
            ),
            13 => ClientPacket::WhisperChat(self.pull_string(), self.pull_string()),
            20 => ClientPacket::CreateBookmark(self.pull_string()),
            30 => ClientPacket::RemoveFriend(self.pull_string()),
            34 => ClientPacket::DeleteCharacter(self.pull_string()),
            40 => {
                self.pull_u8();
                ClientPacket::GlobalChat(self.pull_string())
            }
            43 => ClientPacket::NewsDone,
            47 => ClientPacket::Restart,
            57 => ClientPacket::KeepAlive,
            71 => {
                let val1: u16 = self.pull_u16();
                let val2: u32 = self.pull_u32();
                let val3: u8 = self.pull_u8();
                let val4: u32 = self.pull_u32();
                log::info!("client: found a client version packet");
                ClientPacket::Version(val1, val2, val3, val4)
            }
            72 => ClientPacket::NewCharacter {
                name: self.pull_string(),
                class: self.pull_u8(),
                gender: self.pull_u8(),
                strength: self.pull_u8(),
                dexterity: self.pull_u8(),
                constitution: self.pull_u8(),
                wisdom: self.pull_u8(),
                charisma: self.pull_u8(),
                intelligence: self.pull_u8(),
            },
            74 => ClientPacket::ChangeDirection(self.pull_u8()),
            79 => ClientPacket::AddFriend(self.pull_string()),
            83 => ClientPacket::CharacterSelect {
                name: self.pull_string(),
            },
            88 => ClientPacket::MoveFrom {
                x: self.pull_u16(),
                y: self.pull_u16(),
                heading: self.pull_u8(),
            },
            92 => ClientPacket::GameInitDone,
            97 => {
                self.pull_u8();
                let v2 = self.pull_u8();
                ClientPacket::WindowActivate(v2)
            }
            100 => ClientPacket::ChangePassword {
                account: self.pull_string(),
                oldpass: self.pull_string(),
                newpass: self.pull_string(),
            },
            104 => {
                let t = self.pull_u8();
                let m = self.pull_string();
                match t {
                    0 => match m.chars().take(1).last().unwrap() {
                        '!' => ClientPacket::YellChat(m[1..].to_string()),
                        '-' => ClientPacket::CommandChat(m[1..].to_string()),
                        '.' => ClientPacket::SpecialCommandChat(m[1..].to_string()),
                        _ => ClientPacket::Chat(m),
                    },
                    4 => ClientPacket::PledgeChat(m),        //@
                    11 => ClientPacket::PartyChat(m),        //#
                    13 => ClientPacket::Unknown(self.buf()), //%
                    15 => ClientPacket::Unknown(self.buf()), //~
                    _ => ClientPacket::Unknown(self.buf()),
                }
            }
            111 => ClientPacket::Save,
            112 => ClientPacket::Ping(self.pull_u8()),
            119 => ClientPacket::WhoCommand(self.pull_string()),
            _ => ClientPacket::Unknown(self.buf()),
        }
    }

    /// Get the length of data currently in the packet
    fn len(&self) -> u16 {
        self.data.len().try_into().unwrap()
    }

    /// Get a copy of the raw packet data
    fn buf(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    /// Write a vector of bytes to the packet
    pub fn add_vec(&mut self, d: &[u8]) -> &mut Packet {
        let mut copy = d.to_vec();
        self.data.append(&mut copy);
        self
    }

    /// Write a u8 to the packet
    pub fn add_u8(&mut self, d: u8) -> &mut Packet {
        self.data.push(d);
        self
    }

    /// Write an i8 to the packet
    pub fn add_i8(&mut self, d: i8) -> &mut Packet {
        let a: U8Converter = U8Converter { i: d };
        let a: u8 = unsafe { a.u };
        self.data.push(a);
        self
    }

    /// Write a u16 to the packet
    pub fn add_u16(&mut self, d: u16) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }

    /// Write an i16 to the packet
    pub fn add_i16(&mut self, d: i16) -> &mut Packet {
        let a: U16Converter = U16Converter { i: d };
        let a: u16 = unsafe { a.u };
        self.add_u16(a);
        self
    }

    /// Write a u32 to the packet
    pub fn add_u32(&mut self, d: u32) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }

    /// Write a string to the packet
    pub fn add_string(&mut self, d: &str) -> &mut Packet {
        for n in d.bytes() {
            self.add_u8(n);
        }
        self.add_u8(0);
        self
    }

    /// Fetch the rest of the packet as a vector
    fn pull_remainder(&mut self) -> Vec<u8> {
        let v = self.data[self.read..].to_vec();
        self.read = self.data.len();
        v
    }

    /// Fetch a u8 from the packet
    pub fn pull_u8(&mut self) -> u8 {
        let val: u8 = self.data[self.read];
        self.read += 1;
        val
    }

    /// Fetch a u16 from the packet
    pub fn pull_u16(&mut self) -> u16 {
        let mut val: u16 = (self.data[self.read + 1] as u16) << 8;
        val |= self.data[self.read] as u16;
        self.read += 2;
        val
    }

    /// Fetch a u32 from the packet
    pub fn pull_u32(&mut self) -> u32 {
        let mut val: u32 = self.data[self.read + 3] as u32;
        val = (val << 8) | (self.data[self.read + 2] as u32);
        val = (val << 8) | (self.data[self.read + 1] as u32);
        val = (val << 8) | (self.data[self.read] as u32);
        self.read += 4;
        val
    }

    /// Fetch a string from the packet
    pub fn pull_string(&mut self) -> String {
        let mut v: String = "".to_string();
        //a do while loop
        while {
            let digit = self.pull_u8();
            if digit != 0 {
                v.push(digit as char);
            }
            digit != 0
        } {}
        v
    }

    /// Peek into the packet
    fn peek_u32(&self) -> u32 {
        let v = Vec::from([self.data[0], self.data[1], self.data[2], self.data[3]]);
        u32::from_ne_bytes(v.try_into().unwrap())
    }

    /// Encrypt data into a sendable packet
    fn encrypt(&mut self, key: u64) -> &mut Packet {
        let key_vec = key.to_be_bytes().to_vec();
        self.data[0] ^= key_vec[0];
        for i in 1..self.data.len() {
            self.data[i] ^= key_vec[i & 7] ^ self.data[i - 1];
        }
        self.data[3] ^= key_vec[2];
        self.data[2] ^= key_vec[3] ^ self.data[3];
        self.data[1] ^= key_vec[4] ^ self.data[2];
        self.data[0] ^= key_vec[5] ^ self.data[1];
        self
    }

    /// Decrypt data into a packet
    fn decrypt(&mut self, key: u64) -> &mut Packet {
        let key_vec = key.to_be_bytes().to_vec();
        let b3: u8 = self.data[3];
        self.data[3] ^= key_vec[2];

        let b2: u8 = self.data[2];
        self.data[2] ^= b3 ^ key_vec[3];

        let b1: u8 = self.data[1];
        self.data[1] ^= b2 ^ key_vec[4];

        let mut k: u8 = self.data[0] ^ b1 ^ key_vec[5];
        self.data[0] = k ^ key_vec[0];

        for i in 1..self.data.len() {
            let t: u8 = self.data[i];
            self.data[i] ^= key_vec[i & 7] ^ k;
            k = t;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests encrypting and decrypting together
    #[test]
    fn encrypt_decrypt() {
        let mut rng = rand::thread_rng();
        for packet_length in 4..50 {
            for _ in 1..1000 {
                let mut data: Vec<u8> = Vec::new();
                data.resize(packet_length, 0);
                for index in 1..packet_length {
                    data[index] = rand::Rng::gen(&mut rng);
                }
                let key: u64 = rand::Rng::gen(&mut rng);
                let mut start_packet = Packet::new();
                start_packet.add_vec(&data);
                start_packet.encrypt(key).decrypt(key);
                assert_eq!(data, start_packet.buf());
            }
        }
    }

    /// Tests initialization of an encryption key
    #[test]
    fn test_key_init() {
        let key_init_val = key_init(0x12345678);
        let required: u64 = 0x24700c1a554e71f5;
        assert_eq!(key_init_val, required);
    }

    /// Tests decrypting data of a known value with a known decryption key
    #[test]
    fn test_known_data_decrypt() {
        let data: Vec<u8> = Vec::from([
            0xb0, 0x9d, 0xe8, 0xde, 0x83, 0xcd, 0xbc, 0x1b, 0xd2, 0x28, 0x25, 0x3f,
        ]);
        let key: u64 = key_init(0x12345678);
        let required: Vec<u8> = Vec::from([
            0x47, 0x33, 0x00, 0xe4, 0x04, 0x00, 0x00, 0x52, 0xed, 0x8a, 0x01, 0x00,
        ]);
        let mut packet = Packet::new();
        packet.add_vec(&data);
        let d = packet.decrypt(key).buf();
        let cd = packet.peek_u32();
        assert_eq!(d, required);
        assert_eq!(cd, 0xe4003347);
        let new_key = change_key(key, cd);
        let required_new_key: u64 = 0x63430cfe184ef01d;
        assert_eq!(new_key, required_new_key);
    }

    /// Tests encrypting data of a known value with a known decryption key
    #[test]
    fn test_known_data_encrypt() {
        let required: Vec<u8> = Vec::from([
            0xb0, 0x9d, 0xe8, 0xde, 0x83, 0xcd, 0xbc, 0x1b, 0xd2, 0x28, 0x25, 0x3f,
        ]);
        let key: u64 = key_init(0x12345678);
        let data: Vec<u8> = Vec::from([
            0x47, 0x33, 0x00, 0xe4, 0x04, 0x00, 0x00, 0x52, 0xed, 0x8a, 0x01, 0x00,
        ]);
        let mut packet = Packet::new();
        packet.add_vec(&data);
        let d = packet.encrypt(key).buf();
        assert_eq!(d, required);
    }
}

/// Used for receiving packets from a user
pub struct ServerPacketReceiver {
    /// The read half of a tcp connectio to a user
    reader: tokio::net::tcp::OwnedReadHalf,
    /// The decryption key for receiving packets
    decryption_key: u64,
}

impl ServerPacketReceiver {
    /// Construct a new receiver
    pub fn new(r: tokio::net::tcp::OwnedReadHalf, key: u32) -> ServerPacketReceiver {
        ServerPacketReceiver {
            reader: r,
            decryption_key: key_init(key),
        }
    }

    /// Get the decryption key
    pub fn get_key(&self) -> u64 {
        self.decryption_key
    }

    /// Read a packet from the user
    pub async fn read_packet(&mut self) -> Result<Packet, PacketError> {
        let mut packet = Packet::new();
        let length: usize = self.reader.read_i16_le().await?.try_into().unwrap();
        let mut contents: Vec<u8> = vec![0; length - 2];
        self.reader.read_exact(&mut contents).await?;
        packet.add_vec(&contents);
        packet.decrypt(self.decryption_key);
        let kcv = packet.peek_u32();
        self.decryption_key = change_key(self.decryption_key, kcv);
        Ok(packet)
    }
}

/// Used for sending packets to a single user
pub struct ServerPacketSender {
    /// The write half of a tcp connection
    writer: tokio::net::tcp::OwnedWriteHalf,
    /// The encryption key to use for the next data to send out
    encryption_key: Option<u64>,
}

impl ServerPacketSender {
    pub fn new(w: tokio::net::tcp::OwnedWriteHalf) -> ServerPacketSender {
        ServerPacketSender {
            writer: w,
            encryption_key: None,
        }
    }

    /// Set the encryption key for outbound traffic
    pub fn set_encryption_key(&mut self, d: u64) {
        self.encryption_key = Some(d);
    }

    /// Send a packet
    pub async fn send_packet(&mut self, mut data: Packet) -> Result<(), PacketError> {
        self.writer.writable().await?;
        while data.buf().len() < 4 {
            data.add_u8(0);
        }
        let kcv = data.peek_u32();

        if let Some(key) = self.encryption_key {
            data.encrypt(key);
            self.encryption_key = Some(change_key(key, kcv));
        }
        self.writer.write_u16_le(data.len() + 2).await?;
        self.writer.write_all(&data.buf()).await?;
        Ok(())
    }
}
