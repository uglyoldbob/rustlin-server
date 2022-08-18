use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

/// This is for exploring the resources of the game client
pub struct ExplorerMenu<'a> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget>> = Vec::new();
        b.push(Box::new(TextButton::new(
            tc,
            50,
            100,
            "Png browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            114,
            "Img browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            128,
            "Sprite browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            142,
            "Wav player",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            156,
            "Tile browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            170,
            "Map browser",
            &r.font,
        )));
        Self { b: b }
    }
}

impl<'a> GameMode<'a> for ExplorerMenu<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        for e in events {
            match e {
                MouseEventOutput::Move((_x, _y)) => {}
                MouseEventOutput::LeftDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::MiddleDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::RightDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::DragStop => {}
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.clicked();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((_x, _y)) => {}
                MouseEventOutput::RightClick((_x, _y)) => {}
                MouseEventOutput::ExtraClick => {}
                MouseEventOutput::Extra2Click => {}
                MouseEventOutput::Scrolling(_amount) => {}
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::PngExplorer));
        }
        if self.b[1].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::ImgExplorer));
        }
        if self.b[2].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::SprExplorer));
        }
        if self.b[3].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::WavPlayer));
        }
        if self.b[4].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::TileExplorer));
        }
        if self.b[5].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::MapExplorer));
        }
    }

    fn process_button(
        &mut self,
        _button: sdl2::keyboard::Keycode,
        _down: bool,
        _r: &mut GameResources,
    ) {
    }

    fn process_frame(
        &mut self,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
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
            let _e = send.send(MessageToAsync::LoadPng(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
