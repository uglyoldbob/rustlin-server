use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::collections::VecDeque;
use std::rc::Rc;

/// The screen that allows for user login
pub struct Login<'a> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    background: Option<Rc<Texture<'a>>>,
    detail: Option<Rc<Texture<'a>>>,
}

impl<'a> Login<'a> {
    pub fn new(r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(ImgButton::new(53, 0x213, 0x183, r)));
        b.push(Box::new(ImgButton::new(65, 0x213, 0x195, r)));
        b.push(Box::new(ImgButton::new(55, 0x213, 0x1a8, r)));
        b.push(Box::new(ImgButton::new(57, 0x213, 0x1c2, r)));
        Self {
            b: b,
            background: r.get_or_load_png(814),
            detail: r.get_or_load_img(59),
        }
    }
}

impl<'a> GameMode<'a> for Login<'a> {
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
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
        }
    }

    fn process_button(
        &mut self,
        _button: sdl2::keyboard::Keycode,
        _down: bool,
        _r: &mut GameResources,
    ) {
    }

    fn process_frame(&mut self, _r: &mut GameResources, _requests: &mut VecDeque<DrawModeRequest>) {
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
            let _e = canvas.copy(&t, None, None);
        }

        if let Some(t) = &self.detail {
            let q = t.query();
            let _e = canvas.copy(
                &t,
                None,
                Rect::new(0x1a9, 0x138, q.width.into(), q.height.into()),
            );
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
