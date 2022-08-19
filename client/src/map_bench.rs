#![allow(dead_code)]
use crate::map::MapSegment;
use crate::map::MapSegmentGui;
use crate::map::TileSet;
use crate::map::TileSetGui;
use crate::startup::EMBEDDED_FONT;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sdl2::render::TextureCreator;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

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

pub fn load_map<'a>(map: u16, x: u16, y: u16, rp: PathBuf) -> Option<Box<MapSegmentGui<'a>>> {
    GameResources::load_map_segment(map, x, y, rp)
}

pub fn load_tileset<'a, T>(
    tc: &'a TextureCreator<T>,
    r: &mut GameResources<'a, '_, '_>,
    set: u32,
) -> Option<TileSetGui<'a>> {
    r.load_tileset(set)
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

    let mut d = PathBuf::from(resources.clone());

    let mut group = c.benchmark_group("map loading");
    group.bench_function("map 1", |b| {
        b.iter(|| load_map(4, 32768, 32768, d.clone()));
    });

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut vid_win = video.window("benchmark", 640, 480);
    let mut window = vid_win.position_centered();
    let window = window.opengl().build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();

    let ttf_context = sdl2::ttf::init().unwrap();
    let efont = sdl2::rwops::RWops::from_bytes(EMBEDDED_FONT).unwrap();
    let font = ttf_context.load_font_from_rwops(efont, 14).unwrap();

    let mut game_resources = GameResources::new(font, resources.clone(), &tc);

    group.bench_function("tileset 0", |b| {
        b.iter(|| load_tileset(&tc, &mut game_resources, 0));
    });

    group.finish();
}

criterion_group!(benches, bench1);
criterion_main!(benches);
