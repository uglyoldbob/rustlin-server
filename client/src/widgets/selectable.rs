use std::rc::Rc;

use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use sdl2::rect::Rect;
use sdl2::render::Texture;

pub struct SelectableWidget<'a> {
    num: u16,
    x: u16,
    y: u16,
    clicked: bool,
    selected: bool,
    last_draw: Option<ImageBox>,
    inactive: Option<Rc<Texture<'a>>>,
    active: Option<Rc<Texture<'a>>>,
}

impl<'a> SelectableWidget<'a> {
    pub fn new(num: u16, x: u16, y: u16, r: &mut GameResources<'a, '_, '_>) -> Self {
        Self {
            inactive: r.get_or_load_img(num),
            active: r.get_or_load_img(num + 1),
            num: num,
            x: x,
            y: y,
            clicked: false,
            selected: false,
            last_draw: None,
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, s: bool) {
        self.selected = s;
    }
}

impl<'a> Widget<'a> for SelectableWidget<'a> {
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
        _cursor: bool,
        r: &mut GameResources,
    ) {
        let value = if self.selected {
            &self.active
        } else {
            &self.inactive
        };

        self.last_draw = if let Some(t) = value {
            let q = t.query();
            let _e = canvas.copy(
                &t,
                None,
                Rect::new(
                    self.x as i32,
                    self.y as i32,
                    q.width.into(),
                    q.height.into(),
                ),
            );
            Some(ImageBox {
                x: self.x,
                y: self.y,
                w: q.width as u16,
                h: q.height as u16,
            })
        } else {
            None
        };
    }
}
