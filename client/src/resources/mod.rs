use crate::Font;
use crate::Pack;
use sdl2::mixer::Chunk;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;
use tokio::io::AsyncReadExt;

pub mod stringtable;
use crate::resources::stringtable::*;

use crate::widgets::CharacterDisplayType;

pub mod character_data;
use crate::resources::character_data::*;

pub mod sprite;
use crate::resources::sprite::*;

pub mod map;
use crate::resources::map::*;

use async_trait::async_trait;
/// Represents data and code that can be transferred to the async runtime for execution.
#[async_trait]
pub trait AsyncRunner: Send {
    async fn do_stuff(&mut self, res: &mut Resources);
}

#[derive(Clone)]
pub struct Img {
    width: u16,
    height: u16,
    unknown: u16,
    colorkey: u16,
    data: Vec<u8>,
}

impl Img {
    async fn from_cursor(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Option<Self> {
        let width = cursor.read_u16_le().await.ok()?;
        let height = cursor.read_u16_le().await.ok()?;
        let unknown = cursor.read_u16_le().await.ok()?;
        let colorkey = cursor.read_u16_le().await.ok()?;
        println!("IMG is {} x {} {} {}", width, height, unknown, colorkey);

        let mut data = Vec::new();
        cursor.read_to_end(&mut data).await.ok()?;
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

        //TODO set colorkey
        let text = Texture::from_surface(&surface, t).unwrap();
        Some(text)
    }
}

pub enum MessageToAsync {
    LoadResources(String),
    LoadFont(String),
    LoadSpriteTable,
    LoadTable(String),
    LoadPng(u16),
    LoadImg(u16),
    LoadRunner(Box<dyn AsyncRunner + Send>),
    LoadSprite(u16, u16),
    LoadSfx(u16),
    LoadTileset(u16),
    LoadMapSegment(u16, u16, u16),
}

pub enum MessageFromAsync {
    ResourceStatus(bool),
    StringTable(String, StringTable),
    Png(u16, Vec<u8>),
    Img(u16, Img),
    Sprite(u32, Sprite),
    Sfx(u16, Vec<u8>),
    Tileset(u16, TileSet),
    MapSegment(u16, u16, u16, Box<MapSegment>),
}

struct PackFiles {
    tile: Pack,
    text: Pack,
    sprite: Pack,
    sprites: Vec<Pack>,
}

impl PackFiles {
    fn get_hash_index(name: String) -> u8 {
        let mut j: u32 = 0;
        for c in name.chars() {
            j = j.wrapping_add(c as u32);
        }
        j as u8 & 0xf
    }

    pub async fn load_png(&mut self, name: String) -> Option<Vec<u8>> {
        let hash = PackFiles::get_hash_index(name.clone());
        let mut contents = self.sprites[hash as usize]
            .raw_file_contents(name.clone())
            .await;
        if let None = contents {
            contents = self.sprite.raw_file_contents(name.clone()).await;
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

    pub async fn load_img(&mut self, name: u16) -> Option<Img> {
        let name1 = format!("{}e.img", name);
        let hash = PackFiles::get_hash_index(name1.clone()) as usize;
        let mut data = self.sprites[hash].raw_file_contents(name1.clone()).await;
        if let None = data {
            data = self.sprite.raw_file_contents(name1.clone()).await;
            if let None = data {
                let name1 = format!("{}.img", name);
                let hash = PackFiles::get_hash_index(name1.clone()) as usize;
                data = self.sprites[hash].raw_file_contents(name1.clone()).await;
                if let None = data {
                    data = self.sprite.raw_file_contents(name1.clone()).await;
                }
            }
        }
        if let Some(d) = data {
            println!("Found IMG {}", name);
            let mut cursor = std::io::Cursor::new(&d);
            let img = Img::from_cursor(&mut cursor).await;
            return img;
        } else {
            println!("Failed to load IMG{}", name);
        }
        None
    }

    pub async fn load(path: String) -> Result<Self, ()> {
        let start = std::time::Instant::now();
        let mut packs: Vec<Pack> = Vec::new();
        for i in 0..16 {
            let mut pack = Pack::new(format!("{}/Sprite{:02}", path, i), false);
            let e = pack.load().await;
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
        let _e = tile.load().await;
        println!("TILE");
        for (key, v) in tile.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut text = Pack::new(format!("{}/Text", path), true);
        let _e = text.load().await;
        println!("TEXT");
        for (key, v) in text.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut sprite = Pack::new(format!("{}/Sprite", path), false);
        let _e = sprite.load().await;
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
    Strong(Rc<T>),
    Weak(Weak<T>),
}

impl<T> LoadableReference<T> {
    pub fn get_ref(&mut self) -> Option<Rc<T>> {
        match self {
            LoadableReference::Loading => None,
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
                _ => v.get_ref(),
            },
        }
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<T, LoadableReference<U>> {
        self.map.iter_mut()
    }
}

pub struct GameResources<'a, 'b, 'c> {
    tc: &'a TextureCreator<WindowContext>,
    pub pngs: HashMap<u16, Loadable<Texture<'a>>>,
    pub imgs: HashMap<u16, Loadable<Texture<'a>>>,
    pub font: sdl2::ttf::Font<'b, 'c>,
    pub characters: [CharacterData; 8],
    pub sprites: HashMap<u32, Loadable<SpriteGui<'a>>>,
    pub sfx: HashMap<u16, Loadable<Chunk>>,
    pub tilesets: LoadableMap<u16, TileSetGui<'a>>,
    maps: HashMap<u16, LoadableMap<u32, MapSegment>>,
}

impl<'a, 'b, 'c> GameResources<'a, 'b, 'c> {
    pub fn new(font: sdl2::ttf::Font<'b, 'c>, tc: &'a TextureCreator<WindowContext>) -> Self {
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
        Self {
            tc: tc,
            pngs: HashMap::new(),
            imgs: HashMap::new(),
            font: font,
            characters: chars,
            sprites: HashMap::new(),
            sfx: HashMap::new(),
            tilesets: LoadableMap::new(),
            maps: HashMap::new(),
        }
    }

    pub fn get_map(&mut self, map: u16) -> &mut LoadableMap<u32, MapSegment> {
        if !self.maps.contains_key(&map) {
            self.maps.insert(map, LoadableMap::new());
        }
        self.maps.get_mut(&map).unwrap()
    }
}

pub struct Resources {
    packs: Option<PackFiles>,
}

impl Resources {
    pub fn new() -> Self {
        Self { packs: None }
    }
}

pub async fn async_main(
    mut r: tokio::sync::mpsc::Receiver<MessageToAsync>,
    s: tokio::sync::mpsc::Sender<MessageFromAsync>,
) {
    println!("Async main");

    let mut resource_path: PathBuf = PathBuf::new();
    let mut res: Resources = Resources::new();

    loop {
        let message = r.recv().await;
        match message {
            None => break,
            Some(msg) => match msg {
                MessageToAsync::LoadRunner(mut r) => {
                    r.do_stuff(&mut res).await;
                }
                MessageToAsync::LoadResources(path) => {
                    resource_path = PathBuf::from(path.clone());
                    println!("Loading resources {}", path);
                    match PackFiles::load(path).await {
                        Ok(p) => {
                            res.packs = Some(p);
                            let _e = s.send(MessageFromAsync::ResourceStatus(true)).await;
                        }
                        Err(()) => {
                            let _e = s.send(MessageFromAsync::ResourceStatus(false)).await;
                        }
                    }
                }
                MessageToAsync::LoadFont(file) => {
                    let mut f = resource_path.clone();
                    f.push(file);
                    let path = f.as_os_str().to_str().unwrap().to_string();
                    let _font = Font::load(path).await;
                }
                MessageToAsync::LoadSpriteTable => {
                    let _st = SpriteTableEntry::load_embedded_table().await;
                }
                MessageToAsync::LoadTable(name) => {
                    if let Some(p) = &mut res.packs {
                        let data = p.text.decrypted_file_contents(name.clone()).await;
                        match data {
                            Some(d) => {
                                let table = StringTable::from(d);
                                let _e = s
                                    .send(MessageFromAsync::StringTable(name, table.clone()))
                                    .await;
                            }
                            None => {
                                println!("{} failed to load", name);
                            }
                        }
                    }
                }
                MessageToAsync::LoadPng(name) => {
                    if let Some(p) = &mut res.packs {
                        let name2 = format!("{}.png", name);
                        let data = p.load_png(name2.clone()).await;
                        match data {
                            Some(d) => {
                                let _e = s.send(MessageFromAsync::Png(name, d)).await;
                            }
                            None => {
                                println!("{} failed to load", name2);
                            }
                        }
                    }
                }
                MessageToAsync::LoadImg(name) => {
                    if let Some(p) = &mut res.packs {
                        let data = p.load_img(name).await;
                        match data {
                            Some(d) => {
                                let _e = s.send(MessageFromAsync::Img(name, d)).await;
                            }
                            None => {
                                println!("{} failed to load", name);
                            }
                        }
                    }
                }
                MessageToAsync::LoadSprite(a, b) => {
                    let name = format!("{}-{}.spr", a, b);
                    println!("Loading sprite {}", name);
                    let id = (a as u32) << 16 | (b as u32);
                    if let Some(p) = &mut res.packs {
                        let hash = PackFiles::get_hash_index(name.clone());
                        let mut contents = p.sprites[hash as usize]
                            .raw_file_contents(name.clone())
                            .await;
                        if let None = contents {
                            contents = p.sprite.raw_file_contents(name.clone()).await;
                        }
                        if let Some(c) = &contents {
                            println!("Sprite file is {} file", c.len());
                            let mut cursor = std::io::Cursor::new(c);
                            let spr = Sprite::parse_sprite(&mut cursor).await;
                            if let Some(spr) = spr {
                                println!("Success {}", name);
                                let _e = s.send(MessageFromAsync::Sprite(id, spr)).await;
                            } else {
                                println!("Failed to load sprite file {}", name);
                            }
                        }
                    }
                }
                MessageToAsync::LoadSfx(id) => {
                    let mut f = resource_path.clone();
                    f.push("sound");
                    f.push(format!("{}.wav", id));
                    let f = f.as_os_str().to_str().unwrap().to_string();
                    println!("I need to load {}", f);
                    let data = tokio::fs::File::open(f).await;
                    if let Ok(mut data) = data {
                        let mut c = Vec::new();
                        data.read_to_end(&mut c).await.unwrap();
                        let _e = s.send(MessageFromAsync::Sfx(id, c.clone())).await;
                    }
                }
                MessageToAsync::LoadTileset(id) => {
                    if let Some(p) = &mut res.packs {
                        let name = format!("{}.til", id);
                        println!("Loading {}.til", id);
                        let data = p.tile.raw_file_contents(name.clone()).await;
                        if let Some(data) = data {
                            println!("Decoding {}.til", id);
                            let mut cursor = std::io::Cursor::new(&data);
                            let tileset = TileSet::decode_tileset_data(&mut cursor).await;
                            if let Some(t) = tileset {
                                println!("Submitting {}.til", id);
                                let _e = s.send(MessageFromAsync::Tileset(id, t)).await;
                            }
                        }
                    }
                }
                MessageToAsync::LoadMapSegment(map, x, y) => {
                    let mapn = MapSegment::get_map_name(x, y);
                    let mut f = resource_path.clone();
                    f.push("map");
                    f.push(format!("{}", map));
                    f.push(format!("{}.s32", mapn));
                    let p = f.as_os_str().to_str().unwrap().to_string();
                    let data = tokio::fs::File::open(p).await;
                    let ms = if let Ok(mut data) = data {
                        let mut buf = Vec::new();
                        let _e = data.read_to_end(&mut buf).await;
                        let mut c = std::io::Cursor::new(&buf);
                        let ms = MapSegment::load_map_s32(&mut c, x, y, map).await;
                        ms
                    } else {
                        let mut f = resource_path.clone();
                        f.push("map");
                        f.push(format!("{}", map));
                        f.push(format!("{}.seg", mapn));
                        let p = f.as_os_str().to_str().unwrap().to_string();
                        let data = tokio::fs::File::open(p).await;
                        if let Ok(mut data) = data {
                            let mut buf = Vec::new();
                            let _e = data.read_to_end(&mut buf).await;
                            let mut c = std::io::Cursor::new(&buf);
                            let ms = MapSegment::load_map_seg(&mut c, x, y, map).await;
                            ms
                        } else {
                            None
                        }
                    };
                    if let Some(mapseg) = ms {
                        let _e = s
                            .send(MessageFromAsync::MapSegment(map, x, y, Box::new(mapseg)))
                            .await;
                    }
                }
            },
        }
    }
}
