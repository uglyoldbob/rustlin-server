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
	CharacterSelect(String),
	NewsDone,
    Unknown,
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
			83 => ClientPacket::CharacterSelect(self.pull_string()),
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
		while (data.buf().len() < 4) {
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
            let mut response: Packet = Packet::new();
                response.add_u8(10)
                .add_u8(0)
                .add_u8(2) 
                .add_u32(2) //server version
                .add_u32(0) //cache version
                .add_u32(0) //auth version
                .add_u32(0) //npc version
                .add_u32(0) //start time
                .add_u8(1) //new accounts
                .add_u8(1) //english only
                .add_u8(0); //country
            s.send_packet(response).await?;
        }
        ClientPacket::Login(u,p,v1,v2,v3,v4,v5,v6,v7) => {
            println!("client: login attempt for {} {} {} {} {} {} {} {}", u, v1, v2, v3, v4, v5, v6, v7);
			let mut response = Packet::new();
			response.add_u8(21)
				.add_u8(0)	//TODO put in real value
				.add_u32(0)
				.add_string("".to_string())
				.add_u8(0)
				.add_u16(0);
			s.send_packet(response).await?;
			
			response = Packet::new();
			response.add_u8(90).add_string("This is the news".to_string());
			s.send_packet(response).await?;
        }
		ClientPacket::NewsDone => {
			//send number of characters the player has
			let mut response = Packet::new();
			response.add_u8(113)
				.add_u8(1) //number of characters
				.add_u8(8); //number of slots
			s.send_packet(response).await?;
			for _ in 0..1 {
				response = Packet::new();
				response.add_u8(99)
					.add_string("whatever".to_string())	//character name
					.add_string("whocares".to_string())	//pledge name
					.add_u8(1) //character type
					.add_u8(2) //gender
					.add_u16(3) //alignment
					.add_u16(4) //hp
					.add_u16(5) //mp
					.add_u8(6) //ac
					.add_u8(7) //level
					.add_u8(8) //strength
					.add_u8(9) //dexterity
					.add_u8(10) //constitution
					.add_u8(11) //wisdom
					.add_u8(12) //charisma
					.add_u8(13) //intelligence
					.add_u8(14) //?
					.add_u32(15); //?
				s.send_packet(response).await?;
			}
		}
		ClientPacket::CharacterSelect(c) => {
			println!("client: login with {}", c);
			
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
