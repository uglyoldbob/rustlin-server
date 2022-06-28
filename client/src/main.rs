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

mod resources;
use crate::resources::*;

mod startup;

fn make_dummy_texture<'a,T>(tc: &'a TextureCreator<T>) -> Texture<'a> {
	let mut data : Vec<u8>= vec![0; (4 * 4 * 2) as usize];
        let mut surf = sdl2::surface::Surface::from_data(
            data.as_mut_slice(),
            4,
            4,
            (2 * 4) as u32,
            PixelFormatEnum::RGB555,
        )
        .unwrap();
	surf.set_color_key(true, sdl2::pixels::Color::BLACK);
        Texture::from_surface(&surf, tc).unwrap()
}

pub fn main() {
    startup::startup(DrawMode::Explorer);
}
