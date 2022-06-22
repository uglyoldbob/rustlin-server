use crate::Font;
use crate::Pack;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;

pub mod stringtable;
use crate::resources::stringtable::*;

use async_trait::async_trait;
/// Represents a variety of lynx modules.
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
}

pub enum MessageFromAsync {
    ResourceStatus(bool),
    StringTable(String, StringTable),
    Png(u16, Vec<u8>),
    Img(u16, Img),
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
        let contents = self.sprites[hash as usize]
            .raw_file_contents(name.clone())
            .await;
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
            for (key,v) in pack.file_extensions().iter() {
                println!("Contains {} {}", key, v);
            }
            println!("Time elapsed is {:?}", start.elapsed());
            if let Err(_a) = e {
                return Err(());
            }
            packs.push(pack);
        }
        let mut tile = Pack::new(format!("{}/Tile", path), false);
        tile.load().await;
        println!("TILE");
        for (key, v) in tile.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut text = Pack::new(format!("{}/Text", path), true);
        text.load().await;
        println!("TEXT");
        for (key,v) in text.file_extensions().iter() {
            println!("Contains {} {}", key, v);
        }
        println!("Time elapsed is {:?}", start.elapsed());
        let mut sprite = Pack::new(format!("{}/Sprite", path), false);
        sprite.load().await;
        println!("SPRITE");
        for (key,v) in sprite.file_extensions().iter() {
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

pub struct GameResources<'a> {
    pub pngs: HashMap<u16, Loadable<Texture<'a>>>,
    pub imgs: HashMap<u16, Loadable<Texture<'a>>>,
}

impl<'a> GameResources<'a> {
    pub fn new() -> Self {
        Self {
            pngs: HashMap::new(),
            imgs: HashMap::new(),
        }
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
    mut s: tokio::sync::mpsc::Sender<MessageFromAsync>,
) {
    println!("Async main");

    let mut resource_path: String = "".to_string();
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
                    resource_path = path.clone();
                    println!("Loading resources {}", path);
                    match PackFiles::load(path).await {
                        Ok(p) => {
                            res.packs = Some(p);
                            s.send(MessageFromAsync::ResourceStatus(true)).await;
                        }
                        Err(()) => {
                            s.send(MessageFromAsync::ResourceStatus(false)).await;
                        }
                    }
                }
                MessageToAsync::LoadFont(file) => {
                    let path = format!("{}/{}", resource_path, file);
                    let font = Font::load(path).await;
                }
                MessageToAsync::LoadSpriteTable => {
                    let data = include_bytes!("sprite_table.txt");
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
            },
        }
    }
}
