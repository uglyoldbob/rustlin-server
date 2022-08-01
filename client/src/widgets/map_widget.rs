use crate::map::MapSegment;
use crate::resources::map::MapCoordinate;
use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::MessageToAsync;
use sdl2::pixels::Color;
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
    map: MapCoordinate,
    mapnum: u16,
    segments: [Option<Box<MapSegment>>; 4],
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
            map: MapCoordinate::build(32768, 32768, w as u32 / 2, h as u32 / 2),
            mapnum: 0,
            segments: [
                Some(Box::new(MapSegment::empty_segment())),
                None,
                None,
                None,
            ],
            buffer: texture,
        }
    }

    pub fn check_segments(&mut self) {}

    pub fn set_map_coord_center(&mut self, a: u16, b: u16) {
        self.map = MapCoordinate::build(a, b, self.w as u32 / 2, self.h as u32 / 2);
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
        let _e = canvas.with_texture_canvas(&mut self.buffer, |canvas| {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            if let Some(seg) = &self.segments[0] {
                seg.draw_floor(canvas, &self.map, r);
            }
        });
        let q = self.buffer.query();
        let _e = canvas.copy(
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
