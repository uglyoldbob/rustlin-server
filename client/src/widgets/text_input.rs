use std::collections::VecDeque;

use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

pub struct TextInput<'a, T> {
    tc: &'a TextureCreator<T>,
    t: Texture<'a>,
    t2: Texture<'a>,
    x: u16,
    y: u16,
    pre: VecDeque<char>,
    post: VecDeque<char>,
    s_changed: bool,
    color: sdl2::pixels::Color,
    last_draw: Option<ImageBox>,
    cur_cycle: u8,
    cycles: u8,
    keyfocus: bool,
    password: bool,
}

impl<'a, T> TextInput<'a, T> {
    pub fn new(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        text: String,
        font: &sdl2::ttf::Font,
        color: sdl2::pixels::Color,
        cycles: u8,
    ) -> Self {
        let t1 = if text.len() == 0 {
            format!(" ")
        } else {
            format!("{}", text)
        };
        let pr = font.render(&t1[..]);
        let ft = pr.solid(color).unwrap();
        let t2 = if text.len() == 0 {
            format!("|")
        } else {
            format!("{}|", text)
        };
        let pr2 = font.render(&t2[..]);
        let ft2 = pr2.solid(color).unwrap();

        Self {
            tc: tc,
            t: Texture::from_surface(&ft, tc).unwrap(),
            t2: Texture::from_surface(&ft2, tc).unwrap(),
            x: x,
            y: y,
            pre: text.to_string().chars().collect(),
            post: "".to_string().chars().collect(),
            s_changed: false,
            color: color,
            last_draw: None,
            cur_cycle: 0,
            cycles: cycles,
            keyfocus: false,
            password: false,
        }
    }

    pub fn make_password(&mut self) {
        self.password = true;
    }

    pub fn set_focus(&mut self, f: bool) {
        self.keyfocus = f;
    }

    pub fn process_button(
        &mut self,
        button: sdl2::keyboard::Keycode,
        _m: sdl2::keyboard::Mod,
        down: bool,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        if down {
            match button {
                sdl2::keyboard::Keycode::Backspace => {
                    self.pre.pop_back();
                    self.s_changed = true;
                }
                sdl2::keyboard::Keycode::Delete => {
                    self.post.pop_front();
                    self.s_changed = true;
                }
                sdl2::keyboard::Keycode::Left => {
                    if let Some(s) = self.pre.pop_back() {
                        self.post.push_front(s);
                        self.s_changed = true;
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if let Some(s) = self.post.pop_front() {
                        self.pre.push_back(s);
                        self.s_changed = true;
                    }
                }
                _ => {}
            }
        }
        println!("Processing a button input for text input");
    }

    pub fn process_text(&mut self, t: &String) {
        println!("Processing {} for text input", t);
        for i in t.chars() {
            self.pre.push_back(i);
        }
        self.s_changed = true;
    }

    pub fn update_text(&mut self, font: &sdl2::ttf::Font) {
        if self.s_changed {
            self.s_changed = false;
            let t1 = if (self.pre.len() + self.post.len()) == 0 {
                format!(" ")
            } else {
                if self.password {
                    let p1: String = self.pre.iter().map(|_x| "*").collect();
                    let p2: String = self.post.iter().map(|_x| "*").collect();
                    format!("{}{}", p1, p2)
                } else {
                    let p1: String = self.pre.iter().collect();
                    let p2: String = self.post.iter().collect();
                    format!("{}{}", p1, p2)
                }
            };
            let pr = font.render(&t1[..]);
            let ft = pr.solid(self.color).unwrap();
            let t2 = if (self.pre.len() + self.post.len()) == 0 {
                format!("|")
            } else {
                if self.password {
                    let p1: String = self.pre.iter().map(|_x| "*").collect();
                    let p2: String = self.post.iter().map(|_x| "*").collect();
                    format!("{}|{}", p1, p2)
                } else {
                    let p1: String = self.pre.iter().collect();
                    let p2: String = self.post.iter().collect();
                    format!("{}|{}", p1, p2)
                }
            };
            let pr2 = font.render(&t2[..]);
            let ft2 = pr2.solid(self.color).unwrap();

            self.t = Texture::from_surface(&ft, self.tc).unwrap();
            self.t2 = Texture::from_surface(&ft2, self.tc).unwrap();
        }
    }
}

impl<'a, T> Widget<'a> for TextInput<'a, T> {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        false
    }

    fn clicked(&mut self) {}

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        _r: &mut GameResources,
    ) {
        if !self.keyfocus {
            self.cur_cycle = 0;
        }
        let t = if self.cur_cycle < self.cycles {
            &self.t
        } else {
            &self.t2
        };
        if self.cur_cycle < (2 * self.cycles) {
            self.cur_cycle += 1;
        } else {
            self.cur_cycle = 0;
        }
        let q = t.query();
        let _e = canvas.copy(
            &t,
            None,
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
        self.last_draw = Some(ImageBox {
            x: self.x,
            y: self.y,
            w: q.width as u16,
            h: q.height as u16,
        });
    }
}
