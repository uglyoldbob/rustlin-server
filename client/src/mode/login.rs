use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameModeTrait;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::collections::VecDeque;
use std::rc::Rc;

/// The screen that allows for user login
pub struct Login<'a> {
    b: Vec<Widget<'a>>,
    background: Option<Rc<Texture<'a>>>,
    detail: Option<Rc<Texture<'a>>>,
    username: TextInput<'a>,
    password: TextInput<'a>,
    focus: u8,
}

impl<'a> Login<'a> {
    pub fn new<T>(r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Widget<'a>> = Vec::new();
        b.push(Widget::ImgButton(ImgButton::new(53, 0x213, 0x183, r)));
        b.push(Widget::ImgButton(ImgButton::new(65, 0x213, 0x195, r)));
        b.push(Widget::ImgButton(ImgButton::new(55, 0x213, 0x1a8, r)));
        b.push(Widget::ImgButton(ImgButton::new(57, 0x213, 0x1c2, r)));

        let mut ti = TextInput::new(
            &r.tc,
            0x1fb,
            0x160,
            "".to_string(),
            &r.font,
            Color::WHITE,
            10,
        );
        ti.make_password();
        Self {
            b: b,
            background: r.get_or_load_png(814),
            detail: r.get_or_load_img(59),
            username: TextInput::new(
                &r.tc,
                0x1fb,
                0x14a,
                "asdf".to_string(),
                &r.font,
                Color::WHITE,
                10,
            ),
            password: ti,
            focus: 0,
        }
    }
}

impl<'a, T> GameModeTrait<'a, T> for Login<'a> {
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
        button: sdl2::keyboard::Keycode,
        m: sdl2::keyboard::Mod,
        down: bool,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        if down {
            match button {
                sdl2::keyboard::Keycode::Tab => {
                    if self.focus < 1 {
                        self.focus += 1;
                    } else {
                        self.focus = 0;
                    }
                }
                sdl2::keyboard::Keycode::Backspace
                | sdl2::keyboard::Keycode::Delete
                | sdl2::keyboard::Keycode::Left
                | sdl2::keyboard::Keycode::Right => match self.focus {
                    0 => self.username.process_button(button, m, down, r),
                    1 => self.password.process_button(button, m, down, r),
                    _ => {}
                },
                _ => match self.focus {
                    _ => {}
                },
            }
        }
    }

    fn process_text(&mut self, s: &String) {
        match self.focus {
            0 => self.username.process_text(s),
            1 => self.password.process_text(s),
            _ => {}
        }
    }

    fn process_frame(&mut self, r: &mut GameResources, _requests: &mut VecDeque<DrawModeRequest>) {
        self.username.set_focus(self.focus == 0);
        self.password.set_focus(self.focus == 1);

        self.username.update_text(&r.font);
        self.password.update_text(&r.font);
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
        self.username.draw(canvas, cursor, r);
        self.password.draw(canvas, cursor, r);
    }

    fn framerate(&self) -> u8 {
        20
    }
}
