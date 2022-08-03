use std::rc::Rc;

use crate::resources::map::MapCoordinate;
use crate::resources::map::MapSegment;
use crate::resources::map::MapSegmentGui;
use crate::resources::map::TileSetGui;
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
    segments: [Option<Box<MapSegmentGui<'a>>>; 4],
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
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
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
            segments: [
                Some(Box::new(MapSegment::empty_segment().to_gui(r, send))),
                None,
                None,
                None,
            ],
            buffer: texture,
            cursor: None,
            tile_ref: None,
        }
    }

    pub fn provide_cursor(&mut self, cursor: Option<(i16, i16)>) {
        self.cursor = cursor;
    }

    pub fn check_segments(
        &mut self,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
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

        let mut temp_map: [Option<Box<MapSegmentGui<'a>>>; 4] = [None, None, None, None];
        for (i, (ac, bc)) in required_segments.iter().enumerate() {
            let key = (*ac as u32) << 16 | (*bc as u32);
            let s1 = r.get_map(self.mapnum).get_or_load(key, || {
                let _e = send.blocking_send(MessageToAsync::LoadMapSegment(self.mapnum, *ac, *bc));
            });
            let s2: Option<Box<MapSegmentGui>> = if let Some(r) = s1 {
                let o: MapSegmentGui = (*r).clone();
                Some(Box::new(o))
            } else {
                None
            };
            temp_map[i] = s2;
        }

        self.segments = temp_map;
    }

    pub fn set_map_coord_center(&mut self, a: u16, b: u16) {
        self.map = MapCoordinate::build(a, b, self.w as u32 / 2, self.h as u32 / 2);
    }
}

impl<'a> Widget<'a> for MapWidget<'a> {
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
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        for seg in &mut self.segments {
            if let Some(segment) = seg {
                segment.check_tilesets(r, send);
            }
        }
        if let None = self.tile_ref {
            self.tile_ref = r.tilesets.get_or_load(2, || {
                let _e = send.blocking_send(MessageToAsync::LoadTileset(2));
            });
        }
        let _e = canvas.with_texture_canvas(&mut self.buffer, |canvas| {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            if let Some(seg) = &self.segments[0] {
                seg.draw_floor(canvas, &self.map, r);
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
