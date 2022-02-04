use tokio::net::TcpListener;
use std::error::Error;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::fmt;

use std::convert::TryInto;
use std::vec::Vec;

use rand::Rng;

struct Packet {
	data: Vec<u8>,
}

impl Packet {
	fn new() -> Packet {
		Packet {
			data: Vec::new(),
		}
	}
	fn len(&self) -> u16 {
		self.data.len().try_into().unwrap()
	}
	fn buf(&self) -> Vec<u8> {
		self.data.to_vec()
	}
	fn add_vec(&mut self, d: Vec<u8>) -> &mut Packet {
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
	fn decrypt(&mut self, key: u64) -> &mut Packet {
		let key_vec = key.to_le_bytes().to_vec();
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

struct ServerPacketReceiver {
	reader: tokio::net::tcp::OwnedReadHalf,
	decryption_key: u64,
}

impl ServerPacketReceiver {
	fn new(r: tokio::net::tcp::OwnedReadHalf, key: u32) -> ServerPacketReceiver {
		//TODO: properly generate the starting key from the seed
		ServerPacketReceiver {
			reader: r,
			decryption_key: key as u64,
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
		packet.add_vec(contents);
		packet.decrypt(self.decryption_key);
		//TODO: mutate the decryption key
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
	
	async fn send_packet(&mut self, data: Packet) -> Result<(), ClientError> {
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

async fn process_client(socket: tokio::net::TcpStream) -> Result<u8, ClientError> {
	let (reader, writer) = socket.into_split();
	let mut packet_writer = ServerPacketSender::new(writer);

	let encryption_key : u32 = rand::thread_rng().gen();
	let mut packet_reader = ServerPacketReceiver::new(reader, encryption_key);

	let mut key_packet = Packet::new();
	key_packet.add_u8(65)
		.add_u32(encryption_key);
	packet_writer.send_packet(key_packet).await?;
	packet_writer.set_encrption_key(packet_reader.get_key());
	let packet = packet_reader.read_packet().await?;
	println!("Packet received is {:x?}", packet.buf());

    Ok(0)
}

pub async fn setup_game_server() -> Result<tokio::sync::oneshot::Sender<u32>, Box<dyn Error>> {
    println!("server: Starting the game server");
	let (update_tx, mut update_rx) = tokio::sync::oneshot::channel::<u32>();
    let update_listener = TcpListener::bind("0.0.0.0:2000").await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = update_listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    println!("server: Received a client from {}", addr);
                    tokio::spawn(async move {
                        if let Err(e) = process_client(socket).await {
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