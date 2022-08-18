use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::MessageToAsync;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

pub struct DynamicTextWidget<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    s: String,
    color: sdl2::pixels::Color,
    last_draw: Option<ImageBox>,
}

impl<'a> DynamicTextWidget<'a> {
    pub fn new<T>(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        text: &str,
        font: &sdl2::ttf::Font,
        color: sdl2::pixels::Color,
    ) -> Self {
        let pr = font.render(text);
        let ft = pr.solid(color).unwrap();

        Self {
            t: Texture::from_surface(&ft, tc).unwrap(),
            x: x,
            y: y,
            s: text.to_string(),
            color: color,
            last_draw: None,
        }
    }

    pub fn update_text<T>(
        &mut self,
        tc: &'a TextureCreator<T>,
        text: &str,
        font: &sdl2::ttf::Font,
    ) {
        if text != self.s {
            let pr = font.render(text);
            let ft = pr.solid(self.color).unwrap();
            self.t = Texture::from_surface(&ft, tc).unwrap();
            self.s = text.to_string();
        }
    }
}

impl<'a> Widget<'a> for DynamicTextWidget<'a> {
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
        _send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
    ) {
        let t = &self.t;
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
