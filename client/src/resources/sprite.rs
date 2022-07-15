use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

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

struct SpriteTile {
    x: i8,
    y: i8,
    width: u8,
    height: u8,
    data: Vec<u16>,
}

impl SpriteTile {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            data: Vec::new(),
        }
    }
}

struct SpriteTileInfo {
    x: u8,
    y: u8,
    h: u8,
    tile: u16,
}

impl SpriteTileInfo {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            h: 0,
            tile: 0,
        }
    }
}

struct SpriteFrame {
    x1: u16,
    x2: u16,
    y1: u16,
    y2: u16,
    mystery1: u32,
    tiles: Vec<SpriteTileInfo>,
}

impl SpriteFrame {
    fn new() -> Self {
        Self {
            x1: 0,
            x2: 0,
            y1: 0,
            y2: 0,
            mystery1: 0,
            tiles: Vec::new(),
        }
    }
}
struct Sprite {
    pallete: Option<Vec<u16>>,
    frames: Vec<SpriteFrame>,
    tiles: Vec<SpriteTile>,
}

impl Sprite {
    ///Contents of %d-%d.spr is given to this function in a cursor
    pub async fn parse_sprite(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        println!("Parsing sprite");
        let mut pallete = None;
        let temp = cursor.read_i8().await.unwrap();
        let num_frames = if (temp < 0) {
            println!("Reading pallete for sprite");
            let mut num_pallete_entries: u16 = cursor.read_u8().await.unwrap() as u16;
            if num_pallete_entries == 0 {
                num_pallete_entries = 0x100;
            }
            let mut p = Vec::new();
            for _ in 0..=num_pallete_entries {
                p.push(cursor.read_u16().await.unwrap());
            }
            pallete = Some(p);
            cursor.read_u8().await.unwrap()
        } else {
            temp as u8
        };
        let mut frames = Vec::new();
        for _ in 0..=num_frames {
            let mut frame: SpriteFrame = SpriteFrame::new();
            frame.x1 = cursor.read_u16().await.unwrap();
            frame.x2 = cursor.read_u16().await.unwrap();
            frame.y1 = cursor.read_u16().await.unwrap();
            frame.y2 = cursor.read_u16().await.unwrap();
            frame.mystery1 = cursor.read_u32().await.unwrap();
            let num_tiles = cursor.read_u16().await.unwrap();
            for _ in 0..=num_tiles {
                let mut tile = SpriteTileInfo::new();
                tile.x = cursor.read_u8().await.unwrap();
                tile.y = cursor.read_u8().await.unwrap();
                tile.h = cursor.read_u8().await.unwrap();
                tile.tile = cursor.read_u16().await.unwrap();
                frame.tiles.push(tile);
            }
            frames.push(frame);
        }
        let num_tiles = cursor.read_u32().await.unwrap();
        let mut tile_offset = Vec::with_capacity(num_tiles as usize);
        for _ in 0..=num_tiles {
            tile_offset.push(cursor.read_u32().await.unwrap());
        }
        let tile_position = cursor.stream_position().await.unwrap();
        if let Some(p) = &pallete {
            for i in 0..=num_tiles {
                let _e = cursor
                    .seek(tokio::io::SeekFrom::Start(
                        tile_position + tile_offset[i as usize] as u64,
                    ))
                    .await;
                let mut t: SpriteTile = SpriteTile::new();
            }
        } else {
            for i in 0..=num_tiles {
                let _e = cursor
                    .seek(tokio::io::SeekFrom::Start(
                        tile_position + tile_offset[i as usize] as u64,
                    ))
                    .await;
                let mut t: SpriteTile = SpriteTile::new();
                t.x = cursor.read_i8().await.unwrap();
                t.y = cursor.read_i8().await.unwrap();
                t.width = cursor.read_u8().await.unwrap();
                t.height = cursor.read_u8().await.unwrap();
                t.data = Vec::with_capacity(t.width as usize * t.height as usize);
                for row in 0..=t.height {
                    let row_segments = cursor.read_u8().await.unwrap();
                    let mut row_offset = 0;
                    for segment in 0..=row_segments {
                        let skip = cursor.read_u8().await.unwrap() / 2;
                        let w = cursor.read_u8().await.unwrap();
                        row_offset += skip;
                        for _ in 0..=w {
                            t.data[row as usize * t.width as usize + row_offset as usize] =
                                cursor.read_u16().await.unwrap();
                        }
                    }
                }
            }
        }

        Some(Self {
            pallete: pallete,
            frames: frames,
            tiles: Vec::new(),
        })
    }
}
