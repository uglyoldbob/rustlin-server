use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

const EMBEDDED_SPRITE_TABLE: &[u8] = include_bytes!("sprite_table.txt");

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
        let mut data = EMBEDDED_SPRITE_TABLE.to_vec();
        for d in data.iter_mut() {
            if *d == 0xd {
                *d = b' ';
            }
        }
        let mut cursor = Cursor::new(data);
        let _e = cursor.read_u8().await; //first byte is ignored
        let sprite_val = 0;
        let last_byte: Option<u8>;
        let (b, val) = get_integer(&mut cursor).await;
        last_byte = b;
        println!("SPR: {} {:?}", val, last_byte);
        println!("Sprite {} has {} frames", sprite_val, val);
        if let Some(b) = last_byte {
            if b == b'=' {
                let (_v, val) = get_integer(&mut cursor).await;
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

    pub fn to_gui<'a, T>(&self, t: &'a TextureCreator<T>) -> Texture<'a> {
        let w = self.width as u32;
        let h = self.height as u32;
        let mut data: Vec<u8> = Vec::with_capacity((w * h * 2) as usize);
        for v in &self.data {
            data.extend_from_slice(&v.to_be_bytes());
        }
        let mut surface =
            Surface::from_data(&mut data[..], w, h, w * 2, PixelFormatEnum::RGB555).unwrap();
        let _e = surface.set_color_key(true, sdl2::pixels::Color::BLACK);
        let txt = Texture::from_surface(&surface, t).unwrap();
        txt
    }
}

#[derive(Copy, Clone)]
struct SpriteTileInfo {
    x: i8,
    y: i8,
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

#[derive(Clone)]
struct SpriteFrame {
    x1: i16,
    x2: i16,
    y1: i16,
    y2: i16,
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
pub struct Sprite {
    pallete: Option<Vec<u16>>,
    frames: Vec<SpriteFrame>,
    tiles: Vec<SpriteTile>,
}

/// The sprite struct used by the gui thread
pub struct SpriteGui<'a> {
    frames: Vec<SpriteFrame>,
    tiles: Vec<Texture<'a>>,
    tile_offset: Vec<(i16, i16)>,
}

impl<'a> SpriteGui<'a> {
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn draw(&self, x: i16, y: i16, frame: usize, canvas: &mut sdl2::render::WindowCanvas) {
        let f = &self.frames[frame];
        for tdata in f.tiles.iter() {
            let tx = tdata.x as i32 - 4;
            let ty = tdata.y as i32 + 1;
            let mut x2 = 24 * (tx / 2 + ty);
            let mut y2 = 12 * (ty - tx / 2);
            if (tx % 2) == 1 {
                x2 += 24;
            } else if (tx % 2) == -1 {
                y2 += 12;
            }
            let t = &self.tiles[tdata.tile as usize];
            let (x3, y3) = &self.tile_offset[tdata.tile as usize];
            let x3 = *x3 as i32;
            let y3 = *y3 as i32;
            let q = t.query();
            let xsum = x as i32 + x2 + x3;
            let ysum = y as i32 + y2 + y3;
            let _e = canvas.copy(
                t,
                None,
                sdl2::rect::Rect::new(xsum, ysum, q.width.into(), q.height.into()),
            );
        }
    }
}

impl Sprite {
    pub fn to_gui<'a, T>(&self, t: &'a TextureCreator<T>) -> SpriteGui<'a> {
        let len = self.tiles.len();
        let mut tiles = Vec::with_capacity(len);
        let mut tile_data = Vec::with_capacity(len);
        for tile in &self.tiles {
            tiles.push(tile.to_gui(t));
            tile_data.push((tile.x as i16, tile.y as i16));
        }
        SpriteGui {
            frames: self.frames.clone(),
            tiles: tiles,
            tile_offset: tile_data,
        }
    }

    ///Contents of %d-%d.spr is given to this function in a cursor
    pub async fn parse_sprite(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        let mut pallete = None;
        let temp = cursor.read_i8().await.ok()?;
        let num_frames = if temp < 0 {
            let mut num_pallete_entries: u16 = cursor.read_u8().await.ok()? as u16;
            if num_pallete_entries == 0 {
                num_pallete_entries = 0x100;
            }
            let mut p = Vec::new();
            for _ in 0..num_pallete_entries {
                p.push(cursor.read_u16().await.ok()?);
            }
            pallete = Some(p);
            cursor.read_u8().await.ok()?
        } else {
            temp as u8
        };
        let mut frames = Vec::new();
        for _ in 0..num_frames {
            let mut frame: SpriteFrame = SpriteFrame::new();
            frame.x1 = cursor.read_i16_le().await.ok()?;
            frame.x2 = cursor.read_i16_le().await.ok()?;
            frame.y1 = cursor.read_i16_le().await.ok()?;
            frame.y2 = cursor.read_i16_le().await.ok()?;
            frame.mystery1 = cursor.read_u32_le().await.ok()?;
            let num_tiles = cursor.read_u16_le().await.ok()?;
            for _i in 0..num_tiles {
                let mut tile = SpriteTileInfo::new();
                tile.x = cursor.read_i8().await.ok()?;
                tile.y = cursor.read_i8().await.ok()?;
                tile.h = cursor.read_u8().await.ok()?;
                tile.tile = cursor.read_u16_le().await.ok()?;
                frame.tiles.push(tile);
            }
            frames.push(frame);
        }
        let num_tiles = cursor.read_u32_le().await.ok()?;
        let mut tile_offset = Vec::with_capacity(num_tiles as usize);
        for _ in 0..num_tiles {
            let val = cursor.read_u32_le().await.ok()?;
            tile_offset.push(val);
        }
        let _tile_size = cursor.read_u32_le().await.ok()?;
        let tile_position = cursor.stream_position().await.ok()?;
        let mut tiles = Vec::with_capacity(num_tiles as usize);
        if let Some(p) = &pallete {
            for i in 0..num_tiles {
                let _e = cursor
                    .seek(tokio::io::SeekFrom::Start(
                        tile_position + tile_offset[i as usize] as u64,
                    ))
                    .await;
                let mut t: SpriteTile = SpriteTile::new();
                t.x = cursor.read_i8().await.ok()?;
                t.y = cursor.read_i8().await.ok()?;
                t.width = cursor.read_u8().await.ok()?;
                t.height = cursor.read_u8().await.ok()?;
                let size = t.width as usize * t.height as usize;
                t.data = Vec::with_capacity(size);
                for _i in 0..size {
                    t.data.push(0);
                }
                for row in 0..t.height {
                    let row_segments = cursor.read_u8().await.ok()?;
                    let mut row_offset = 0;
                    for _segment in 0..row_segments {
                        let skip = cursor.read_u8().await.ok()? / 2;
                        let w = cursor.read_u8().await.ok()?;
                        row_offset += skip;
                        for i in 0..w {
                            let i: usize = i as usize;
                            let index = cursor.read_u8().await.ok()?;
                            let val = p[index as usize];
                            t.data[row as usize * t.width as usize + row_offset as usize + i] = val;
                        }
                        row_offset += w;
                    }
                }
                tiles.push(t);
            }
        } else {
            for i in 0..num_tiles {
                let _e = cursor
                    .seek(tokio::io::SeekFrom::Start(
                        tile_position + tile_offset[i as usize] as u64,
                    ))
                    .await;
                let mut t: SpriteTile = SpriteTile::new();
                t.x = cursor.read_i8().await.ok()?;
                t.y = cursor.read_i8().await.ok()?;
                t.width = cursor.read_u8().await.ok()?;
                t.height = cursor.read_u8().await.ok()?;
                let size = t.width as usize * t.height as usize;
                t.data = Vec::with_capacity(size);
                for _i in 0..size {
                    t.data.push(0);
                }
                for row in 0..t.height {
                    let row_segments = cursor.read_u8().await.ok()?;
                    let mut row_offset = 0;
                    for _segment in 0..row_segments {
                        let skip = cursor.read_u8().await.ok()? / 2;
                        let w = cursor.read_u8().await.ok()?;
                        row_offset += skip;
                        for i in 0..w {
                            let i: usize = i as usize;
                            let val = cursor.read_u16().await.ok()?;
                            t.data[row as usize * t.width as usize + row_offset as usize + i] = val;
                        }
                        row_offset += w;
                    }
                }
                tiles.push(t);
            }
        }
        Some(Self {
            pallete: pallete,
            frames: frames,
            tiles: tiles,
        })
    }
}
