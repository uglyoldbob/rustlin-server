use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::render::TextureCreator;

pub struct MapWidget {
    clicked: bool,
    x: u16,
    y: u16,
}

impl MapWidget {
    pub fn new<T>(_tc: &TextureCreator<T>, x: u16, y: u16) -> Self {
        Self {
            clicked: false,
            x: x,
            y: y,
        }
    }
}

impl Widget for MapWidget {
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
        let current_tile = 0;
        let current_subtile = 0;
        match r.tilesets.get(&current_tile) {
            Some(ts) => match ts {
                Loaded(t) => {
                    t.draw_left(self.x, self.y, current_subtile, canvas);
                    t.draw_right(self.x, self.y, current_subtile, canvas);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
