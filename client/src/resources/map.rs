use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

#[derive(Clone)]
pub struct Tile {
    x: i8,
    y: i8,
    w: u8,
    h: u8,
    data: [u16; 24 * 48],
}

pub struct TileSetGui<'a> {
    tiles: Vec<Texture<'a>>,
}

impl<'a> TileSetGui<'a> {
    pub fn draw_tile(&self, x: u16, y: u16, subtile: u16, canvas: &mut sdl2::render::WindowCanvas) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(
                t,
                None,
                Rect::new(x as i32, y as i32, q.width.into(), q.height.into()),
            );
        }
    }

    pub fn draw_left(&self, x: u16, y: u16, subtile: u16, canvas: &mut sdl2::render::WindowCanvas) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(
                t,
                Rect::new(0, 0, q.width / 2, 24),
                Rect::new(x as i32, y as i32, q.width / 2, q.height.into()),
            );
        }
    }

    pub fn draw_right(
        &self,
        x: u16,
        y: u16,
        subtile: u16,
        canvas: &mut sdl2::render::WindowCanvas,
    ) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(
                t,
                Rect::new(q.width as i32 / 2, 0, q.width / 2, 24),
                Rect::new(
                    x as i32 + q.width as i32 / 2,
                    y as i32,
                    q.width / 2,
                    q.height.into(),
                ),
            );
        }
    }
}

#[derive(Clone)]
pub struct TileSet {
    tiles: Vec<Tile>,
}

impl TileSet {
    pub async fn decode_tileset_data(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        let num_tiles = cursor.read_u16_le().await.ok()?;
        cursor.read_u16_le().await.ok()?;
        let mut offsets = Vec::with_capacity(num_tiles as usize);
        for _ in 0..num_tiles {
            offsets.push(cursor.read_u32_le().await.ok()?);
        }
        cursor.read_u32_le().await.ok()?;
        let base_offset = cursor.position();
        let mut tiles = Vec::new();
        for t in offsets {
            let _e = cursor
                .seek(tokio::io::SeekFrom::Start(base_offset + t as u64))
                .await;
            let v1 = cursor.read_u8().await.ok()?;
            let mirrored_tile_data = if (v1 & 2) != 0 {
                let mut mirrored_tile_data = [0 as u16; 24 * 48];
                let x = cursor.read_u8().await.ok()?;
                let y = cursor.read_u8().await.ok()?;
                let w = cursor.read_u8().await.ok()?;
                let h = cursor.read_u8().await.ok()?;
                for i in 0..h {
                    let num_segments = cursor.read_u8().await.ok()?;
                    let mut skip_index = 0;
                    for _ in 0..num_segments {
                        let skip = cursor.read_u8().await.ok()? / 2;
                        let seg_width = cursor.read_u8().await.ok()?;
                        skip_index += skip;
                        for pixel in 0..seg_width {
                            let val = cursor.read_u16_le().await.ok()?;
                            let index = x as usize
                                + (y + i) as usize * 48
                                + skip_index as usize
                                + pixel as usize;
                            mirrored_tile_data[index] = val;
                        }
                        skip_index += seg_width;
                    }
                }
                mirrored_tile_data
            } else {
                let mut tile_data: [u16; 288] = [0; 288];
                for i in 0..288 {
                    tile_data[i] = cursor.read_u16_le().await.ok()?;
                }
                let mut mirrored_tile_data = [0 as u16; 24 * 48];
                let mut ind_offset = 0;

                for i in 0..24 {
                    let mut width = 2 * (i + 1);
                    if i > 11 {
                        width -= 4 * (i - 11);
                    }
                    for j in 0..width {
                        let d: u16 = tile_data[ind_offset];
                        ind_offset += 1;
                        mirrored_tile_data[i * 48 + 24 + j] = d;
                        mirrored_tile_data[i * 48 + 23 - j] = d;
                    }
                }
                mirrored_tile_data
            };

            tiles.push(Tile {
                x: 0,
                y: 0,
                w: 24,
                h: 24,
                data: mirrored_tile_data,
            });
        }
        Some(TileSet { tiles: tiles })
    }

    pub fn to_gui<'a, T>(self, tc: &'a TextureCreator<T>) -> TileSetGui {
        let mut t = Vec::new();
        for tmp in &self.tiles {
            let w = 48;
            let h = 24;
            let mut data8: Vec<u8> = tmp.data.iter().flat_map(|val| val.to_le_bytes()).collect();
            let mut surf = sdl2::surface::Surface::from_data(
                data8.as_mut_slice(),
                w as u32,
                h as u32,
                (2 * w) as u32,
                PixelFormatEnum::RGB555,
            )
            .unwrap();
            let _e = surf.set_color_key(true, sdl2::pixels::Color::BLACK);
            let texture = Texture::from_surface(&surf, tc).unwrap();
            t.push(texture);
        }
        TileSetGui { tiles: t }
    }
}

#[derive(Clone)]
pub struct TileData {
    x: i8,
    y: i8,
    h: i8,
    data: u32,
}

#[derive(Clone)]
pub struct MapObject {
    tiles: Vec<TileData>,
}

#[derive(Clone)]
pub struct MapSegment {
    tiles: [u32; 64 * 128],
    attributes: [u16; 64 * 128],
    mystery1: Vec<[u16; 3]>,
    objects: Vec<MapObject>,
    switches: Vec<u32>,
    x: u16,
    y: u16,
}

impl MapSegment {
    pub fn get_map_name(x: u16, y: u16) -> String {
        let modx = (x >> 6) + 0x7e00;
        let mody = (y >> 6) + 0x7e00;
        format!("{}{}", modx, mody)
    }

    pub async fn load_map_seg(
        cursor: &mut std::io::Cursor<&Vec<u8>>,
        x: u16,
        y: u16,
    ) -> Option<Self> {
        let mut t = [0; 64 * 128];
        for t in t.iter_mut() {
            *t = cursor.read_u16_le().await.ok()? as u32;
        }

        let _mystery1 = cursor.read_u16_le().await.ok()?;

        let mut t2 = [0; 64 * 128];
        for t in t2.iter_mut() {
            *t = cursor.read_u8().await.ok()?;
        }

        let mystery2 = cursor.read_u32_le().await.ok()?;
        for _ in 0..mystery2 {
            cursor.read_u16_le().await.ok()?;
            let _m1 = cursor.read_u16_le().await.ok()?;
        }

        let mystery3 = cursor.read_u32_le().await.ok()?;
        for _ in 0..mystery3 {
            let _m1 = cursor.read_u16_le().await.ok()?;
            let _m2 = cursor.read_u16_le().await.ok()?;
        }

        let num_tilesets = cursor.read_u8().await.ok()?;
        for _ in 0..num_tilesets {
            let _tileset = cursor.read_u8().await.ok()?;
        }

        let num_portals = cursor.read_u16_le().await.ok()?;
        for _ in 0..num_portals {
            cursor.read_u8().await.ok()?;
            cursor.read_u8().await.ok()?;
            cursor.read_u8().await.ok()?;
            cursor.read_u16_le().await.ok()?;
            cursor.read_u16_le().await.ok()?;
            cursor.read_u16_le().await.ok()?;
        }

        Some(Self {
            tiles: t,
            attributes: [0; 64 * 64 * 2],
            mystery1: Vec::new(),
            objects: Vec::new(),
            switches: Vec::new(),
            x: x,
            y: y,
        })
    }

    pub async fn load_map_s32(
        cursor: &mut std::io::Cursor<&Vec<u8>>,
        x: u16,
        y: u16,
    ) -> Option<Self> {
        let mut t = [0; 64 * 128];
        for t in t.iter_mut() {
            *t = cursor.read_u32_le().await.ok()?;
        }
        let mut mys1 = Vec::new();
        let quant = cursor.read_u16_le().await.ok()?;
        for _ in 0..quant {
            let mut data = [0; 3];
            for d in data.iter_mut() {
                *d = cursor.read_u16_le().await.ok()?;
            }
            mys1.push(data);
        }
        let mut attr = [0; 64 * 128];
        for t in attr.iter_mut() {
            *t = cursor.read_u16_le().await.ok()?;
        }

        let num_objects = cursor.read_u32_le().await.ok()?;
        let mut objs = Vec::with_capacity(num_objects as usize);
        for _ in 0..num_objects {
            let _index = cursor.read_u16_le().await.ok()?;
            let num_tiles = cursor.read_u16_le().await.ok()?;
            let mut t = Vec::with_capacity(num_tiles as usize);
            for _ in 0..num_tiles {
                let b = cursor.read_i8().await.ok()?;
                let c = cursor.read_i8().await.ok()?;
                if b == -51i8 && c == -51i8 {
                    for _ in 0..5 {
                        cursor.read_u8().await.ok()?;
                    }
                    println!("This map segment feature is unimplemented?");
                } else {
                    let h = cursor.read_i8().await.ok()?;
                    let data = cursor.read_u32_le().await.ok()?;
                    let tile = TileData {
                        x: b,
                        y: c,
                        h: h,
                        data: data,
                    };
                    t.push(tile);
                }
            }
            let obj = MapObject { tiles: t };
            objs.push(obj);
        }

        let num_switches = cursor.read_u32_le().await.ok()?;
        let mut switches = Vec::with_capacity(num_switches as usize);
        for _ in 0..num_switches {
            switches.push(cursor.read_u32_le().await.ok()?);
        }

        let num_portals = cursor.read_u32_le().await.ok()?;
        for _ in 0..num_portals {
            cursor.read_u8().await.ok()?;
            cursor.read_u8().await.ok()?;
            cursor.read_u8().await.ok()?;
            cursor.read_u16_le().await.ok()?;
            cursor.read_u16_le().await.ok()?;
            cursor.read_u16_le().await.ok()?;
        }

        Some(Self {
            tiles: t,
            attributes: attr,
            mystery1: mys1,
            objects: objs,
            switches: switches,
            x: x,
            y: y,
        })
    }
}
