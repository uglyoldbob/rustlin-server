use std::io::Cursor;
use tokio::io::AsyncReadExt;

const embedded_sprite_table: &[u8] = include_bytes!("sprite_table.txt");

async fn get_integer(c: &mut Cursor<Vec<u8>>) -> (Option<u8>, i32) {
    let mut last_byte = c.read_u8().await;
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
        cursor.read_u8().await; //first byte is ignored
        let (b, val) = get_integer(&mut cursor).await;
        println!("SPR: {} {:?}", val, b);
        Self {}
    }
}

struct Sprite {}
