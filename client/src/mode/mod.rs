use crate::GameResources;
use crate::Loadable::*;
use crate::MessageFromAsync;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

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
pub struct ExplorerMenu {}

impl ExplorerMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameMode for ExplorerMenu {
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
            match &r.pngs[&value] {
                Unloaded => {
                    r.pngs.insert(value, Loading);
                    send.blocking_send(MessageToAsync::LoadPng(value));
                }
                Loading => {}
                Loaded(t) => {
                    canvas.copy(t, None, None);
                }
            }
        } else {
            r.pngs.insert(value, Loading);
            send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = 330;
        if r.imgs.contains_key(&value) {
            match &r.imgs[&value] {
                Unloaded => {
                    r.imgs.insert(value, Loading);
                    send.blocking_send(MessageToAsync::LoadImg(value));
                }
                Loading => {}
                Loaded(t) => {
                    canvas.copy(
                        t.texture(),
                        None,
                        Rect::new(241, 385, t.width().into(), t.height().into()),
                    );
                }
            }
        } else {
            r.imgs.insert(value, Loading);
            send.blocking_send(MessageToAsync::LoadImg(value));
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
