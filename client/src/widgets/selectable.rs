use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::rect::Rect;

pub struct SelectableWidget {
    num: u16,
    x: u16,
    y: u16,
    clicked: bool,
    selected: bool,
    last_draw: Option<ImageBox>,
}

impl SelectableWidget {
    pub fn new(num: u16, x: u16, y: u16) -> Self {
        Self {
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

impl Widget for SelectableWidget {
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
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let value = if self.selected {
            if let Some(i) = r.imgs.get(&(self.num + 1)) {
                if let Loaded(_) = i {
                    self.num + 1
                } else {
                    self.num
                }
            } else {
                r.imgs.insert(self.num + 1, Loading);
                let _e = send.blocking_send(MessageToAsync::LoadImg(self.num + 1));
                self.num
            }
        } else {
            self.num
        };

        self.last_draw = if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
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
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
            None
        };
    }
}
