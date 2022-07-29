use crate::map::MapSegment;
use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;

pub struct MapWidget<'a> {
    clicked: bool,
    //the screen coordinates for the top left corner of the map widget
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    mapx: u16,
    mapy: u16,
    segments: [Option<MapSegment>; 4],
    buffer: Texture<'a>,
}

impl<'a> MapWidget<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, w: u16, h: u16) -> Self {
        let texture = tc
            .create_texture_target(PixelFormatEnum::RGB555, w as u32, h as u32)
            .unwrap();
        Self {
            clicked: false,
            x: x,
            y: y,
            w: w,
            h: h,
            mapx: 0,
            mapy: 0,
            segments: [Some(MapSegment::empty_segment()), None, None, None],
            buffer: texture,
        }
    }
}

impl<'a> Widget for MapWidget<'a> {
    fn last_draw(&self) -> Option<ImageBox> {
        None
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
        _cursor: bool,
        r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.with_texture_canvas(&mut self.buffer, |canvas| {
            if let Some(seg) = &self.segments[0] {
                seg.draw_floor(canvas, 0, 0, r);
            }
        });
        let q = self.buffer.query();
        canvas.copy(
            &self.buffer,
            None,
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
    }
}
