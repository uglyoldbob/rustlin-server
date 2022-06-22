use crate::GameResources;
use crate::Loadable::*;
use crate::MessageFromAsync;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

/// This trait is used for the widgets in the game
pub trait Widget {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    );
    fn active_pixel(x: u16, y: u16) -> bool {
        true
    }
    fn contains_point(x: u16, y: u16) -> bool;
}

pub enum WidgetEnum<'a> {
    PlainColorButton(PlainColorButton<'a>),
}

pub struct PlainColorButton<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
}

impl<'a> PlainColorButton<'a> {
    fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, w: u16, h: u16) -> Self {
        let mut data = vec![0x7f; (w * h * 2) as usize];
        let surf = sdl2::surface::Surface::from_data(
            &mut data[..],
            w as u32,
            h as u32,
            (2 * w) as u32,
            PixelFormatEnum::RGB555,
        )
        .unwrap();
        Self {
            t: surf.as_texture(tc).unwrap(),
            x: x,
            y: y,
        }
    }
}

impl<'a> Widget for PlainColorButton<'a> {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let q = self.t.query();
        canvas.copy(
            &self.t,
            None,
            Rect::new(self.x.into(), self.y.into(), q.width.into(), q.height.into()),
        );
    }
    fn contains_point(x: u16, y: u16) -> bool {
        false
    }
}

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode {
    fn parse_message(&mut self, m: &MessageFromAsync, r: &mut GameResources);
    fn parse_event(&mut self, e: sdl2::event::Event, r: &mut GameResources);
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    );
    /// Framerate is specified in frames per second
    fn framerate(&self) -> u8;
}

/// This is for exploring the resources of the game client
pub struct ExplorerMenu<'a> {
    b: PlainColorButton<'a>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        Self {
            b: PlainColorButton::new(tc, 50, 50, 50, 50),
        }
    }
}

impl<'a> GameMode for ExplorerMenu<'a> {
    fn parse_message(&mut self, m: &MessageFromAsync, r: &mut GameResources) {
        match m {
            MessageFromAsync::ResourceStatus(_b) => {}
            MessageFromAsync::StringTable(_name, _data) => {}
            MessageFromAsync::Png(_name, _data) => {}
            MessageFromAsync::Img(_name, _data) => {}
        }
    }

    fn parse_event(&mut self, e: sdl2::event::Event, r: &mut GameResources) {
        match e {
            _ => {}
        }
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let value = 811;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = 330;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                canvas.copy(
                    t,
                    None,
                    Rect::new(241, 385, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            send.blocking_send(MessageToAsync::LoadImg(value));
        }
        self.b.draw(canvas, r, send);
    }

    fn framerate(&self) -> u8 {
        20
    }
}
