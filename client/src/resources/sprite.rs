use std::io::Cursor;
use tokio::io::AsyncReadExt;

const embedded_sprite_table: &[u8] = include_bytes!("sprite_table.txt");

async fn get_integer(c: &mut Cursor<Vec<u8>>) -> (Option<u8>, i32) {
    let mut last_byte = c.read_u8().await;
    loop {
        match last_byte {
            Ok(last) => match last {
                b'0'..=b'9' => break,
                _ => {
                    last_byte = c.read_u8().await;
                }
            },
            Err(_) => break,
        }
    }
    let mut collected_value: i32 = 0;
    let mut index = 0;
    let mut found_minus = false;
    loop {
        match last_byte {
            Ok(last_byte) => match last_byte {
                b'0'..=b'9' => {
                    collected_value *= 10;
                    collected_value += (last_byte - b'0') as i32;
                    index += 1;
                }
                b'-' => {
                    if index == 0 {
                        found_minus = true;
                    }
                }
                _ => {
                    break;
                }
            },
            Err(_) => {
                break;
            }
        }
        last_byte = c.read_u8().await;
    }
    if found_minus {
        collected_value = collected_value * -1;
    }

    (last_byte.ok(), collected_value)
}

pub struct SpriteTableEntry {}

impl SpriteTableEntry {
    pub async fn load_embedded_table() -> Self {
        let mut data = embedded_sprite_table.to_vec();
        for d in data.iter_mut() {
            if *d == 0xd {
                *d = b' ';
            }
        }
        let mut cursor = Cursor::new(data);
        let _e = cursor.read_u8().await; //first byte is ignored
        let sprite_val = 0;
        let mut last_byte: Option<u8> = None;
        let (b, val) = get_integer(&mut cursor).await;
        last_byte = b;
        println!("SPR: {} {:?}", val, last_byte);
        println!("Sprite {} has {} frames", sprite_val, val);
        if let Some(b) = last_byte {
            if b == b'=' {
                let (v, val) = get_integer(&mut cursor).await;
                println!("Sprite {} has alias {}", sprite_val, val);
            }
        }
        Self {}
    }
}

struct Sprite {}

impl Sprite {
    ///Contents of %d-%d.spr is given to this function in a cursor
    pub async fn parse_sprite(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        Some(Self {})
    }
}
