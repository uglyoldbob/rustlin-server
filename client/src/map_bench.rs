#![allow(dead_code)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

pub fn bench1(c: &mut Criterion) {}

criterion_group!(benches, bench1);
criterion_main!(benches);
