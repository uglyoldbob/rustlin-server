use crate::Loadable::Loaded;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Duration;

use std::fs;

mod sprites;
use crate::sprites::*;

mod pack;
use crate::pack::*;

mod font;
use crate::font::*;

mod exception;
use crate::exception::*;

mod mode;
use crate::mode::*;

mod mouse;
use crate::mouse::*;

mod keyboard;

mod resources;
use crate::resources::*;

mod startup;

pub fn main() {
    startup::startup(DrawMode::GameLoader);
}
