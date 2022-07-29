use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::render::TextureCreator;

pub struct MapWidget {
    clicked: bool,
}

impl MapWidget {
    pub fn new<T>(_tc: &TextureCreator<T>, _x: u16, _y: u16) -> Self {
        Self { clicked: false }
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
        _canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
    }
}
