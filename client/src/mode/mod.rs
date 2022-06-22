use crate::mouse::MouseEventOutput;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageFromAsync;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

/// Al of the various kinds of widgets that can exist in the game
pub enum Widget<'a> {
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
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
    }
    fn contains_point(x: u16, y: u16) -> bool {
        false
    }
}

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode {
    fn process_mouse(&mut self, events: &Vec<MouseEventOutput>);
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
    fn process_mouse(&mut self, events: &Vec<MouseEventOutput>) {
        for e in events {
            match e {
                MouseEventOutput::Move((x, y)) => {
                    println!("Moved the mouse to {} {}", x, y);
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                    println!("Left drag to {} {}", x, y);
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                    println!("Middle drag to {} {}", x, y);
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                    println!("Right drag to {} {}", x, y);
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    println!("Left click at {} {}", x, y);
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                    println!("Middle click at {} {}", x, y);
                }
                MouseEventOutput::RightClick((x, y)) => {
                    println!("Right click at {} {}", x, y);
                }
                MouseEventOutput::ExtraClick => {
                    println!("Extra click");
                }
                MouseEventOutput::Extra2Click => {
                    println!("Extra2 click");
                }
                MouseEventOutput::Scrolling(amount) => {
                    println!("Scrolled by {}", amount);
                }
            }
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
