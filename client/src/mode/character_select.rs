use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameModeTrait;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;
use std::rc::Rc;

/// The screen that allows for selection of which character to play
pub struct CharacterSelect<'a> {
    b: Vec<Widget<'a>>,
    char_sel: Vec<CharacterSelectWidget<'a>>,
    page: u8,
    selection: Option<u8>,
    background: Option<Rc<Texture<'a>>>,
    page1: [Option<Rc<Texture<'a>>>; 2],
    page2: [Option<Rc<Texture<'a>>>; 2],
    //1764.img for disabled slot
}

impl<'a> CharacterSelect<'a> {
    pub fn new<T>(_tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Widget<'a>> = Vec::new();
        b.push(Widget::ImgButton(ImgButton::new(0x6e5, 0x0f7, 0x10b, r)));
        b.push(Widget::ImgButton(ImgButton::new(0x6e7, 0x16c, 0x10b, r)));
        b.push(Widget::ImgButton(ImgButton::new(0x334, 0x20d, 0x185, r)));
        b.push(Widget::ImgButton(ImgButton::new(0x336, 0x20d, 0x19a, r)));
        b.push(Widget::ImgButton(ImgButton::new(0x134, 0x20d, 0x1b5, r)));
        let mut ch = Vec::new();

        ch.push(CharacterSelectWidget::new(0x13, 0, r));
        ch.push(CharacterSelectWidget::new(0xb0, 0, r));
        ch.push(CharacterSelectWidget::new(0x14d, 0, r));
        ch.push(CharacterSelectWidget::new(0x1ea, 0, r));
        Self {
            b: b,
            char_sel: ch,
            page: 0,
            selection: None,
            background: r.get_or_load_png(815),
            page1: [r.get_or_load_img(0x6ea), r.get_or_load_img(0x6e9)],
            page2: [r.get_or_load_img(0x6ec), r.get_or_load_img(0x6eb)],
        }
    }
}

impl<'a, T> GameModeTrait<'a, T> for CharacterSelect<'a> {
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
        _m: sdl2::keyboard::Mod,
        _down: bool,
        _r: &mut GameResources<'a, '_, '_>,
    ) {
    }

    fn process_frame(
        &mut self,
        r: &mut GameResources<'a, '_, '_>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        self.char_sel[0].set_type(r.characters[(0 + self.page * 4) as usize].t, r);
        self.char_sel[1].set_type(r.characters[(1 + self.page * 4) as usize].t, r);
        self.char_sel[2].set_type(r.characters[(2 + self.page * 4) as usize].t, r);
        self.char_sel[3].set_type(r.characters[(3 + self.page * 4) as usize].t, r);

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
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        if let Some(t) = &self.background {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
        }

        let value = if self.page == 0 {
            &self.page1[0]
        } else {
            &self.page1[1]
        };
        if let Some(t) = value {
            let q = t.query();
            let _e = canvas.copy(
                &t,
                None,
                Rect::new(0x127, 0x10f, q.width.into(), q.height.into()),
            );
        }

        let value = if self.page == 1 {
            &self.page2[0]
        } else {
            &self.page2[1]
        };
        if let Some(t) = value {
            let q = t.query();
            let _e = canvas.copy(
                &t,
                None,
                Rect::new(0x146, 0x10f, q.width.into(), q.height.into()),
            );
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
        for w in &mut self.char_sel {
            w.draw(canvas, cursor, r);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
