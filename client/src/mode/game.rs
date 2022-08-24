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

/// The screen that allows for selection of which character to play
pub struct Game<'a> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    sprites: Vec<SpriteWidget>,
    framing: [Option<Rc<Texture<'a>>>; 5],
}

impl<'a> Game<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(PlainColorButton::new(tc, 50, 50, 50, 50)));
        let mut d = Vec::new();
        d.push(DynamicTextWidget::new(
            tc,
            35,
            390,
            "0",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            35,
            407,
            "1",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            35,
            426,
            "2",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));

        Self {
            b: b,
            disp: d,
            sprites: Vec::new(),
            framing: [
                r.get_or_load_img(1028),
                r.get_or_load_img(1019),
                r.get_or_load_img(1029),
                r.get_or_load_img(1030),
                r.get_or_load_img(1031),
            ],
        }
    }
}

impl<'a> GameMode<'a> for Game<'a> {
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
        _m: sdl2::keyboard::Mod,
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

        if let Some(t) = &self.framing[0] {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(0, 368, q.width.into(), q.height.into()));
        }

        if let Some(t) = &self.framing[1] {
            let q = t.query();
            let _e = canvas.copy(
                &t,
                None,
                Rect::new(485, 366, q.width.into(), q.height.into()),
            );
        }

        if let Some(t) = &self.framing[2] {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(3, 386, q.width.into(), q.height.into()));
        }

        if let Some(t) = &self.framing[3] {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(3, 402, q.width.into(), q.height.into()));
        }

        if let Some(t) = &self.framing[4] {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(3, 423, q.width.into(), q.height.into()));
        }

        for w in &mut self.disp {
            w.draw(canvas, cursor, r);
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
