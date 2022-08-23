use std::f32::consts::E;

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
    s: String,
    last_s: String,
    color: sdl2::pixels::Color,
    last_draw: Option<ImageBox>,
    cur_cycle: u8,
    cycles: u8,
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
        let pr = font.render(&text[..]);
        let ft = pr.solid(color).unwrap();
        let mut text2 = text.clone();
        text2.push_str("|");
        let pr2 = font.render(&text2[..]);
        let ft2 = pr2.solid(color).unwrap();

        Self {
            tc: tc,
            t: Texture::from_surface(&ft, tc).unwrap(),
            t2: Texture::from_surface(&ft2, tc).unwrap(),
            x: x,
            y: y,
            s: text.to_string(),
            last_s: text.to_string(),
            color: color,
            last_draw: None,
            cur_cycle: 0,
            cycles: cycles,
        }
    }

    pub fn update_text(&mut self, font: &sdl2::ttf::Font) {
        if self.last_s != self.s {
            let pr = font.render(&self.s[..]);
            let ft = pr.solid(self.color).unwrap();

            let mut text2 = self.s.clone();
            text2.push_str("|");
            let pr2 = font.render(&text2[..]);
            let ft2 = pr2.solid(self.color).unwrap();

            self.t = Texture::from_surface(&ft, self.tc).unwrap();
            self.t2 = Texture::from_surface(&ft2, self.tc).unwrap();
            self.last_s = self.s.clone();
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
