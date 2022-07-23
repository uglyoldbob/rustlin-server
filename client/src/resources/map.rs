use tokio::io::AsyncReadExt;

#[derive(Copy, Clone)]
pub struct TileSet {}

impl TileSet {
    pub async fn decode_tileset_data(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        let num_tiles = cursor.read_u16_le().await.ok()?;
        let mut offsets = Vec::with_capacity(num_tiles as usize);
        for _ in 0..num_tiles {
            offsets.push(cursor.read_u32_le().await.ok()?);
        }

        Some(TileSet {})
    }

    pub fn draw_tile(&self, x: u16, y: u16, subtile: u16, canvas: &mut sdl2::render::WindowCanvas) {
    }
}

pub struct TileData {
    x: i8,
    y: i8,
    h: i8,
    data: u32,
}

pub struct MapObject {
    tiles: Vec<TileData>,
}

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
