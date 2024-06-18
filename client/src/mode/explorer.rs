use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;
use std::rc::Rc;

/// This is for exploring the resources of the game client
pub struct ExplorerMenu<'a> {
    b: Vec<Widget<'a>>,
    background: Option<Rc<Texture<'a>>>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Widget<'a>> = Vec::new();
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            100,
            "Png browser",
            &r.font,
        )));
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            114,
            "Img browser",
            &r.font,
        )));
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            128,
            "Sprite browser",
            &r.font,
        )));
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            142,
            "Wav player",
            &r.font,
        )));
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            156,
            "Tile browser",
            &r.font,
        )));
        b.push(Widget::TextButton(TextButton::new(
            tc,
            50,
            170,
            "Map browser",
            &r.font,
        )));
        Self {
            b: b,
            background: r.get_or_load_png(811),
        }
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
        _m: sdl2::keyboard::Mod,
        _down: bool,
        _r: &mut GameResources<'a, '_, '_>,
    ) {
    }

    fn process_frame(
        &mut self,
        _r: &mut GameResources<'a, '_, '_>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        if let Some(t) = &self.background {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
