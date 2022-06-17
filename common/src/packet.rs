use futures::FutureExt;
use std::error::Error;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use std::convert::TryInto;
use std::vec::Vec;

use rand::Rng;

pub enum PacketError {
    IoError,
}

impl From<std::io::Error> for PacketError {
    fn from(_: std::io::Error) -> PacketError {
        PacketError::IoError
    }
}


/// The 'ClientPacket' type. Represents packets sent by the client
pub enum ClientPacket {
    Version(u16, u32, u8, u32),
    Login(String, String, u32, u32, u32, u32, u32, u32, u32),
    CharacterSelect { name: String },
    NewsDone,
    KeepAlive,
    GameInitDone,
    WindowActivate(u8),
    Save,
    Move { x: u16, y: u16, heading: u8 },
    ChangeDirection(u8),
    Chat(String),
    YellChat(String),
    PartyChat(String),
    PledgeChat(String),
    WhisperChat(String, String),
    GlobalChat(String),
    CommandChat(String),
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
    Unknown(Vec<u8>),
}

//TODO create enums for the option values

/// Represents packets sent to the client, from the server
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
	NewCharacterDetails{
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
        ctype: u8,
        gender: u8,
        alignment: u16,
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
        alignment: u16,
        fire_resist: u8,
        water_resist: u8,
        wind_resist: u8,
        earth_resist: u8,
    },
    MapId(u16, u8),
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
        alignment: u16,
        name: String,
        title: String,
        status2: u8,
        pledgeid: u32,
        pledgename: String,
        unknown: String,
        v1: u8,
        hp_bar: u8,
        v2: u8,
        v3: u8,
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
}

impl ServerPacket {
    pub fn build(self) -> Packet {
        let mut p = Packet::new();
        match self {
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
                p.add_u8(90).add_string(news);
            }
			//TODO verify this
			ServerPacket::CharacterCreationStatus(v) => {
				p.add_u8(106).add_u8(v).add_u32(0).add_u32(0);
			}
			ServerPacket::NewCharacterDetails{ name,
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
				p.add_u8(98).add_string(name).add_string(pledge).add_u8(class)
				 .add_u8(gender).add_i16(alignment).add_u16(hp).add_u16(mp)
				 .add_i8(ac).add_u8(level).add_u8(strength).add_u8(dexterity)
				 .add_u8(constitution).add_u8(wisdom).add_u8(charisma)
				 .add_u8(intelligence).add_u8(1).add_u8(2).add_u32(3);
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
                    .add_string(name)
                    .add_string(pledge)
                    .add_u8(ctype)
                    .add_u8(gender)
                    .add_u16(alignment)
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
                    .add_u8(14) //TODO
                    .add_u32(15); //TODO
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
                    .add_u16(alignment)
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
                unknown,
                v1,
                hp_bar,
                v2,
                v3,
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
                    .add_u16(alignment)
                    .add_string(name)
                    .add_string(title)
                    .add_u8(status2)
                    .add_u32(pledgeid)
                    .add_string(pledgename)
                    .add_string(unknown)
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
                p.add_u8(105).add_u8(9).add_string(m);
            }
            ServerPacket::NpcShout(m) => {
                p.add_u8(42)
                    .add_u8(2)
                    .add_u32(0)
                    .add_string(m)
                    .add_u16(1)
                    .add_u16(2);
            }
            ServerPacket::RegularChat { id, msg } => {
                p.add_u8(8).add_u8(0).add_u32(id).add_string(msg);
            }
            ServerPacket::YellChat { id, msg, x, y } => {
                p.add_u8(8)
                    .add_u8(2)
                    .add_u32(id)
                    .add_string(msg)
                    .add_u16(x)
                    .add_u16(y);
            }
            ServerPacket::GlobalChat(msg) => {
                p.add_u8(105).add_u8(3).add_string(msg);
            }
            ServerPacket::PledgeChat(msg) => {
                p.add_u8(105).add_u8(4).add_string(msg);
            }
            ServerPacket::PartyChat(msg) => {
                p.add_u8(105).add_u8(11).add_string(msg);
            }
            ServerPacket::WhisperChat { name, msg } => {
                p.add_u8(91).add_string(name).add_string(msg);
            }
        }
        p
    }
}

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

pub struct Packet {
    data: Vec<u8>,
    read: usize,
}

union U8Converter {
    u: u8,
    i: i8,
}

union U16Converter {
	u: u16,
	i: i16,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            data: Vec::new(),
            read: 0,
        }
    }
    pub fn convert(mut self) -> ClientPacket {
        let opcode: u8 = self.pull_u8();
        match opcode {
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
			34 => ClientPacket::DeleteCharacter(self.pull_string()),
            40 => {
                self.pull_u8();
                ClientPacket::GlobalChat(self.pull_string())
            }
            43 => ClientPacket::NewsDone,
            57 => ClientPacket::KeepAlive,
            71 => {
                let val1: u16 = self.pull_u16();
                let val2: u32 = self.pull_u32();
                let val3: u8 = self.pull_u8();
                let val4: u32 = self.pull_u32();
                println!("client: found a client version packet");
                ClientPacket::Version(val1, val2, val3, val4)
            }
			72 => {
				ClientPacket::NewCharacter{
					name: self.pull_string(),
					class: self.pull_u8(),
					gender: self.pull_u8(),
					strength: self.pull_u8(),
					dexterity: self.pull_u8(),
					constitution: self.pull_u8(),
					wisdom: self.pull_u8(),
					charisma: self.pull_u8(),
					intelligence: self.pull_u8(),
				}
			}
            74 => ClientPacket::ChangeDirection(self.pull_u8()),
            83 => ClientPacket::CharacterSelect {
                name: self.pull_string(),
            },
            88 => ClientPacket::Move {
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
            _ => ClientPacket::Unknown(self.buf()),
        }
    }
    fn len(&self) -> u16 {
        self.data.len().try_into().unwrap()
    }
    fn buf(&self) -> Vec<u8> {
        self.data.to_vec()
    }
    pub fn add_vec(&mut self, d: &Vec<u8>) -> &mut Packet {
        let mut copy = d.clone();
        self.data.append(&mut copy);
        self
    }
    pub fn add_u8(&mut self, d: u8) -> &mut Packet {
        self.data.push(d);
        self
    }
    pub fn add_i8(&mut self, d: i8) -> &mut Packet {
        let a: U8Converter = U8Converter { i: d };
        let a: u8 = unsafe { a.u };
        self.data.push(a);
        self
    }
    pub fn add_u16(&mut self, d: u16) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }
	pub fn add_i16(&mut self, d: i16) -> &mut Packet {
		let a: U16Converter = U16Converter { i: d };
		let a: u16 = unsafe { a.u };
		self.add_u16(a);
		self
	}
    pub fn add_u32(&mut self, d: u32) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }
    pub fn add_string(&mut self, d: String) -> &mut Packet {
        for n in d.bytes() {
            self.add_u8(n);
        }
        self.add_u8(0);
        self
    }
    fn pull_u8(&mut self) -> u8 {
        let val: u8 = self.data[self.read];
        self.read += 1;
        val
    }
    fn pull_u16(&mut self) -> u16 {
        let mut val: u16 = (self.data[self.read + 1] as u16) << 8;
        val |= self.data[self.read] as u16;
        self.read += 2;
        val
    }
    fn pull_u32(&mut self) -> u32 {
        let mut val: u32 = self.data[self.read + 3] as u32;
        val = (val << 8) | (self.data[self.read + 2] as u32);
        val = (val << 8) | (self.data[self.read + 1] as u32);
        val = (val << 8) | (self.data[self.read] as u32);
        self.read += 4;
        val
    }
    fn pull_string(&mut self) -> String {
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
    fn peek_u32(&self) -> u32 {
        let v = Vec::from([self.data[0], self.data[1], self.data[2], self.data[3]]);
        u32::from_ne_bytes(v.try_into().unwrap())
    }
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

    #[test]
    fn test_key_init() {
        let key_init_val = key_init(0x12345678);
        let required: u64 = 0x24700c1a554e71f5;
        assert_eq!(key_init_val, required);
    }

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

pub struct ServerPacketReceiver {
    reader: tokio::net::tcp::OwnedReadHalf,
    decryption_key: u64,
}

impl ServerPacketReceiver {
    pub fn new(r: tokio::net::tcp::OwnedReadHalf, key: u32) -> ServerPacketReceiver {
        ServerPacketReceiver {
            reader: r,
            decryption_key: key_init(key),
        }
    }

    pub fn get_key(&self) -> u64 {
        self.decryption_key
    }

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

pub struct ServerPacketSender {
    writer: tokio::net::tcp::OwnedWriteHalf,
    encryption_key: Option<u64>,
}

impl ServerPacketSender {
    pub fn new(w: tokio::net::tcp::OwnedWriteHalf) -> ServerPacketSender {
        ServerPacketSender {
            writer: w,
            encryption_key: None,
        }
    }

    pub fn set_encryption_key(&mut self, d: u64) {
        self.encryption_key = Some(d);
    }

    pub async fn send_packet(&mut self, mut data: Packet) -> Result<(), PacketError> {
        while data.buf().len() < 4 {
            data.add_u8(0);
        }
        let kcv = data.peek_u32();

        if let Some(key) = self.encryption_key {
            data.encrypt(key);
            self.encryption_key = Some(change_key(key, kcv));
        }
        self.writer.write_u16_le(data.len() + 2).await?;
        self.writer.write(&data.buf()).await?;
        Ok(())
    }
}


