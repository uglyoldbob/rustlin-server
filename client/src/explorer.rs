#![allow(dead_code)]

use crate::map::MapSegment;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

mod pack;
use crate::pack::*;

mod font;

mod exception;
use crate::exception::*;

mod mode;
use crate::mode::*;

mod mouse;

mod keyboard;

mod resources;
use crate::resources::*;

pub mod widgets;

mod startup;

async fn test_map_load() {
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
    d.push("map");

    let mut num_success = 0;

    let mut maps = Vec::new();
    let mut seg_maps = Vec::new();

    let mut s32maps = HashSet::new();

    let entries = fs::read_dir(&d).unwrap();
    for e in entries.into_iter() {
        if let Ok(e) = e {
            let path = e.path();
            let meta = fs::metadata(&path).unwrap();
            if meta.is_dir() {
                let mapnum = path.file_name().unwrap().to_string_lossy();
                let mapnum = mapnum.parse::<u16>().unwrap();
                let map_segs = fs::read_dir(path).unwrap();
                for e in map_segs.into_iter() {
                    if let Ok(e) = e {
                        let path = e.path();
                        let meta = fs::metadata(&path).unwrap();
                        if meta.is_file() {
                            if path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .ends_with(".s32")
                            {
                                let mapseg = path.file_stem().unwrap().to_string_lossy();
                                let mapseg = u32::from_str_radix(&mapseg, 16);
                                if let Ok(mapseg) = mapseg {
                                    let totalmapseg = (mapnum as u64) << 32 | mapseg as u64;
                                    s32maps.insert(totalmapseg);
                                    maps.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let entries = fs::read_dir(d).unwrap();
    for e in entries.into_iter() {
        if let Ok(e) = e {
            let path = e.path();
            let meta = fs::metadata(&path).unwrap();
            if meta.is_dir() {
                let mapnum = path.file_name().unwrap().to_string_lossy();
                let mapnum = mapnum.parse::<u16>().unwrap();
                let map_segs = fs::read_dir(path).unwrap();
                for e in map_segs.into_iter() {
                    if let Ok(e) = e {
                        let path = e.path();
                        let meta = fs::metadata(&path).unwrap();
                        if meta.is_file() {
                            if path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .ends_with(".seg")
                            {
                                let mapseg = path.file_stem().unwrap().to_string_lossy();
                                let mapseg = u32::from_str_radix(&mapseg, 16);
                                if let Ok(mapseg) = mapseg {
                                    let totalmapseg = (mapnum as u64) << 32 | mapseg as u64;
                                    if !s32maps.contains(&totalmapseg) {
                                        seg_maps.push(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    assert_ne!(0, seg_maps.len());

    let mut map_s32_failures = Vec::new();
    let mut map_seg_failures = Vec::new();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.push("map_testing_results.txt");
    let mut fo = std::fs::File::create(d).unwrap();

    let w = format!("Number of seg maps {}\n", seg_maps.len());
    fo.write_all(w.as_bytes())
        .expect("Failed writing results to file");

    let w = format!("Number of s32 maps {}\n", maps.len());
    fo.write_all(w.as_bytes())
        .expect("Failed writing results to file");

    for f in &maps {
        println!("Map file {}", f.display());
        let data = std::fs::File::open(f);
        if let Ok(mut data) = data {
            let mut buf = Vec::new();
            let _e = data.read_to_end(&mut buf);
            let mut c = std::io::Cursor::new(&buf);
            let ms = MapSegment::load_map_s32(&mut c, 32768, 32768, 0);
            match ms {
                Ok(_m) => {
                    num_success += 1;
                    let w = format!("Map success {}\n", f.display());
                    fo.write_all(w.as_bytes())
                        .expect("Failed writing results to file");
                }
                Err(e) => {
                    map_s32_failures.push(f);
                    let w = format!("Map failure {} is:\n{}\n", f.display(), e);
                    fo.write_all(w.as_bytes())
                        .expect("Failed writing results to file");
                }
            }
        }
    }

    for f in &seg_maps {
        println!("Map file {}", f.display());
        let data = std::fs::File::open(f);
        if let Ok(mut data) = data {
            let mut buf = Vec::new();
            let _e = data.read_to_end(&mut buf);
            let mut c = std::io::Cursor::new(&buf);
            let ms = MapSegment::load_map_seg(&mut c, 32768, 32768, 0);
            match ms {
                Ok(_m) => {
                    num_success += 1;
                    let w = format!("Map seg success {}\n", f.display());
                    fo.write_all(w.as_bytes())
                        .expect("Failed writing results to file");
                }
                Err(e) => {
                    map_seg_failures.push(f);
                    let w = format!("Map seg failure {} is:\n{}\n", f.display(), e);
                    fo.write_all(w.as_bytes())
                        .expect("Failed writing results to file");
                }
            }
        }
    }

    assert_eq!(maps.len() + seg_maps.len(), num_success);
}

#[test]
fn check_map_load() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(std::pin::Pin::from(Box::new(test_map_load())));
}

pub fn main() {
    startup::startup(DrawMode::Explorer);
}
