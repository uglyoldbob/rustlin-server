use crate::Pack;
use omnom::ReadExt;
use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::mixer::LoaderRWops;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;

pub mod stringtable;

use crate::widgets::CharacterDisplayType;

pub mod character_data;
use crate::resources::character_data::*;

pub mod sprite;
use crate::resources::sprite::*;

pub mod map;
use crate::resources::map::*;

#[derive(Clone)]
pub struct Img {
    width: u16,
    height: u16,
    unknown: u16,
    colorkey: u16,
    data: Vec<u8>,
}

impl Img {
    fn from_cursor(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        let width: u16 = cursor.read_le().ok()?;
        let height: u16 = cursor.read_le().ok()?;
        let unknown: u16 = cursor.read_le().ok()?;
        let colorkey: u16 = cursor.read_le().ok()?;

        let mut data = Vec::new();
        cursor.read_to_end(&mut data).ok()?;
        Some(Self {
            width: width,
            height: height,
            unknown: unknown,
            colorkey: colorkey,
            data: data,
        })
    }

    pub fn convert_img_data<'a, T>(&mut self, t: &'a TextureCreator<T>) -> Option<Texture<'a>> {
        let mut surface = Surface::from_data(
            self.data.as_mut_slice(),
            self.width as u32,
            self.height as u32,
            2 * self.width as u32,
            PixelFormatEnum::RGB555,
        )
        .unwrap();
        let color = sdl2::pixels::Color::from_u32(
            &sdl2::pixels::PixelFormat::try_from(PixelFormatEnum::RGB555).unwrap(),
            self.colorkey.into(),
        );
        let _e = surface.set_color_key(true, color);
        let text = Texture::from_surface(&surface, t).unwrap();
        Some(text)
    }
}

pub struct PackFiles {
    pub tile: Pack,
    pub text: Pack,
    pub sprite: Pack,
    pub sprites: Vec<Pack>,
}

impl PackFiles {
    fn get_hash_index(name: String) -> u8 {
        let mut j: u32 = 0;
        for c in name.chars() {
            j = j.wrapping_add(c as u32);
        }
        j as u8 & 0xf
    }

    pub fn load_png(&mut self, name: String) -> Option<Vec<u8>> {
        let hash = PackFiles::get_hash_index(name.clone());
        let mut contents = self.sprites[hash as usize].raw_file_contents(name.clone());
        if let None = contents {
            contents = self.sprite.raw_file_contents(name.clone());
        }
        if let Some(c) = &mut contents {
            if c[3] == 0x58 {
                println!("Need to fixup this png resource");
                c[3] = 0x47;
                let size = c.len();
                for i in 1..=size - 5 {
                    c[size - i] ^= c[size - i - 1];
                    c[size - i] ^= 0x52;
                }
            }
        }
        contents
    }

    pub fn load_img(&mut self, name: u16) -> Option<Img> {
        let name1 = format!("{}e.img", name);
        let hash = PackFiles::get_hash_index(name1.clone()) as usize;
        let mut data = self.sprites[hash].raw_file_contents(name1.clone());
        if let None = data {
            data = self.sprite.raw_file_contents(name1.clone());
            if let None = data {
                let name1 = format!("{}.img", name);
                let hash = PackFiles::get_hash_index(name1.clone()) as usize;
                data = self.sprites[hash].raw_file_contents(name1.clone());
                if let None = data {
                    data = self.sprite.raw_file_contents(name1.clone());
                }
            }
        }
        if let Some(d) = data {
            let mut cursor = std::io::Cursor::new(&d);
            let img = Img::from_cursor(&mut cursor);
            return img;
        } else {
            println!("Failed to load IMG{}", name);
        }
        None
    }

    pub fn load(path: String) -> Result<Self, ()> {
        let start = std::time::Instant::now();
        let mut packs: Vec<Pack> = Vec::new();
        for i in 0..16 {
            let mut pack = Pack::new(format!("{}/Sprite{:02}", path, i), false);
            let e = pack.load();
            for (key, v) in pack.file_extensions().iter() {
                println!("Contains {} {}", key, v);
            }
            println!("Time elapsed is {:?}", start.elapsed());
            if let Err(_a) = e {
                return Err(());
            }
            packs.push(pack);
        }
        let mut tile = Pack::new(format!("{}/Tile", path), false);
        let _e = tile.load();
        println!("TILE");
        for (key, v) in tile.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut text = Pack::new(format!("{}/Text", path), true);
        let _e = text.load();
        println!("TEXT");
        for (key, v) in text.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut sprite = Pack::new(format!("{}/Sprite", path), false);
        let _e = sprite.load();
        println!("SPRITE");
        for (key, v) in sprite.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        Ok(Self {
            tile: tile,
            text: text,
            sprite: sprite,
            sprites: packs,
        })
    }
}

pub enum Loadable<T> {
    Loading,
    Loaded(T),
}

pub enum LoadableReference<T> {
    Loading,
    NotFound,
    Strong(Rc<T>),
    Weak(Weak<T>),
}

impl<T> LoadableReference<T> {
    pub fn get_ref(&mut self) -> Option<Rc<T>> {
        match self {
            LoadableReference::Loading => None,
            LoadableReference::NotFound => None,
            LoadableReference::Strong(r) => {
                let s = r.clone();
                let w = Rc::<T>::downgrade(&r);
                *self = LoadableReference::Weak(w);
                Some(s)
            }
            LoadableReference::Weak(w) => w.upgrade(),
        }
    }
}

pub struct LoadableMap<T, U> {
    map: HashMap<T, LoadableReference<U>>,
}

impl<T, U> LoadableMap<T, U> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: T, val: U)
    where
        T: Eq + std::hash::Hash,
    {
        self.map
            .insert(key, LoadableReference::Strong(Rc::new(val)));
    }

    fn run_load<F>(&mut self, key: T, func: F) -> Option<Rc<U>>
    where
        F: FnOnce() -> Option<U>,
        T: Eq + std::hash::Hash,
    {
        let nobj = func();
        match nobj {
            Some(o) => {
                let sp = Rc::new(o);
                self.map
                    .insert(key, LoadableReference::Weak(Rc::downgrade(&sp)));
                Some(sp)
            }
            None => {
                self.map.insert(key, LoadableReference::NotFound);
                None
            }
        }
    }

    pub fn sync_get_or_load<F>(&mut self, key: T, func: F) -> Option<Rc<U>>
    where
        F: FnOnce() -> Option<U>,
        T: Eq + std::hash::Hash,
    {
        let check = self.map.get_mut(&key);
        match check {
            None => self.run_load(key, func),
            Some(v) => match v {
                LoadableReference::NotFound => None,
                LoadableReference::Loading => self.run_load(key, func),
                _ => {
                    let r = v.get_ref();
                    match r {
                        None => self.run_load(key, func),
                        Some(r) => Some(r),
                    }
                }
            },
        }
    }

    pub fn get_or_load<F>(&mut self, key: T, func: F) -> Option<Rc<U>>
    where
        F: FnOnce(),
        T: Eq + std::hash::Hash,
    {
        let check = self.map.get_mut(&key);
        match check {
            None => {
                self.map.insert(key, LoadableReference::Loading);
                func();
                None
            }
            Some(v) => match v {
                LoadableReference::Loading => None,
                _ => {
                    let r = v.get_ref();
                    if let None = r {
                        self.map.insert(key, LoadableReference::Loading);
                        func();
                    }
                    r
                }
            },
        }
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<T, LoadableReference<U>> {
        self.map.iter_mut()
    }
}

pub struct GameResources<'a, 'b, 'c> {
    resource_path: PathBuf,
    tc: &'a TextureCreator<WindowContext>,
    pngs: LoadableMap<u16, Texture<'a>>,
    imgs: LoadableMap<u16, Texture<'a>>,
    pub font: sdl2::ttf::Font<'b, 'c>,
    pub characters: [CharacterData; 8],
    pub sprites: HashMap<u32, Loadable<SpriteGui<'a>>>,
    pub sfx: HashMap<u16, Loadable<Chunk>>,
    pub tilesets: LoadableMap<u32, TileSetGui<'a>>,
    maps: HashMap<u16, LoadableMap<u32, Box<MapSegmentGui<'a>>>>,
    pub packs: Option<PackFiles>,
}

impl<'a, 'b, 'c> GameResources<'a, 'b, 'c> {
    pub fn new(
        font: sdl2::ttf::Font<'b, 'c>,
        path: String,
        tc: &'a TextureCreator<WindowContext>,
    ) -> Self {
        let mut chars = [
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
            CharacterData::new(),
        ];
        chars[1].t = CharacterDisplayType::MaleDarkElf;
        chars[2].t = CharacterDisplayType::Locked;

        let p = PathBuf::from(path.clone());
        let pack = PackFiles::load(path).ok();
        Self {
            resource_path: p,
            tc: tc,
            pngs: LoadableMap::new(),
            imgs: LoadableMap::new(),
            font: font,
            characters: chars,
            sprites: HashMap::new(),
            sfx: HashMap::new(),
            tilesets: LoadableMap::new(),
            maps: HashMap::new(),
            packs: pack,
        }
    }

    pub fn get_map(&mut self, map: u16) -> &mut LoadableMap<u32, Box<MapSegmentGui<'a>>> {
        if !self.maps.contains_key(&map) {
            self.maps.insert(map, LoadableMap::new());
        }
        self.maps.get_mut(&map).unwrap()
    }

    pub fn load_map_segment(
        map: u16,
        a: u16,
        b: u16,
        resource_path: PathBuf,
    ) -> Option<Box<MapSegmentGui<'a>>> {
        let mapn = MapSegment::get_map_name(a, b);
        let mut f = resource_path.clone();
        f.push("map");
        f.push(format!("{}", map));
        f.push(format!("{}.s32", mapn));
        let p = f.as_os_str().to_str().unwrap().to_string();
        let data = std::fs::File::open(p);
        let ms = if let Ok(mut data) = data {
            let mut buf = Vec::new();
            let _e = data.read_to_end(&mut buf);
            let mut c = std::io::Cursor::new(&buf);
            let ms = MapSegment::load_map_s32(&mut c, a, b, map);
            if let Err(e) = &ms {
                println!("Map s32 error");
                println!("{}", e);
                Some(MapSegment::empty_segment(a, b, map))
            } else {
                ms.ok()
            }
        } else {
            let mut f = resource_path.clone();
            f.push("map");
            f.push(format!("{}", map));
            f.push(format!("{}.seg", mapn));
            let p = f.as_os_str().to_str().unwrap().to_string();
            let data = std::fs::File::open(p);
            if let Ok(mut data) = data {
                let mut buf = Vec::new();
                let _e = data.read_to_end(&mut buf);
                let mut c = std::io::Cursor::new(&buf);
                let ms = MapSegment::load_map_seg(&mut c, a, b, map);
                if let Err(e) = &ms {
                    println!("Map seg error");
                    println!("{}", e);
                    Some(MapSegment::empty_segment(a, b, map))
                } else {
                    ms.ok()
                }
            } else {
                Some(MapSegment::empty_segment(a, b, map))
            }
        };
        if let Some(mapseg) = ms {
            Some(Box::new(mapseg.to_gui()))
        } else {
            println!("Map segment was not loaded");
            None
        }
    }

    pub fn get_map_segment(
        &mut self,
        map: u16,
        a: u16,
        b: u16,
    ) -> Option<Rc<Box<MapSegmentGui<'a>>>> {
        let resource_path = self.resource_path.clone();
        let lmap = self.get_map(map);
        let key = MapSegment::get_map_combined(a, b);
        let ms = lmap.sync_get_or_load(key, || None);
        if let None = ms {
            let nms = GameResources::load_map_segment(map, a, b, resource_path);
            if let Some(mut ms) = nms {
                ms.check_tilesets(self);
                self.get_map(map).insert(key, ms);
                self.get_map(map).sync_get_or_load(key, || None)
            } else {
                None
            }
        } else {
            ms
        }
    }

    pub fn get_or_load_sprite(&mut self, a: u16, b: u16) -> Option<&SpriteGui> {
        let id = (a as u32) << 16 | (b as u32);

        if !self.sprites.contains_key(&id) {
            let name = format!("{}-{}.spr", a, b);
            if let Some(p) = &mut self.packs {
                let hash = PackFiles::get_hash_index(name.clone());
                let mut contents = p.sprites[hash as usize].raw_file_contents(name.clone());
                if let None = contents {
                    contents = p.sprite.raw_file_contents(name.clone());
                }
                if let Some(c) = &contents {
                    println!("Sprite file is {} file", c.len());
                    let mut cursor = std::io::Cursor::new(c);
                    let spr = Sprite::parse_sprite(&mut cursor);
                    if let Some(spr) = spr {
                        let sprite = spr.to_gui(&self.tc);
                        self.sprites.insert(id, Loadable::Loaded(sprite));
                    } else {
                        println!("Failed to load sprite file {}", name);
                    }
                }
            }
        }
        let t = if let Some(s) = self.sprites.get(&id) {
            match s {
                Loadable::Loaded(t) => Some(t),
                _ => None,
            }
        } else {
            None
        };
        t
    }

    pub fn get_or_load_sfx(&mut self, i: u16) -> Option<&Chunk> {
        if self.sfx.contains_key(&i) {
            let t = self.sfx.get(&i);
            if let Some(t) = t {
                if let Loadable::Loaded(t) = t {
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let mut f = self.resource_path.clone();
            f.push("sound");
            f.push(format!("{}.wav", i));
            let f = f.as_os_str().to_str().unwrap().to_string();
            let data = std::fs::File::open(f);
            if let Ok(mut data) = data {
                let mut c = Vec::new();
                data.read_to_end(&mut c).unwrap();
                let rwops = sdl2::rwops::RWops::from_bytes(&c[..]).unwrap();
                let chnk = rwops.load_wav().unwrap();
                self.sfx.insert(i, Loadable::Loaded(chnk));
                if let Some(t) = self.sfx.get(&i) {
                    match t {
                        Loadable::Loaded(t) => Some(t),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn load_tileset(&mut self, i: u32) -> Option<TileSetGui<'a>> {
        if let Some(p) = &mut self.packs {
            let name = format!("{}.til", i);
            let data = p.tile.raw_file_contents(name.clone());
            if let Some(data) = data {
                let mut cursor = std::io::Cursor::new(&data);
                let tileset = TileSet::decode_tileset_data(&mut cursor);
                if let Some(t) = tileset {
                    let t = t.to_gui(&self.tc);
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_or_load_tileset(&mut self, i: u32) -> Option<Rc<TileSetGui<'a>>> {
        self.tilesets.sync_get_or_load(i, || {
            if let Some(p) = &mut self.packs {
                let name = format!("{}.til", i);
                let data = p.tile.raw_file_contents(name.clone());
                if let Some(data) = data {
                    let mut cursor = std::io::Cursor::new(&data);
                    let tileset = TileSet::decode_tileset_data(&mut cursor);
                    if let Some(t) = tileset {
                        let t = t.to_gui(&self.tc);
                        Some(t)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn get_or_load_img(&mut self, i: u16) -> Option<Rc<Texture<'a>>> {
        self.imgs.sync_get_or_load(i, || {
            if let Some(p) = &mut self.packs {
                let data = p.load_img(i);
                match data {
                    Some(mut d) => {
                        let img = d.convert_img_data(&self.tc);
                        img
                    }
                    None => None,
                }
            } else {
                None
            }
        })
    }

    pub fn get_or_load_png(&mut self, i: u16) -> Option<Rc<Texture<'a>>> {
        self.pngs.sync_get_or_load(i, || {
            if let Some(p) = &mut self.packs {
                let name2 = format!("{}.png", i);
                let data = p.load_png(name2.clone());
                match data {
                    Some(d) => {
                        let png = self.tc.load_texture_bytes(&d);
                        match png {
                            Ok(mut a) => {
                                a.set_blend_mode(sdl2::render::BlendMode::Add);
                                Some(a)
                            }
                            Err(e) => {
                                println!("PNG {} fail {}", i, e);
                                println!("PNG DATA {:x?}", &d[0..25]);
                                None
                            }
                        }
                    }
                    None => {
                        println!("{} failed to load", name2);
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}
