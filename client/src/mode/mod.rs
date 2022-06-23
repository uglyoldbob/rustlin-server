use crate::mouse::MouseEventOutput;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

pub enum DrawMode {
    Explorer,
    Login,
}

/// The kind of request that can be issued by a draw mode
pub enum DrawModeRequest {
    ChangeDrawMode(DrawMode),
}

/// All of the various kinds of widgets that can exist in the game
pub enum Widget<'a> {
    PlainColorButton(PlainColorButton<'a>),
}

impl<'a> Widget<'a> {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        match self {
            Widget::PlainColorButton(button) => {
                button.draw(canvas, r, send);
            }
        }
    }
    fn contains(&self, x: i16, y: i16) -> bool {
        match self {
            Widget::PlainColorButton(button) => button.contains_point(x, y),
        }
    }
    fn left_click(&mut self) {
        match self {
            Widget::PlainColorButton(button) => {
                button.clicked();
                println!("Clicked the button");
            }
        }
    }
    fn was_clicked(&mut self) -> bool {
        match self {
            Widget::PlainColorButton(button) => button.was_clicked(),
        }
    }
}

pub struct PlainColorButton<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
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
            clicked: false,
        }
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let q = self.t.query();
        let _e = canvas.copy(
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
    fn contains_point(&self, x: i16, y: i16) -> bool {
        let x = if x < 0 { 0 as u16 } else { x as u16 };
        let y = if y < 0 { 0 as u16 } else { y as u16 };
        if x >= self.x && y >= self.y {
            let q = self.t.query();
            if x < (self.x + q.width as u16) && y < (self.y + q.height as u16) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    );
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
    b: Vec<Widget<'a>>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        let mut b = Vec::new();
        b.push(Widget::PlainColorButton(PlainColorButton::new(
            tc, 50, 50, 50, 50,
        )));
        Self { b: b }
    }
}

impl<'a> GameMode for ExplorerMenu<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
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
                MouseEventOutput::DragStop => {
                    println!("Stopped dragging");
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
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

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Login));
            println!("You clicked the button");
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
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = 330;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(241, 385, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }
        for w in &mut self.b {
            w.draw(canvas, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}


/// This is for exploring the resources of the game client
pub struct Login<'a> {
    b: Vec<Widget<'a>>,
}

impl<'a> Login<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        let mut b = Vec::new();
        b.push(Widget::PlainColorButton(PlainColorButton::new(
            tc, 50, 50, 50, 50,
        )));
        Self { b: b }
    }
}

impl<'a> GameMode for Login<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
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
                MouseEventOutput::DragStop => {
                    println!("Stopped dragging");
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
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

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Explorer));
            println!("You clicked the button");
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
        let value = 814;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        for w in &mut self.b {
            w.draw(canvas, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
