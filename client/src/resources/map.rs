use rand::Rng;
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
            x: (24 * (self.b as i32) + 24 * (self.a as i32)) as i32 - self.x0,
            y: (12 * (self.b as i32) - 12 * (self.a as i32)) as i32 - self.y0,
            x0: self.x0,
            y0: self.y0,
        }
    }

    ///Converts the given map coordinate to the top left pixel coordinate, like to_screen
    pub fn screen(&self, a: u16, b: u16) -> ScreenCoordinate {
        ScreenCoordinate {
            x: (24 * (b as i32) + 24 * (a as i32)) as i32 - self.x0,
            y: (12 * (b as i32) - 12 * (a as i32)) as i32 - self.y0,
            x0: self.x0,
            y0: self.y0,
        }
    }

    pub fn delta(a: i32, b: i32) -> (i32, i32) {
        (24 * b + 24 * a, 12 * b - 12 * a)
    }

    /// This function builds a MapCoordinate that places the dead center of the tile at the given screen coordinates
    pub fn build(a: u16, b: u16, x1: u32, y1: u32) -> Self {
        Self {
            a: a,
            b: b,
            x0: 24 * (b as i32) + 24 * (a as i32) - (x1 as i32) + 24,
            y0: 12 * (b as i32) - 12 * (a as i32) - (y1 as i32) + 12,
        }
    }
}

impl ScreenCoordinate {
    pub fn to_map(&self) -> MapCoordinate {
        MapCoordinate {
            a: (((self.x as i32) + self.x0 - 2 * (self.y as i32) - 2 * self.y0) / 48) as u16,
            b: ((2 * (self.y as i32) + 2 * self.y0 + (self.x as i32) + self.x0) / 48) as u16,
            x0: self.x0,
            y0: self.y0,
        }
    }

    pub fn map_coordinates(&self, x: i16, y: i16) -> (u16, u16) {
        let x1 = x - 24;
        let y1 = y - 12;
        let a = (((x1 as i32) + self.x0 - 2 * (y1 as i32) - 2 * self.y0) / 48) as u16;
        let x1 = x - 12;
        let y1 = y - 6;
        let b = ((2 * (y1 as i32) + 2 * self.y0 + (x1 as i32) + self.x0) / 48) as u16;
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
            (-1, -1, -48, 0),
            (-1, 0, -24, 12),
            (-1, 1, 0, 24),
            (0, -1, -24, -12),
            (0, 1, 24, 12),
            (1, -1, 0, -24),
            (1, 0, 24, -12),
            (1, 1, 48, 0),
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
        subtile: u8,
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
        subtile: u8,
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
        subtile: u8,
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
                        mirrored_tile_data[i * 48 + 24 - width + j] = d;
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
    x: u8,
    y: u8,
    h: u8,
    data: u32,
}

#[derive(Clone)]
pub struct MapObject {
    tiles: Vec<TileData>,
}

#[derive(Clone)]
pub struct MapSegmentGui<'a> {
    tile_ref: HashMap<u32, Rc<TileSetGui<'a>>>,
    tilesets: HashSet<u32>,
    tiles: [u32; 64 * 128],
    /// Bit definitions
    /// 0 = obstacle present
    /// 1 =
    attributes: [u16; 64 * 128],
    extra_floor_tiles: Vec<(u8, u8, u32)>,
    objects: Vec<MapObject>,
    min_object_depth: u8,
    max_object_depth: u8,
    switches: Vec<(u8, u8, u16, u8)>,
    x: u16,
    y: u16,
    mapnum: u16,
    partial: bool,
}

#[derive(Clone)]
pub struct MapSegment {
    tiles: [u32; 64 * 128],
    attributes: [u16; 64 * 128],
    extra_floor_tiles: Vec<(u8, u8, u32)>,
    objects: Vec<MapObject>,
    min_object_depth: u8,
    max_object_depth: u8,
    switches: Vec<(u8, u8, u16, u8)>,
    x: u16,
    y: u16,
    mapnum: u16,
    tilesets: HashSet<u32>,
    partial: bool,
}

impl<'a> MapSegmentGui<'a> {
    pub fn get_mapnum(&self) -> u16 {
        self.mapnum
    }
    pub fn combined(&self) -> u32 {
        let c: u32 = (self.x as u32) << 16 | (self.y as u32);
        c
    }

    pub fn check_tilesets(
        &mut self,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> bool {
        let mut done = true;
        for tileset in &self.tilesets {
            if !self.tile_ref.contains_key(tileset) {
                done = false;
                let t = r.tilesets.get_or_load(*tileset, || {
                    let _e = send.blocking_send(MessageToAsync::LoadTileset(*tileset));
                });
                if let Some(t) = t {
                    self.tile_ref.insert(*tileset, t);
                }
            }
        }
        done
    }

    pub fn draw_objects<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut sdl2::render::Canvas<T>,
        map: &MapCoordinate,
        _r: &mut GameResources,
        layer: u8,
    ) {
        let screen = map.screen(self.x, self.y);
        if layer == 0 {
            for (x, y, c) in &self.extra_floor_tiles {
                let a = (x / 2) as i32;
                let b = *y as i32;
                let tile = c >> 8;
                let subtile = (c & 0xFF) as u8;
                let mut startx: i32 = b * 24 + a * 24 + screen.x;
                if (x % 2) == 1 {
                    startx += 24;
                }
                let starty: i32 = b * 12 - a * 12 + screen.y;
                match self.tile_ref.get(&tile) {
                    Some(ts) => {
                        ts.draw_tile(startx, starty, subtile, canvas);
                    }
                    _ => {}
                }
            }
        }

        //for the objects displayed, check self.switches
        //(x,y,obj,d)
        //if player is on a tile indicated by (x/2,y), the obj obj should be half-transparent
        //d is unknown usage

        for o in &self.objects {
            for tdata in &o.tiles {
                if tdata.h == layer {
                    let a = (tdata.x / 2) as i32;
                    let b = tdata.y as i32;
                    let tile = tdata.data >> 8;
                    let subtile = (tdata.data & 0xFF) as u8;
                    let mut startx: i32 = b * 24 + a * 24 + screen.x;
                    if (tdata.x % 2) == 1 {
                        startx += 24;
                    }
                    let starty: i32 = b * 12 - a * 12 + screen.y;
                    match self.tile_ref.get(&tile) {
                        Some(ts) => {
                            ts.draw_tile(startx, starty, subtile, canvas);
                        }
                        _ => {}
                    }
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
        let mut rng = rand::thread_rng();
        let random = rng.gen::<f32>();
        if self.partial && (random < 0.5) {
            //return;
        }
        let screen = map.screen(self.x, self.y);
        for a in 0..64 {
            for b in 0..64 {
                let startx: i32 = b * 24 + a * 24 + screen.x;
                let starty: i32 = b * 12 - a * 12 + screen.y;
                let index = b * 128 + 2 * a;
                let t = self.tiles[index as usize];
                let current_tile = (t >> 8) as u32;
                let current_subtile = (t & 0xFF) as u8;
                match self.tile_ref.get(&current_tile) {
                    Some(ts) => {
                        ts.draw_left(startx, starty, current_subtile, canvas);
                    }
                    _ => {}
                }
                let t = self.tiles[(index + 1) as usize];
                let current_tile = (t >> 8) as u32;
                let current_subtile = (t & 0xFF) as u8;
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
        MapSegmentGui {
            tile_ref: HashMap::new(),
            tilesets: self.tilesets,
            tiles: self.tiles,
            attributes: self.attributes,
            extra_floor_tiles: self.extra_floor_tiles,
            objects: self.objects,
            switches: self.switches,
            x: self.x,
            y: self.y,
            mapnum: self.mapnum,
            min_object_depth: self.min_object_depth,
            max_object_depth: self.max_object_depth,
            partial: self.partial,
        }
    }

    pub fn empty_segment(x: u16, y: u16, map: u16) -> Self {
        let mut ts = HashSet::new();
        ts.insert(0);
        Self {
            tiles: [0; 64 * 128],
            attributes: [1; 64 * 128],
            extra_floor_tiles: Vec::new(),
            objects: Vec::new(),
            switches: Vec::new(),
            x: x,
            y: y,
            mapnum: map,
            tilesets: ts,
            min_object_depth: 0,
            max_object_depth: 0,
            partial: false,
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
    ) -> Result<Self, String> {
        let mut ts = HashSet::new();
        let mut t = [0; 64 * 128];
        for t in t.iter_mut() {
            let data = cursor.read_u16_le().await.ok().ok_or("Not enough data")? as u32;
            ts.insert(data >> 8);
            *t = data;
        }

        let _mystery1 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;

        let mut t2 = [0; 64 * 128];
        for t in t2.iter_mut() {
            *t = cursor.read_u8().await.ok().ok_or("Not enough data")?;
        }

        let mystery2 = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        println!("num 2 is {} at 0x{:x}", mystery2, cursor.position());
        for _ in 0..mystery2 {
            cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            let _m1 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        }

        let mystery3 = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        println!("num 3 is {} at 0x{:x}", mystery3, cursor.position());
        for _ in 0..mystery3 {
            let _m1 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            let _m2 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        }

        let num_tilesets = cursor.read_u8().await.ok().ok_or("Not enough data")?;
        println!(
            "num tilesets is {} at 0x{:x}",
            num_tilesets,
            cursor.position()
        );
        for _ in 0..num_tilesets {
            let _tileset = cursor.read_u8().await.ok().ok_or("Not enough data")?;
        }

        let num_portals = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        println!(
            "num portals is {} at 0x{:x}",
            num_portals,
            cursor.position()
        );
        for _ in 0..num_portals {
            cursor.read_u8().await.ok().ok_or("Not enough data")?;
            cursor.read_u8().await.ok().ok_or("Not enough data")?;
            cursor.read_u8().await.ok().ok_or("Not enough data")?;
            cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        }

        let mut v = Vec::new();
        cursor
            .read_to_end(&mut v)
            .await
            .ok()
            .ok_or("Not enough data")?;

        if v.len() > 0 {
            println!("There were {} bytes remaining", v.len());
            //return None;
        }
        Ok(Self {
            tiles: t,
            attributes: [0; 64 * 64 * 2],
            extra_floor_tiles: Vec::new(),
            objects: Vec::new(),
            switches: Vec::new(),
            x: x,
            y: y,
            mapnum: mapnum,
            tilesets: ts,
            min_object_depth: 0,
            max_object_depth: 0,
            partial: true,
        })
    }

    pub async fn load_map_s32(
        cursor: &mut std::io::Cursor<&Vec<u8>>,
        x: u16,
        y: u16,
        mapnum: u16,
    ) -> Result<Self, String> {
        let mut serr = String::new();
        let mut partial = false;
        let mut ts = HashSet::new();
        let mut t = [0; 64 * 128];
        for t in t.iter_mut() {
            let data = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
            ts.insert(data >> 8);
            *t = data;
        }
        ts.insert(2); // for testing
        let mut extra_floor_tiles = Vec::new();
        let quant = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        for _ in 0..quant {
            partial = true;
            let (a, b, c): (u8, u8, u32) = (
                cursor.read_u8().await.ok().ok_or("Not enough data")?,
                cursor.read_u8().await.ok().ok_or("Not enough data")?,
                cursor.read_u32_le().await.ok().ok_or("Not enough data")?,
            );
            ts.insert(c >> 8);
            extra_floor_tiles.push((a, b, c));
        }
        let mut attr = [0; 64 * 128];
        for t in attr.iter_mut() {
            *t = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
        }

        let num_objects = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        let mut objs = Vec::with_capacity(num_objects as usize);
        let mut min_depth = 255;
        let mut max_depth = 0;
        for _ in 0..num_objects {
            let _index = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            let num_tiles = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            let mut t = Vec::with_capacity(num_tiles as usize);
            for _ in 0..num_tiles {
                let b = cursor.read_u8().await.ok().ok_or("Not enough data")?;
                let c = cursor.read_u8().await.ok().ok_or("Not enough data")?;
                if b == 205 && c == 205 {
                    for _ in 0..5 {
                        cursor.read_u8().await.ok().ok_or("Not enough data")?;
                    }
                    serr.push_str(&format!("This map segment feature is unimplemented?\n"));
                    return Err(serr);
                } else {
                    let h = cursor.read_u8().await.ok().ok_or("Not enough data")?;
                    let data = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
                    ts.insert(data >> 8);
                    if min_depth > h {
                        min_depth = h;
                    }
                    if max_depth < h {
                        max_depth = h;
                    }
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
        let num_switches = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        let mut switches = Vec::with_capacity(num_switches as usize);
        for _ in 0..num_switches {
            let mys1 = cursor.read_u8().await.ok().ok_or("Not enough data")?;
            let mys2 = cursor.read_u8().await.ok().ok_or("Not enough data")?;
            let val = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
            let mys3 = cursor.read_u8().await.ok().ok_or("Not enough data")?;
            switches.push((mys1, mys2, val, mys3));
        }
        let num_tilesets = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        for _ in 0..num_tilesets {
            let val = cursor.read_u32_le().await.ok().ok_or("Not enough data")?;
        }

        // if there is data left
        let amount_portal = cursor.read_u16_le().await;
        if let Ok(num_portal) = amount_portal {
            serr.push_str(&format!(
                "There are {} portals at 0x{:x}\n",
                num_portal,
                cursor.position()
            ));
            for _ in 0..num_portal {
                let mys1 = cursor.read_u8().await.ok().ok_or(serr.clone())?;
                for _ in 0..mys1 {
                    cursor.read_u8().await.ok().ok_or(serr.clone())?;
                }
                let mys2 = cursor.read_u8().await.ok().ok_or(serr.clone())?;
                let mys3 = cursor.read_u8().await.ok().ok_or(serr.clone())?;
                let mys4 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
                let mys5 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
                let mys6 = cursor.read_u16_le().await.ok().ok_or("Not enough data")?;
                serr.push_str(&format!(
                    "portal data {} {} {} {} {}\n",
                    mys2, mys3, mys4, mys5, mys6
                ));
            }
        }

        let amount_stuff = cursor.read_u16_le().await;
        if let Ok(amount) = amount_stuff {
            serr.push_str(&format!(
                "There are {} stuff at 0x{:x}\n",
                amount,
                cursor.position()
            ));
            for _ in 0..amount {
                let a = cursor.read_u16_le().await.ok().ok_or_else(|| {
                    let mut serr = serr.clone();
                    serr.push_str("Not enough data");
                    println!("{}", serr);
                    serr
                })?;
                let b = cursor.read_u16_le().await.ok().ok_or_else(|| {
                    let mut serr = serr.clone();
                    serr.push_str("Not enough data 2");
                    println!("{}", serr);
                    serr
                })?;
                let c = cursor.read_u16_le().await.ok().ok_or_else(|| {
                    let mut serr = serr.clone();
                    serr.push_str("Not enough data 3");
                    println!("{}", serr);
                    serr
                })?;
                serr.push_str(&format!("Stuff data {} {} {}", a, b, c));
            }
        }
        let pos = cursor.position();
        let mut v = Vec::new();
        cursor
            .read_to_end(&mut v)
            .await
            .ok()
            .ok_or("Not enough data")?;

        if v.len() > 0 {
            serr.push_str(&format!(
                "There were {} bytes remaining at 0x{:x}\n",
                v.len(),
                pos
            ));
            return Err(serr);
        }

        println!("Map loaded ok\n{}", serr);

        Ok(Self {
            tiles: t,
            attributes: attr,
            extra_floor_tiles: extra_floor_tiles,
            objects: objs,
            switches: switches,
            x: x,
            y: y,
            mapnum: mapnum,
            tilesets: ts,
            min_object_depth: min_depth,
            max_object_depth: max_depth,
            partial: partial | (v.len() > 0),
        })
    }
}
