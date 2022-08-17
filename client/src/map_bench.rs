#![allow(dead_code)]
use crate::map::MapSegment;
use crate::map::MapSegmentGui;
use crate::map::TileSet;
use crate::map::TileSetGui;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sdl2::render::TextureCreator;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

mod pack;
use crate::pack::*;
mod font;
mod startup;
use crate::font::*;
mod exception;
use crate::exception::*;
mod mode;
use crate::mode::*;
mod keyboard;
mod mouse;
mod resources;
use crate::resources::*;
mod widgets;

async fn async_bench1(
    mut r: tokio::sync::mpsc::Receiver<MessageToAsync>,
    s: tokio::sync::mpsc::Sender<MessageFromAsync>,
    resource_path: PathBuf,
) {
    let mut res: Resources = Resources::new();
    loop {
        if let Some(msg) = r.recv().await {
            match msg {
                MessageToAsync::LoadResources(path) => {
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
                        if let Err(e) = &ms {
                            Some(MapSegment::empty_segment(x, y, map))
                        } else {
                            ms.ok()
                        }
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
                            if let Err(e) = &ms {
                                Some(MapSegment::empty_segment(x, y, map))
                            } else {
                                ms.ok()
                            }
                        } else {
                            Some(MapSegment::empty_segment(x, y, map))
                        }
                    };
                    if let Some(mapseg) = ms {
                        let _e = s
                            .send(MessageFromAsync::MapSegment(map, x, y, Box::new(mapseg)))
                            .await;
                    } 
                }
                MessageToAsync::LoadTileset(id) => {
                    if let Some(p) = &mut res.packs {
                        let name = format!("{}.til", id);
                        let data = p.tile.raw_file_contents(name.clone()).await;
                        if let Some(data) = data {
                            let mut cursor = std::io::Cursor::new(&data);
                            let tileset = TileSet::decode_tileset_data(&mut cursor).await;
                            if let Some(t) = tileset {
                                let _e = s.send(MessageFromAsync::Tileset(id, t)).await;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn load_map<'a>(
    r: &mut tokio::sync::mpsc::Receiver<MessageFromAsync>,
    s: tokio::sync::mpsc::Sender<MessageToAsync>,
    map: u16,
    x: u16,
    y: u16,
) -> MapSegmentGui<'a> {
    let e = s.blocking_send(MessageToAsync::LoadMapSegment(map, x, y));
    loop {
        if let Ok(msg) = r.try_recv() {
            match msg {
                MessageFromAsync::MapSegment(_map, _x, _y, data) => {
                    let data = data.clone();
                    let ms = data.to_gui();
                    return ms;
                }
                _ => {}
            }
        }
    }
}

pub fn load_tileset<'a, T>(
    r: &mut tokio::sync::mpsc::Receiver<MessageFromAsync>,
    s: tokio::sync::mpsc::Sender<MessageToAsync>,
    tc: &'a TextureCreator<T>,
    set: u32,
) -> TileSetGui<'a> {
    let e = s.blocking_send(MessageToAsync::LoadTileset(set));
    loop {
        if let Ok(msg) = r.try_recv() {
            match msg {
                MessageFromAsync::Tileset(id, tileset) => {
                    let data = tileset.clone();
                    let ms = data.to_gui(tc);
                    return ms;
                }
                _ => {}
            }
        }
    }
}



pub fn bench1(c: &mut Criterion) {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.push("client-settings.ini");
    println!("Reading settings from {}", d.display());
    let settings_file = fs::read_to_string(d.into_os_string().into_string().unwrap());
    let settings_con = match settings_file {
        Ok(con) => con,
        Err(_) => "".to_string(),
    };
    let mut settings = configparser::ini::Ini::new();
    let settings_result = settings.read(settings_con);
    if let Err(e) = settings_result {
        println!("Failed to read settings {}", e);
    }

    let resources = settings.get("general", "resources").unwrap();

    let mut d = PathBuf::from(resources);

    let (mut s1, r1) = tokio::sync::mpsc::channel(100);
    let (s2, mut r2) = tokio::sync::mpsc::channel(100);
    s1.blocking_send(MessageToAsync::LoadResources(d.as_os_str().to_str().unwrap().to_string()));
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async_bench1(r1, s2, d.clone()));



    let mut r2 = RefCell::new(r2);
    let mut group = c.benchmark_group("map loading");
    group.bench_function("map 1", |b|{
        b.iter(||load_map(r2.get_mut(), s1.clone(), 4, 32768, 32768));
    });
    drop(rt);

    let (mut s1, r1) = tokio::sync::mpsc::channel(100);
    let (s2, mut r2) = tokio::sync::mpsc::channel(100);
    s1.blocking_send(MessageToAsync::LoadResources(d.as_os_str().to_str().unwrap().to_string()));
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async_bench1(r1, s2, d.clone()));
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut vid_win = video.window("benchmark", 640, 480);
    let mut window = vid_win.position_centered();
    let window = window.opengl().build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();

    let mut r2 = RefCell::new(r2);
    group.bench_function("tileset 0", |b| {
        b.iter(||load_tileset(r2.get_mut(), s1.clone(), &tc, 0));
    });

    group.finish();
}

criterion_group!(benches, bench1);
criterion_main!(benches);
