#![allow(dead_code)]
use crate::map::MapSegment;
use crate::map::MapSegmentGui;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
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
    loop {
        if let Some(msg) = r.recv().await {
            match msg {
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


    let mut group = c.benchmark_group("map loading");
    group.bench_function("map 1", |b|{
        let (mut s1, r1) = tokio::sync::mpsc::channel(100);
        let (s2, mut r2) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.spawn(async_bench1(r1, s2, d.clone()));

        let mut r2 = RefCell::new(r2);
        b.iter(||load_map(r2.get_mut(), s1.clone(), 4, 32768, 32768));
    });
    group.finish();
}

criterion_group!(benches, bench1);
criterion_main!(benches);
