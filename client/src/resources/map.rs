use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::GameResources;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;

use super::MessageToAsync;

///Represents the top left pixel of a map coordinate on the screen. This is where the tile for that coordinate is rendered.
#[derive(Debug, PartialEq)]
pub struct ScreenCoordinate {
    pub x: i32,
    pub y: i32,
    x0: i32,
    y0: i32,
}

#[derive(Debug, PartialEq)]
pub struct MapCoordinate {
    pub a: u16,
    pub b: u16,
    pub x0: i32,
    pub y0: i32,
}

impl MapCoordinate {
    /// Converts the map coordinate to the top left pixel coordinate of where the coordinate should be drawn.
    pub fn to_screen(&self) -> ScreenCoordinate {
        ScreenCoordinate {
            x: (24 * (self.b as i32) - 24 * (self.a as i32)) as i32 - self.x0,
            y: (12 * (self.a as i32) + 12 * (self.b as i32)) as i32 - self.y0,
            x0: self.x0,
            y0: self.y0,
        }
    }

    ///Converts the given map coordinate to the top left pixel coordinate, like to_screen
    pub fn screen(&self, a: u16, b: u16) -> ScreenCoordinate {
        ScreenCoordinate {
            x: (24 * (b as i32) - 24 * (a as i32)) as i32 - self.x0,
            y: (12 * (a as i32) + 12 * (b as i32)) as i32 - self.y0,
            x0: self.x0,
            y0: self.y0,
        }
    }

    pub fn delta(a: i32, b: i32) -> (i32, i32) {
        (24 * b - 24 * a, 12 * a + 12 * b)
    }

    /// This function builds a MapCoordinate that places the dead center of the tile at the given screen coordinates
    pub fn build(a: u16, b: u16, x1: u32, y1: u32) -> Self {
        Self {
            a: a,
            b: b,
            x0: 24 * (b as i32) - 24 * (a as i32) - (x1 as i32) + 24,
            y0: 12 * (a as i32) + 12 * (b as i32) - (y1 as i32) + 12,
        }
    }
}

impl ScreenCoordinate {
    pub fn to_map(&self) -> MapCoordinate {
        MapCoordinate {
            a: ((2 * (self.y as i32) + 2 * self.y0 - (self.x as i32) - self.x0) / 48) as u16,
            b: ((2 * (self.y as i32) + 2 * self.y0 + (self.x as i32) + self.x0) / 48) as u16,
            x0: self.x0,
            y0: self.y0,
        }
    }

    pub fn map_coordinates(&self, x: i16, y: i16) -> (u16, u16) {
        let x1 = x - 12;
        let y1 = y + 6;
        let a = ((2 * (y1 as i32) + 2 * self.y0 - (x1 as i32) - self.x0) / 48) as u16;
        let x2 = x - 12;
        let y2 = y - 6;
        let b = ((2 * (y2 as i32) + 2 * self.y0 + (x2 as i32) + self.x0) / 48) as u16;
        (a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_transform1() {
        let map = MapCoordinate::build(32768, 32768, 320, 240);
        let screen = map.to_screen();
        assert_eq!(screen.x, 296);
        assert_eq!(screen.y, 228);
        let map2 = screen.to_map();
        assert_eq!(map, map2);
        let mut index = 0;
        for (a, b, x, y) in [
            (-1, -1, 0, -24),
            (-1, 0, 24, -12),
            (-1, 1, 48, 0),
            (0, -1, -24, -12),
            (0, 1, 24, 12),
            (1, -1, -48, 0),
            (1, 0, -24, 12),
            (1, 1, 0, 24),
        ] {
            println!("Case {}", index);
            index += 1;
            let am = 32768 + (a as i32);
            let bm = 32768 + (b as i32);
            let screen2 = map.screen(am as u16, bm as u16);
            assert_eq!(screen2.x, 296 + x);
            assert_eq!(screen2.y, 228 + y);

            let (xcalc, ycalc) = MapCoordinate::delta(a, b);
            assert_eq!(xcalc, x);
            assert_eq!(ycalc, y);
        }
    }
    #[test]
    fn coordinate_transform2() {
        let map = MapCoordinate::build(0, 0, 320, 240);
        let screen = map.to_screen();
        assert_eq!(screen.x, 296);
        assert_eq!(screen.y, 228);
        let map2 = screen.to_map();
        assert_eq!(map, map2);
    }
    #[test]
    fn coordinate_transform3() {
        let map = MapCoordinate::build(65535, 65535, 320, 240);
        let screen = map.to_screen();
        assert_eq!(screen.x, 296);
        assert_eq!(screen.y, 228);
        let map2 = screen.to_map();
        assert_eq!(map, map2);
    }
    #[test]
    fn coordinate_transform4() {
        let map = MapCoordinate::build(65535, 0, 320, 240);
        let screen = map.to_screen();
        assert_eq!(screen.x, 296);
        assert_eq!(screen.y, 228);
        let map2 = screen.to_map();
        assert_eq!(map, map2);
    }
    #[test]
    fn coordinate_transform5() {
        let map = MapCoordinate::build(0, 65535, 320, 240);
        let screen = map.to_screen();
        assert_eq!(screen.x, 296);
        assert_eq!(screen.y, 228);
        let map2 = screen.to_map();
        assert_eq!(map, map2);
    }
}

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
    pub fn draw_tile<T: sdl2::render::RenderTarget>(
        &self,
        x: i32,
        y: i32,
        subtile: u16,
        canvas: &mut sdl2::render::Canvas<T>,
    ) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(t, None, Rect::new(x, y, q.width.into(), q.height.into()));
        }
    }

    pub fn draw_left<T: sdl2::render::RenderTarget>(
        &self,
        x: i32,
        y: i32,
        subtile: u16,
        canvas: &mut sdl2::render::Canvas<T>,
    ) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(
                t,
                Rect::new(0, 0, q.width / 2, 24),
                Rect::new(x, y, q.width / 2, q.height.into()),
            );
        }
    }

    pub fn draw_right<T: sdl2::render::RenderTarget>(
        &self,
        x: i32,
        y: i32,
        subtile: u16,
        canvas: &mut sdl2::render::Canvas<T>,
    ) {
        if let Some(t) = self.tiles.get(subtile as usize) {
            let q = t.query();
            let _e = canvas.copy(
                t,
                Rect::new(q.width as i32 / 2, 0, q.width / 2, 24),
                Rect::new(x + q.width as i32 / 2, y, q.width / 2, q.height.into()),
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
                let _w = cursor.read_u8().await.ok()?;
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
pub struct MapSegmentGui<'a> {
    tile_ref: HashMap<u16, Rc<TileSetGui<'a>>>,
    tilesets: HashSet<u16>,
    tiles: [u32; 64 * 128],
    attributes: [u16; 64 * 128],
    mystery1: Vec<[u16; 3]>,
    objects: Vec<MapObject>,
    switches: Vec<u32>,
    x: u16,
    y: u16,
    mapnum: u16,
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
    mapnum: u16,
}

impl<'a> MapSegmentGui<'a> {
    pub fn get_mapnum(&self) -> u16 {
        self.mapnum
    }
    pub fn check_tilesets(
        &mut self,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        for tileset in &self.tilesets {
            if !self.tile_ref.contains_key(tileset) {
                let t = r.tilesets.get_or_load(*tileset, || {
                    let _e = send.blocking_send(MessageToAsync::LoadTileset(*tileset));
                });
                if let Some(t) = t {
                    self.tile_ref.insert(*tileset, t);
                }
            }
        }
    }

    pub fn draw_floor<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        map: &MapCoordinate,
        _r: &mut GameResources,
    ) {
        let screen = map.screen(self.x, self.y);
        for a in 0..64 {
            for b in 0..64 {
                let startx: i32 = b * 24 - a * 24 + screen.x;
                let starty: i32 = b * 12 + a * 12 + screen.y;
                let index = a * 64 + 2 * b;
                let t = self.tiles[index as usize];
                let current_tile = (t >> 16) as u16;
                let current_subtile = (t & 0xFFFF) as u16;
                match self.tile_ref.get(&current_tile) {
                    Some(ts) => {
                        ts.draw_left(startx, starty, current_subtile, canvas);
                    }
                    _ => {}
                }
                let t = self.tiles[(index + 1) as usize];
                let current_tile = (t >> 16) as u16;
                let current_subtile = (t & 0xFFFF) as u16;
                match self.tile_ref.get(&current_tile) {
                    Some(ts) => {
                        ts.draw_right(startx, starty, current_subtile, canvas);
                    }
                    _ => {}
                }
            }
        }
    }
}

impl MapSegment {
    pub fn to_gui<'a>(
        self,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> MapSegmentGui<'a> {
        let mut tilesets = HashSet::new();
        for tiles in &self.tiles {
            let tileset = (tiles >> 16) as u16;
            tilesets.insert(tileset);
        }
        let mut tr = HashMap::new();
        for tileset in &tilesets {
            if !tr.contains_key(tileset) {
                let t = r.tilesets.get_or_load(*tileset, || {
                    let _e = send.blocking_send(MessageToAsync::LoadTileset(*tileset));
                });
                if let Some(t) = t {
                    tr.insert(*tileset, t);
                }
            }
        }
        MapSegmentGui {
            tile_ref: tr,
            tilesets: tilesets,
            tiles: self.tiles,
            attributes: self.attributes,
            mystery1: self.mystery1,
            objects: self.objects,
            switches: self.switches,
            x: self.x,
            y: self.y,
            mapnum: self.mapnum,
        }
    }

    pub fn empty_segment() -> Self {
        Self {
            tiles: [1; 64 * 128],
            attributes: [0; 64 * 128],
            mystery1: Vec::new(),
            objects: Vec::new(),
            switches: Vec::new(),
            x: 32768,
            y: 32768,
            mapnum: 0,
        }
    }

    pub fn get_map_name(x: u16, y: u16) -> String {
        let modx = (x >> 6) + 0x7e00;
        let mody = (y >> 6) + 0x7e00;
        format!("{:4x}{:4x}", modx, mody)
    }

    pub async fn load_map_seg(
        cursor: &mut std::io::Cursor<&Vec<u8>>,
        x: u16,
        y: u16,
        mapnum: u16,
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
            mapnum: mapnum,
        })
    }

    pub async fn load_map_s32(
        cursor: &mut std::io::Cursor<&Vec<u8>>,
        x: u16,
        y: u16,
        mapnum: u16,
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
            mapnum: mapnum,
        })
    }
}
