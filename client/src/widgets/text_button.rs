use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

pub struct TextButton<'a> {
    t: Texture<'a>,
    t2: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
    last_draw: Option<ImageBox>,
}

impl<'a> TextButton<'a> {
    pub fn new<T>(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        text: &str,
        font: &sdl2::ttf::Font,
    ) -> Self {
        let pr = font.render(text);
        let ft = pr.solid(sdl2::pixels::Color::RED).unwrap();
        let pr = font.render(text);
        let ft2 = pr.solid(sdl2::pixels::Color::YELLOW).unwrap();

        Self {
            t: Texture::from_surface(&ft, tc).unwrap(),
            t2: Texture::from_surface(&ft2, tc).unwrap(),
            x: x,
            y: y,
            clicked: false,
            last_draw: None,
        }
    }
}

impl<'a> Widget<'a> for TextButton<'a> {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        _r: &mut GameResources,
    ) {
        let t = if cursor { &self.t2 } else { &self.t };
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
