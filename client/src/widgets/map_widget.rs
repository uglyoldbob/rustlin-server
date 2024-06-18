use std::rc::Rc;

use crate::resources::map::MapCoordinate;
use crate::resources::map::MapSegmentGui;
use crate::resources::map::TileSetGui;
use crate::widgets::WidgetTrait;
use crate::GameResources;
use crate::ImageBox;
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
    segments: [Option<Rc<Box<MapSegmentGui<'a>>>>; 4],
    buffer: Texture<'a>,
    cursor: Option<(i16, i16)>,
    tile_ref: Option<Rc<TileSetGui<'a>>>,
}

impl<'a> MapWidget<'a> {
    pub fn new<T>(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        _r: &mut GameResources<'a, '_, '_>,
    ) -> Self {
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
            mapnum: 4,
            segments: [None, None, None, None],
            buffer: texture,
            cursor: None,
            tile_ref: None,
        }
    }

    pub fn provide_cursor(&mut self, cursor: Option<(i16, i16)>) {
        self.cursor = cursor;
    }

    pub fn check_segments(&mut self, r: &mut GameResources<'a, '_, '_>) {
        for seg in &mut self.segments {
            if let Some(segment) = seg {
                if segment.get_mapnum() != self.mapnum {
                    *seg = None;
                }
            }
        }
        let screen = self.map.to_screen();
        let (a1, b1) = screen.map_coordinates(0, 0);
        let (a2, b2) = screen.map_coordinates(self.w as i16, 0);
        let (a3, b3) = screen.map_coordinates(0, self.h as i16);
        let (a4, b4) = screen.map_coordinates(self.w as i16, self.h as i16);

        let a1 = a1 & 0xFFC0;
        let a2 = a2 & 0xFFC0;
        let a3 = a3 & 0xFFC0;
        let a4 = a4 & 0xFFC0;
        let b1 = b1 & 0xFFC0;
        let b2 = b2 & 0xFFC0;
        let b3 = b3 & 0xFFC0;
        let b4 = b4 & 0xFFC0;

        let a = [a1, a2, a3, a4];
        let b = [b1, b2, b3, b4];
        let amin = a.iter().min().unwrap();
        let bmin = b.iter().min().unwrap();

        let required_segments = [
            (*amin, *bmin),
            (*amin, *bmin + 64),
            (*amin + 64, *bmin),
            (*amin + 64, *bmin + 64),
        ];

        let mut temp_map: [Option<Rc<Box<MapSegmentGui<'a>>>>; 4] = [None, None, None, None];
        for (i, (ac, bc)) in required_segments.iter().enumerate() {
            let s1: Option<Rc<Box<MapSegmentGui<'a>>>> = r.get_map_segment(self.mapnum, *ac, *bc);
            temp_map[i] = s1;
        }

        self.segments = temp_map;
    }

    pub fn set_map(&mut self, m: u16) {
        self.mapnum = m;
    }

    pub fn set_map_coord_center(&mut self, a: u16, b: u16) {
        self.map = MapCoordinate::build(a, b, self.w as u32 / 2, self.h as u32 / 2);
    }
}

impl<'a> WidgetTrait<'a> for MapWidget<'a> {
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
        r: &mut GameResources<'a, '_, '_>,
    ) {
        if let None = self.tile_ref {
            self.tile_ref = r.tilesets.get_or_load(2, || {});
        }
        let _e = canvas.with_texture_canvas(&mut self.buffer, |canvas| {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            for maybesegment in self.segments.iter() {
                if let Some(seg) = maybesegment {
                    seg.draw_floor(canvas, &self.map, r);
                }
            }
            for layer in 0..=255 {
                for maybesegment in self.segments.iter() {
                    if let Some(seg) = maybesegment {
                        seg.draw_objects(canvas, &self.map, r, layer);
                    }
                }
            }
            for maybesegment in self.segments.iter() {
                if let Some(seg) = maybesegment {
                    seg.draw_attr(canvas, &self.map, r, 2);
                }
            }
            if let Some((x, y)) = self.cursor {
                let screen = self.map.to_screen();
                let (a, b) = screen.map_coordinates(x, y);
                let screen2 = self.map.screen(a, b);
                if let Some(t) = &self.tile_ref {
                    t.draw_left(screen2.x, screen2.y, 89, canvas);
                    t.draw_right(screen2.x, screen2.y, 89, canvas);
                }
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
