use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

/// The screen that allows for selection of which character to play
pub struct CharacterSelect<'a> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    char_sel: Vec<CharacterSelectWidget>,
    page: u8,
    selection: Option<u8>,
    //1764.img for disabled slot
}

impl<'a> CharacterSelect<'a> {
    pub fn new<T>(_tc: &'a TextureCreator<T>, _r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Box<dyn Widget<'a> + 'a>> = Vec::new();
        b.push(Box::new(ImgButton::new(0x6e5, 0x0f7, 0x10b)));
        b.push(Box::new(ImgButton::new(0x6e7, 0x16c, 0x10b)));
        b.push(Box::new(ImgButton::new(0x334, 0x20d, 0x185)));
        b.push(Box::new(ImgButton::new(0x336, 0x20d, 0x19a)));
        b.push(Box::new(ImgButton::new(0x134, 0x20d, 0x1b5)));
        let mut ch = Vec::new();

        ch.push(CharacterSelectWidget::new(0x13, 0));
        ch.push(CharacterSelectWidget::new(0xb0, 0));
        ch.push(CharacterSelectWidget::new(0x14d, 0));
        ch.push(CharacterSelectWidget::new(0x1ea, 0));
        Self {
            b: b,
            char_sel: ch,
            page: 0,
            selection: None,
        }
    }
}

impl<'a> GameMode<'a> for CharacterSelect<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        _requests: &mut VecDeque<DrawModeRequest>,
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
                    for w in &mut self.char_sel {
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
        r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        self.char_sel[0].set_type(r.characters[(0 + self.page * 4) as usize].t);
        self.char_sel[1].set_type(r.characters[(1 + self.page * 4) as usize].t);
        self.char_sel[2].set_type(r.characters[(2 + self.page * 4) as usize].t);
        self.char_sel[3].set_type(r.characters[(3 + self.page * 4) as usize].t);

        if self.b[0].was_clicked() {
            if self.page > 0 {
                self.page -= 1;
                //todo update the animation data for each char_sel widget
                self.char_sel[0].set_animating(false);
                self.char_sel[1].set_animating(false);
                self.char_sel[2].set_animating(false);
                self.char_sel[3].set_animating(false);
                self.selection = None;
            }
        }
        if self.b[1].was_clicked() {
            if self.page < 1 {
                self.page += 1;
                //todo update the animation data for each char_sel widget
                self.selection = None;
                self.char_sel[0].set_animating(false);
                self.char_sel[1].set_animating(false);
                self.char_sel[2].set_animating(false);
                self.char_sel[3].set_animating(false);
            }
        }

        if self.char_sel[0].was_clicked() {
            self.selection = Some(4 * self.page + 0);
            self.char_sel[0].set_animating(true);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[1].was_clicked() {
            self.selection = Some(4 * self.page + 1);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(true);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[2].was_clicked() {
            self.selection = Some(4 * self.page + 2);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(true);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[3].was_clicked() {
            self.selection = Some(4 * self.page + 3);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(true);
        }

        if self.b[2].was_clicked() {
            if let Some(c) = self.selection {
                match r.characters[c as usize].t {
                    CharacterDisplayType::NewCharacter => {
                        requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::NewCharacter));
                    }
                    _ => {
                        requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Game));
                    }
                }
            }
        }
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
        let value = 815;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.send(MessageToAsync::LoadPng(value));
        }

        let value = if self.page == 0 { 0x6ea } else { 0x6e9 };
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0x127, 0x10f, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.send(MessageToAsync::LoadImg(value));
        }

        let value = if self.page == 1 { 0x6ec } else { 0x6eb };
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0x146, 0x10f, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.send(MessageToAsync::LoadImg(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.char_sel {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
