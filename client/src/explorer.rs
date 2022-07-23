#![allow(dead_code)]

mod pack;
use crate::pack::*;

mod font;
use crate::font::*;

mod exception;
use crate::exception::*;

mod mode;
use crate::mode::*;

mod mouse;

mod keyboard;

mod resources;
use crate::resources::*;

mod startup;

pub fn main() {
    startup::startup(DrawMode::Explorer);
}
