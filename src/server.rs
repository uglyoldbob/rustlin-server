use futures::FutureExt;
use std::error::Error;
use tokio::net::TcpListener;

use std::fmt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use std::convert::TryInto;
use std::vec::Vec;

use rand::Rng;

use crate::client_data::*;
use crate::user::*;
use crate::ClientMessage;
use crate::ServerMessage;

/// The 'ClientPacket' type. Represents packets sent by the client
enum ClientPacket {
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
enum ServerPacket {
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
    fn build(self) -> Packet {
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

fn change_key(k: u64, v: u32) -> u64 {
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

fn key_init(k: u32) -> u64 {
    let key: u32 = 0x930fd7e2;
    let rotr: u32 = k ^ 0x9c30d539;
    let big0: u32 = rotr >> 13 | rotr << 19;
    let big1: u32 = big0 ^ key ^ 0x7c72e993;
    let mut keyvec2 = big0.to_be_bytes().to_vec();
    let mut keyvec = big1.to_be_bytes().to_vec();
    keyvec.append(&mut keyvec2);
    u64::from_ne_bytes(keyvec.try_into().unwrap())
}

struct Packet {
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
    fn new() -> Packet {
        Packet {
            data: Vec::new(),
            read: 0,
        }
    }
    fn convert(mut self) -> ClientPacket {
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
    fn add_vec(&mut self, d: &Vec<u8>) -> &mut Packet {
        let mut copy = d.clone();
        self.data.append(&mut copy);
        self
    }
    fn add_u8(&mut self, d: u8) -> &mut Packet {
        self.data.push(d);
        self
    }
    fn add_i8(&mut self, d: i8) -> &mut Packet {
        let a: U8Converter = U8Converter { i: d };
        let a: u8 = unsafe { a.u };
        self.data.push(a);
        self
    }
    fn add_u16(&mut self, d: u16) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }
	fn add_i16(&mut self, d: i16) -> &mut Packet {
		let a: U16Converter = U16Converter { i: d };
		let a: u16 = unsafe { a.u };
		self.add_u16(a);
		self
	}
    fn add_u32(&mut self, d: u32) -> &mut Packet {
        self.data.append(&mut d.to_le_bytes().to_vec());
        self
    }
    fn add_string(&mut self, d: String) -> &mut Packet {
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

struct ServerPacketReceiver {
    reader: tokio::net::tcp::OwnedReadHalf,
    decryption_key: u64,
}

impl ServerPacketReceiver {
    fn new(r: tokio::net::tcp::OwnedReadHalf, key: u32) -> ServerPacketReceiver {
        ServerPacketReceiver {
            reader: r,
            decryption_key: key_init(key),
        }
    }

    fn get_key(&self) -> u64 {
        self.decryption_key
    }

    async fn read_packet(&mut self) -> Result<Packet, ClientError> {
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

struct ServerPacketSender {
    writer: tokio::net::tcp::OwnedWriteHalf,
    encryption_key: Option<u64>,
}

impl ServerPacketSender {
    fn new(w: tokio::net::tcp::OwnedWriteHalf) -> ServerPacketSender {
        ServerPacketSender {
            writer: w,
            encryption_key: None,
        }
    }

    fn set_encryption_key(&mut self, d: u64) {
        self.encryption_key = Some(d);
    }

    async fn send_packet(&mut self, mut data: Packet) -> Result<(), ClientError> {
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

#[derive(Debug, Clone)]
struct ClientError;
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client error")
    }
}

impl From<std::io::Error> for ClientError {
    fn from(_: std::io::Error) -> ClientError {
        ClientError {}
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for ClientError {
    fn from(_: tokio::sync::broadcast::error::RecvError) -> ClientError {
        ClientError {}
    }
}

impl From<tokio::sync::mpsc::error::SendError<ClientMessage>> for ClientError {
    fn from(_: tokio::sync::mpsc::error::SendError<ClientMessage>) -> ClientError {
        ClientError {}
    }
}

impl From<mysql_async::Error> for ClientError {
    fn from(_: mysql_async::Error) -> ClientError {
        ClientError {}
    }
}

async fn process_packet(
    p: Packet,
    s: &mut ServerPacketSender,
    server_tx: &tokio::sync::mpsc::Sender<ClientMessage>,
    id: u32,
    mysql: &mut mysql_async::Conn,
) -> Result<(), ClientError> {
    let c = p.convert();
    Ok(match c {
        ClientPacket::Version(a, b, c, d) => {
            println!("client: version {} {} {} {}", a, b, c, d);
            let response: Packet = ServerPacket::ServerVersion {
                id: 2,
                version: 2,
                time: 3,
                new_accounts: 1,
                english: 1,
                country: 0,
            }
            .build();
            s.send_packet(response).await?;
        }
        ClientPacket::Login(u, p, v1, v2, v3, v4, v5, v6, v7) => {
            println!(
                "client: login attempt for {} {} {} {} {} {} {} {}",
                &u, v1, v2, v3, v4, v5, v6, v7
            );
            let user = get_user_details(u.clone(), mysql).await;
            match user {
                Some(us) => {
                    println!("User {} exists", u.clone());
                    us.print();
                    //TODO un-hardcode the salt for the password hashing
					let password_success = us.check_login("lineage".to_string(), p);
					println!("User pw test {}", hash_password("testtest".to_string(), "lineage".to_string(), "password".to_string()));
					println!("User login check is {}", password_success);
					if password_success {
						s.send_packet(ServerPacket::LoginResult { code: 0 }.build()).await?;
						s.send_packet(ServerPacket::News("This is the news".to_string()).build()).await?;
						let _ = &server_tx
							.send(ClientMessage::LoggedIn(id, u)).await?;
					}
					else
					{
						s.send_packet(ServerPacket::LoginResult { code: 8 }.build()).await?;
					}
                }
                None => {
                    println!("User {} does not exist!", u.clone());
                    //TODO actually determine if auto account creation is enabled
                    if true {
                        //TODO: put in accurate ip information
                        //TODO un-hardcode the salt for the password hashing
                        let newaccount = UserAccount::new(u.clone(), p, "127.0.0.1".to_string(), "lineage".to_string());
                        newaccount.insert_into_db(mysql).await;
                        s.send_packet(ServerPacket::LoginResult { code: 0 }.build()).await?;
						s.send_packet(ServerPacket::News("This is the news".to_string()).build()).await?;
						let _ = &server_tx
							.send(ClientMessage::LoggedIn(id, u.clone())).await?;
                    }
                    else
                    {
                        s.send_packet(ServerPacket::LoginResult { code: 8 }.build()).await?;
                    }
                }
            }
        }
        ClientPacket::NewsDone => {
            //send number of characters the player has
            let mut response = ServerPacket::NumberCharacters(1, 8).build();
            s.send_packet(response).await?;

            for _ in 0..1 {
                response = ServerPacket::LoginCharacterDetails {
                    name: "whatever".to_string(),
                    pledge: "whocares".to_string(),
                    ctype: 1,
                    gender: 2,
                    alignment: 32767,
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
                .build();
                s.send_packet(response).await?;
            }
        }
		ClientPacket::NewCharacter{name, class, gender, strength,
			dexterity, constitution, wisdom, charisma, intelligence} => {
			let _ = &server_tx
                .send(ClientMessage::NewCharacter { id: id,
					name: name,
					class: class,
					gender: gender,
					strength: strength,
					dexterity: dexterity,
					constitution: constitution,
					wisdom: wisdom,
					charisma: charisma,
					intelligence: intelligence,
					})
                .await?;
		}
		ClientPacket::DeleteCharacter(n) => {
			let _ = &server_tx.send(ClientMessage::DeleteCharacter{id: id, name: n}).await?;
			//TODO determine if character level is 30 or higher
			//TODO send DeleteCharacterWait if level is 30 or higher
			s.send_packet(ServerPacket::DeleteCharacterOk.build()).await?;
		}
        ClientPacket::CharacterSelect { name } => {
            println!("client: login with {}", name);
            let mut response = ServerPacket::StartGame(0).build();
            s.send_packet(response).await?;

            response = ServerPacket::CharacterDetails {
                id: 1,
                level: 5,
                xp: 1234,
                strength: 12,
                dexterity: 12,
                constitution: 13,
                wisdom: 14,
                charisma: 15,
                intelligence: 16,
                curr_hp: 123,
                max_hp: 985,
                curr_mp: 34,
                max_mp: 345,
                time: 1,
                ac: -13,
                food: 1.0,
                weight: 0.5,
                alignment: 32675,
                fire_resist: 0,
                water_resist: 1,
                wind_resist: 2,
                earth_resist: 3,
            }
            .build();
            s.send_packet(response).await?;

            s.send_packet(ServerPacket::MapId(4, 0).build()).await?;

            s.send_packet(
                ServerPacket::PutObject {
                    x: 33430,
                    y: 32815,
                    id: 1,
                    icon: 1,
                    status: 0,
                    direction: 0,
                    light: 5,
                    speed: 50,
                    xp: 1234,
                    alignment: 32767,
                    name: "testing".to_string(),
                    title: "i am groot".to_string(),
                    status2: 0,
                    pledgeid: 0,
                    pledgename: "avengers".to_string(),
                    unknown: "".to_string(),
                    v1: 0,
                    hp_bar: 100,
                    v2: 0,
                    v3: 0,
                }
                .build(),
            )
            .await?;

            s.send_packet(ServerPacket::CharSpMrBonus { sp: 0, mr: 0 }.build())
                .await?;

            s.send_packet(ServerPacket::Weather(0).build()).await?;

            //TODO send owncharstatus packet
        }
        ClientPacket::KeepAlive => {}
        ClientPacket::GameInitDone => {}
        ClientPacket::WindowActivate(v2) => {
            println!("Client window activate {}", v2);
        }
        ClientPacket::Save => {}
        ClientPacket::Move { x, y, heading } => {
            println!("client: moving to {} {} {}", x, y, heading);
        }
        ClientPacket::ChangeDirection(d) => {
            println!("client: change direction to {}", d);
        }
        ClientPacket::Chat(m) => {
            let _ = &server_tx
                .send(ClientMessage::RegularChat { id: id, msg: m })
                .await?;
        }
        ClientPacket::YellChat(m) => {
            //TODO put in the correct coordinates for yelling
			let _ = &server_tx.send(ClientMessage::YellChat{
                id: id, msg: m, x: 32768, y: 32768,
            }).await?;
        }
        ClientPacket::PartyChat(m) => {
            let _ = &server_tx.send(ClientMessage::PartyChat(id, m)).await?;
        }
        ClientPacket::PledgeChat(m) => {
            let _ = &server_tx.send(ClientMessage::PledgeChat(id,m)).await?;
        }
        ClientPacket::WhisperChat(n, m) => {
            let _ = &server_tx.send(ClientMessage::WhisperChat(id, n, m)).await?;
        }
        ClientPacket::GlobalChat(m) => {
            let _ = &server_tx.send(ClientMessage::GlobalChat(id, m)).await?;
        }
        ClientPacket::CommandChat(m) => {
            println!("client: command chat {}", m);
            let mut words = m.split_whitespace();
            let first_word = words.next();
            if let Some(m) = first_word {
                match m {
                    "asdf" => {
                        println!("A command called asdf");
                    }
					"quit" => {
						s.send_packet(ServerPacket::Disconnect.build()).await?;
					}
                    "chat" => {
                        s.send_packet(
                            ServerPacket::SystemMessage(
                                "This is a test of the system message".to_string(),
                            )
                            .build(),
                        )
                        .await?;
                        s.send_packet(ServerPacket::NpcShout("NPC Shout test".to_string()).build())
                            .await?;

                        s.send_packet(
                            ServerPacket::RegularChat {
                                id: 0,
                                msg: "regular chat".to_string(),
                            }
                            .build(),
                        )
                        .await?;
                        s.send_packet(
                            ServerPacket::YellChat {
                                id: 0,
                                msg: "yelling".to_string(),
                                x: 32768,
                                y: 32768,
                            }
                            .build(),
                        )
                        .await?;
                        s.send_packet(ServerPacket::GlobalChat("global chat".to_string()).build())
                            .await?;
                        s.send_packet(ServerPacket::PledgeChat("pledge chat".to_string()).build())
                            .await?;
                        s.send_packet(ServerPacket::PartyChat("party chat".to_string()).build())
                            .await?;
                        s.send_packet(
                            ServerPacket::WhisperChat {
                                name: "test".to_string(),
                                msg: "whisper message".to_string(),
                            }
                            .build(),
                        )
                        .await?;
                    }
                    _ => {
                        println!("An unknown command {}", m);
                    }
                }
            }
        }
        ClientPacket::SpecialCommandChat(m) => {
            println!("client: special command chat {}", m);
        }
		ClientPacket::ChangePassword {account,oldpass,newpass} => {
			let user = get_user_details(account.clone(), mysql).await;
            match user {
                Some(us) => {
                    println!("User {} exists", account);
                    us.print();
					let password_success = us.check_login("lineage".to_string(), oldpass);
					println!("User login check is {}", password_success);
					if password_success {
						println!("User wants to change password and entered correct details");
						s.send_packet(ServerPacket::LoginResult { code: 0x30 }.build()).await?;
					}
					else
					{
						let mut p = Packet::new();
						s.send_packet(ServerPacket::LoginResult { code: 8 }.build()).await?;
					}
				}
				_ => {
					let mut p = Packet::new();
						s.send_packet(ServerPacket::LoginResult { code: 8 }.build()).await?;
				}
			}
		}
        ClientPacket::Unknown(d) => {
            println!("client: received unknown packet {:x?}", d);
        }
    })
}

async fn handle_server_message(p: ServerMessage,
	packet_writer: &mut ServerPacketSender, )  -> Result<u8, ClientError> {
	match p {
		ServerMessage::AssignId(i) => {
		}
		ServerMessage::Disconnect => {
			packet_writer.send_packet(ServerPacket::Disconnect.build()).await?;
		}
		ServerMessage::SystemMessage(m) => {
			packet_writer.send_packet(ServerPacket::SystemMessage(m).build()).await?;
		}
		ServerMessage::NpcShout(m) => {
			packet_writer.send_packet(ServerPacket::NpcShout(m).build()).await?;
		}
		ServerMessage::RegularChat{id, msg} => {
			packet_writer.send_packet(ServerPacket::RegularChat{id: id, msg: msg}.build()).await?;
		}
        ServerMessage::WhisperChat(name, msg) => {
            packet_writer.send_packet(
                ServerPacket::WhisperChat {
                    name: name,
                    msg: msg,
                }
                .build(),
            ).await?;
        }
		ServerMessage::YellChat{id, msg, x, y} => {
			packet_writer.send_packet(ServerPacket::YellChat{id: id, msg: msg, x: x, y: y}.build()).await?;
		}
		ServerMessage::GlobalChat(m) => {
			packet_writer.send_packet(ServerPacket::GlobalChat(m).build()).await?;
		}
		ServerMessage::PledgeChat(m) => {
			packet_writer.send_packet(ServerPacket::PledgeChat(m).build()).await?;
		}
		ServerMessage::PartyChat(m) => {
			packet_writer.send_packet(ServerPacket::PartyChat(m).build()).await?;
		}
		ServerMessage::CharacterCreateStatus(v) => {
			match v {
				0 => {
					packet_writer.send_packet(ServerPacket::CharacterCreationStatus(2).build()).await?;
				}
				1 => {
					packet_writer.send_packet(ServerPacket::CharacterCreationStatus(9).build()).await?;
				}
				2 => {
					packet_writer.send_packet(ServerPacket::CharacterCreationStatus(6).build()).await?;
				}
				3 => {
					packet_writer.send_packet(ServerPacket::CharacterCreationStatus(21).build()).await?;
				}
				_ => {
					println!("wrong char creation status");
				}
			}
		}
		ServerMessage::NewCharacterDetails{ name,
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
			packet_writer.send_packet(ServerPacket::NewCharacterDetails{
				name: name,
				pledge: pledge,
				class: class,
				gender: gender,
				alignment: alignment,
				hp: hp,
				mp: mp,
				ac: ac,
				level: level,
				strength: strength,
				dexterity: dexterity,
				constitution: constitution,
				wisdom: wisdom,
				charisma: charisma,
				intelligence: intelligence,
				}.build()).await?;
		}
	}
	Ok(0)
}

async fn client_event_loop(
    mut packet_writer: ServerPacketSender,
    mut brd_rx: tokio::sync::broadcast::Receiver<ServerMessage>,
    reader: tokio::net::tcp::OwnedReadHalf,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
    server_tx: &tokio::sync::mpsc::Sender<ClientMessage>,
    id: u32,
    mut mysql: mysql_async::Conn,
) -> Result<u8, ClientError> {
    let encryption_key: u32 = rand::thread_rng().gen();
    let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

    let mut key_packet = Packet::new();
    key_packet.add_u8(65).add_u32(encryption_key);
    packet_writer.send_packet(key_packet).await?;
    packet_writer.set_encryption_key(packet_reader.get_key());
    loop {
        futures::select! {
            packet = packet_reader.read_packet().fuse() => {
                let p = packet?;
                process_packet(p, &mut packet_writer, server_tx, id, &mut mysql).await?;
            }
            msg = brd_rx.recv().fuse() => {
                let p = msg.unwrap();
                handle_server_message(p, &mut packet_writer).await?;
            }
            msg = rx.recv().fuse() => {
                match msg {
                    None => {}
                    Some(p) => {handle_server_message(p, &mut packet_writer).await?;}
                }
            }
        }
    }
    //TODO send disconnect packet if applicable
    Ok(0)
}

async fn process_client(socket: tokio::net::TcpStream, cd: ClientData) -> Result<u8, ClientError> {
    let (reader, writer) = socket.into_split();
    let packet_writer = ServerPacketSender::new(writer);

    let brd_rx: tokio::sync::broadcast::Receiver<ServerMessage> = cd.get_broadcast_rx();
    let server_tx = &cd.server_tx;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let _ = &server_tx.send(ClientMessage::Register(tx)).await?;

    let client_id: u32;

    let mysql = cd.get_mysql().await?;

    println!("client: Waiting to receive the id");
    loop {
        let msg = rx.recv().await;
        match msg.unwrap() {
            ServerMessage::AssignId(i) => {
                println!("client: assigned id of {} to self", i);
                client_id = i;
                break;
            }
            _ => {}
        }
    }

    if let Err(_) = client_event_loop(
        packet_writer,
        brd_rx,
        reader,
        rx,
        &server_tx,
        client_id,
        mysql,
    )
    .await
    {
        println!("test: Client errored");
    }

    server_tx.send(ClientMessage::Unregister(client_id)).await?;

    Ok(0)
}

pub async fn setup_game_server(
    cd: ClientData,
) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    println!("server: Starting the game server");
    let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = update_listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    println!("server: Received a client from {}", addr);
                    let cd2 = cd.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process_client(socket, cd2).await {
                            println!("server: Client {} errored {}", addr, e);
                        }
                    });
                }
                _ = (&mut update_rx) => {
                    println!("server: Received a message to shut down");
                    break;
                }
            }
        }
        println!("update: Ending the server thread!");
    });

    Ok(update_tx)
}
