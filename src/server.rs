use futures::FutureExt;
use tokio::net::TcpListener;
use std::error::Error;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::fmt;

use std::convert::TryInto;
use std::vec::Vec;

use rand::Rng;

use crate::client_data::*;
use crate::ServerMessage;

/// The 'ClientPacket' type. Represents packets sent by the client
enum ClientPacket {
    Version(u16,u32,u8,u32),
    Login(String,String,u32,u32,u32,u32,u32,u32,u32),
	CharacterSelect{name: String},
	NewsDone,
    Unknown,
}

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
	LoginResult{ code: u8 },
	News(String),
	NumberCharacters(u8,u8),
	LoginCharacterDetails {
		name: String,
		pledge: String,
		ctype: u8,
		gender: u8,
		alignment: u16,
		hp: u16,
		mp: u16,
		ac: u8,
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
		ac: u8,
		time: u32,
		food: u8,
		weight: u8,
		alignment: u16,
		fire_resist: u8,
		water_resist: u8,
		wind_resist: u8,
		earth_resist: u8
	},
	MapId(u16, u8),
	Unknown,
	PutObject{
		x: u16,
		y: u16,
		id: u32,
		icon: u16,
		status: u8,
		direction:u8,
		light:u8,
		speed:u8,
		xp:u32,
		alignment:u16,
		name:String,
		title:String,
		status2:u8,
		pledgeid: u32,
		pledgename: String,
		unknown: String,
		v1: u8,
		hp_bar: u8,
		v2:u8,
		v3:u8,
	},
}

impl ServerPacket {
	fn build(self) -> Packet {
		let mut p = Packet::new();
		match self {
			ServerPacket::ServerVersion{id,version,time,new_accounts,english,country} => {
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
			ServerPacket::LoginResult{code} => {
				p.add_u8(21).add_u8(code).add_u32(0);
			}
			ServerPacket::News(news) => {
				p.add_u8(90).add_string(news);
			}
			ServerPacket::NumberCharacters(num,max) => {
				p.add_u8(113)
				.add_u8(num) //number of characters
				.add_u8(max); //number of slots
			}
			ServerPacket::LoginCharacterDetails{
				name, pledge, ctype, gender, alignment,
				hp,	mp,	ac,	level,	strength,
				dexterity, constitution,
				wisdom,	charisma, intelligence } => {
					p.add_u8(99)
					.add_string(name)
					.add_string(pledge)
					.add_u8(ctype)
					.add_u8(gender)
					.add_u16(alignment)
					.add_u16(hp)
					.add_u16(mp)
					.add_u8(ac)
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
			ServerPacket::CharacterDetails{
				id, level, xp, strength, dexterity,
				constitution, wisdom, charisma, intelligence,
				curr_hp, max_hp, curr_mp, max_mp, ac, time,
				food, weight, alignment, fire_resist,
				water_resist, wind_resist, earth_resist	} => {
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
					.add_u8(ac)
					.add_u32(time)
					.add_u8(food)
					.add_u8(weight)
					.add_u16(alignment)
					.add_u8(fire_resist)
					.add_u8(water_resist)
					.add_u8(wind_resist)
					.add_u8(earth_resist);
			}
			ServerPacket::StartGame(i) => {
				p.add_u8(63).add_u8(3).add_u32(i);
			}
			ServerPacket::Unknown => {
				p.add_u8(107).add_u8(0x14).add_u8(0x69); //TODO these values are magic
			}
			ServerPacket::MapId(map, underwater) => {
				p.add_u8(76).add_u16(map).add_u8(underwater);
			}
			ServerPacket::PutObject{x,y,id,icon,status,direction,
				light,speed,xp,alignment,name,title,status2,
				pledgeid,pledgename,unknown,v1,hp_bar,v2,v3} => {
				p.add_u8(64).add_u16(x).add_u16(y).add_u32(id)
				 .add_u16(icon).add_u8(status).add_u8(direction)
				 .add_u8(light).add_u8(speed).add_u32(xp)
				 .add_u16(alignment).add_string(name).add_string(title)
				 .add_u8(status).add_u32(pledgeid).add_string(pledgename)
				 .add_string(unknown).add_u8(v1).add_u8(hp_bar).add_u8(v2)
				 .add_u8(v3);
			}
		}
		p
	}
}

fn change_key(k: u64, v: u32) -> u64 {
    let d : u32 = u32::from_be(v);
    let mut little : u32 = u32::from_be((k & 0xFFFFFFFF).try_into().unwrap());
    let mut little64 = little as u64;
    
    little64 += 0x287effc3;
    little64 &= 0xffffffff;

    little = little64.try_into().unwrap();
    little = u32::from_be(little);
    let mut nk = (k ^ ((d as u64)<<32)) & 0xFFFFFFFF00000000;
    nk |= little as u64;
    nk
}

fn key_init(k: u32) -> u64 {
    let key : u32 = 0x930fd7e2;
    let rotr : u32 = k ^ 0x9c30d539;
    let big0 : u32 = rotr>>13 | rotr<<19;
    let big1 : u32 = big0 ^ key ^ 0x7c72e993;
    let mut keyvec2 = big0.to_be_bytes().to_vec();
    let mut keyvec = big1.to_be_bytes().to_vec();
    keyvec.append(&mut keyvec2);
    u64::from_ne_bytes(keyvec.try_into().unwrap())		
}

struct Packet {
	data: Vec<u8>,
    read: usize,
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
            12 => {
                ClientPacket::Login(self.pull_string(),
                    self.pull_string(),
                    self.pull_u32(),
                    self.pull_u32(),
                    self.pull_u32(),
                    self.pull_u32(),
                    self.pull_u32(),
                    self.pull_u32(),
                    self.pull_u32())

            }
			43 => ClientPacket::NewsDone,
            71 => {
                let val1: u16 = self.pull_u16();
                let val2: u32 = self.pull_u32();
                let val3: u8 = self.pull_u8();
                let val4: u32 = self.pull_u32();
                println!("client: found a client version packet");
                ClientPacket::Version(val1,val2,val3,val4)
            }
			83 => ClientPacket::CharacterSelect{name: self.pull_string()},
            _ => ClientPacket::Unknown
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
	fn add_u16(&mut self, d: u16) -> &mut Packet {
		self.data.append(&mut d.to_le_bytes().to_vec());
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
        let mut val : u16 = (self.data[self.read+1] as u16) << 8;
        val |= self.data[self.read] as u16;
        self.read += 2;
        val
    }
    fn pull_u32(&mut self) -> u32 {
         let mut val : u32 = self.data[self.read+3] as u32;
        val = (val<<8) | (self.data[self.read+2] as u32);
        val = (val<<8) | (self.data[self.read+1] as u32);
        val = (val<<8) | (self.data[self.read] as u32);
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
            self.data[i] ^= key_vec[i&7] ^ self.data[i-1];
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

		let mut k : u8 = self.data[0] ^ b1 ^ key_vec[5];
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
                let mut data : Vec<u8>  = Vec::new();
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
        let data: Vec<u8> = Vec::from([0xb0, 0x9d, 0xe8, 0xde, 0x83, 0xcd, 0xbc, 0x1b, 0xd2, 0x28, 0x25, 0x3f]);
        let key: u64 = key_init(0x12345678);
        let required : Vec<u8> = Vec::from([0x47, 0x33, 0x00, 0xe4, 0x04, 0x00, 0x00, 0x52, 0xed, 0x8a, 0x01, 0x00]);
        let mut packet = Packet::new();
        packet.add_vec(&data);
        let d = packet.decrypt(key).buf();
        let cd = packet.peek_u32();
        assert_eq!(d, required);
        assert_eq!(cd, 0xe4003347);
        let new_key = change_key(key, cd);
        let required_new_key : u64 =  0x63430cfe184ef01d;
        assert_eq!(new_key, required_new_key);
    }

     #[test]
    fn test_known_data_encrypt() {
        let required: Vec<u8> = Vec::from([0xb0, 0x9d, 0xe8, 0xde, 0x83, 0xcd, 0xbc, 0x1b, 0xd2, 0x28, 0x25, 0x3f]);
        let key: u64 = key_init(0x12345678);
        let data : Vec<u8> = Vec::from([0x47, 0x33, 0x00, 0xe4, 0x04, 0x00, 0x00, 0x52, 0xed, 0x8a, 0x01, 0x00]);
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
		let length : usize = self.reader.read_i16_le().await?.try_into().unwrap();
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
	
	fn set_encrption_key(&mut self, d: u64) {
		self.encryption_key = Some(d);
	}
	
	async fn send_packet(&mut self, mut data: Packet) -> Result<(), ClientError> {
		while data.buf().len() < 4 {
			data.add_u8(0);
		}
        let kcv = data.peek_u32();
        println!("client: send packet {:x?}", data.buf());
        if let Some(key) = self.encryption_key {
            data.encrypt(key);
            self.encryption_key = Some(change_key(key, kcv));
        }
		self.writer.write_u16_le(data.len()+2).await?;
		self.writer.write(&data.buf()).await?;
		Ok(())
	}
}

#[derive(Debug,Clone)]
struct ClientError;
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client error")
    }
}

impl From<std::io::Error> for ClientError {
    fn from(_: std::io::Error) -> ClientError {
        ClientError{}
    }
}

async fn process_packet(p: Packet, s: &mut ServerPacketSender) -> Result<(), ClientError> {
    let c = p.convert();
    Ok(
    match c {
        ClientPacket::Version(a,b,c,d) => {
            println!("client: version {} {} {} {}", a, b, c, d);
            let mut response: Packet = ServerPacket::ServerVersion{
				id: 2,
				version: 2,
				time: 3,
				new_accounts: 1,
				english: 1,
				country: 0,
			}.build();
            s.send_packet(response).await?;
        }
        ClientPacket::Login(u,p,v1,v2,v3,v4,v5,v6,v7) => {
            println!("client: login attempt for {} {} {} {} {} {} {} {}", u, v1, v2, v3, v4, v5, v6, v7);
			let mut response = ServerPacket::LoginResult{
				code: 0 }.build();//TODO put in real value, this means login success
			s.send_packet(response).await?;
			
			response = ServerPacket::News("This is the news".to_string()).build();
			s.send_packet(response).await?;
        }
		ClientPacket::NewsDone => {
			//send number of characters the player has
			let mut response = ServerPacket::NumberCharacters(1,8).build();
			s.send_packet(response).await?;
			
			for _ in 0..1 {
				response = ServerPacket::LoginCharacterDetails{
					name: "whatever".to_string(),
					pledge: "whocares".to_string(),
					ctype: 1,
					gender: 2,
					alignment: 32767,
					hp: 1234,
					mp: 95,
					ac: 248,
					level: 51,
					strength: 12,
					dexterity: 12,
					constitution: 12,
					wisdom: 12,
					charisma: 12,
					intelligence: 12,
				}.build();
				s.send_packet(response).await?;
			}
		}
		ClientPacket::CharacterSelect{name} => {
			println!("client: login with {}", name);
			let mut response = ServerPacket::StartGame(0).build();
			s.send_packet(response).await?;
			
			response = ServerPacket::Unknown.build();
			s.send_packet(response).await?;
			
			response = ServerPacket::CharacterDetails{
				id: 1, level: 5, xp: 1234, strength: 12, dexterity: 12,
				constitution: 12, wisdom: 12, charisma: 12, intelligence: 12,
				curr_hp: 123, max_hp: 985, curr_mp: 34, max_mp: 345, time: 1, ac: 253,
				food: 100, weight: 23, alignment: 32675, fire_resist: 0,
				water_resist: 0, wind_resist: 0, earth_resist: 0}.build();
			s.send_packet(response).await?;
			
			s.send_packet(ServerPacket::MapId(4,0).build()).await?;
			
			s.send_packet(ServerPacket::PutObject{
				x: 32767,y:32767,id:1,icon:1,status:0,direction:0,
				light:5,speed:50,xp:1234,alignment:32767,name:"testing".to_string(),
				title:"i am groot".to_string(),status2:0,
				pledgeid:0,pledgename:"avengers".to_string(),unknown:"potato".to_string(),
				v1:0,hp_bar:100,v2:0,v3:0}.build()).await?;
			
			//TODO send spmr packet?
			
			//TODO send title packet
			
			//TODO send weather packet
			
			//TODO send owncharstatus packet
		}
        ClientPacket::Unknown => {
            println!("client: received unknown packet");
        }
    })
}

async fn process_client(socket: tokio::net::TcpStream, cd: ClientData) -> Result<u8, ClientError> {
	let (reader, writer) = socket.into_split();
	let mut packet_writer = ServerPacketSender::new(writer);

	let encryption_key : u32 = rand::thread_rng().gen();
	let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

    let mut brd_rx : tokio::sync::broadcast::Receiver<ServerMessage> = cd.get_broadcast_rx();
    let server_tx = cd.server_tx;

	let mut key_packet = Packet::new();
	key_packet.add_u8(65)
		.add_u32(encryption_key);
	packet_writer.send_packet(key_packet).await?;
	packet_writer.set_encrption_key(packet_reader.get_key());
    loop {
        futures::select! {
            packet = packet_reader.read_packet().fuse() => {
                let p = packet.unwrap();
        	    println!("client: Packet received is {:x?}", p.buf());
                process_packet(p, &mut packet_writer).await;
            }
            msg = brd_rx.recv().fuse() => {
                println!("client: Received broadcast message from server");
            }
        }
    }

    Ok(0)
}

pub async fn setup_game_server(cd: ClientData) -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
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
