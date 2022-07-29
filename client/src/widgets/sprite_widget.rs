use crate::widgets::Widget;
use crate::GameResources;
use crate::ImageBox;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::render::TextureCreator;

pub struct SpriteWidget {
    clicked: bool,
    last_draw: Option<ImageBox>,
    id_major: u16,
    id_minor: u16,
    frame_index: u16,
}

impl SpriteWidget {
    pub fn new<T>(_tc: &TextureCreator<T>, _x: u16, _y: u16) -> Self {
        Self {
            clicked: false,
            last_draw: None,
            id_major: 0,
            id_minor: 0,
            frame_index: 0,
        }
    }

    pub fn set_sprite_major(&mut self, m: u16) {
        if self.id_major != m {
            self.id_major = m;
            self.frame_index = 0;
        }
    }

    pub fn set_sprite_minor(&mut self, m: u16) {
        if self.id_minor != m {
            self.id_minor = m;
            self.frame_index = 0;
        }
    }
}

impl Widget for SpriteWidget {
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
        let id = (self.id_major as u32) << 16 | self.id_minor as u32;
        if let Some(i) = r.sprites.get(&id) {
            match i {
                Loading => {
                    self.last_draw = None;
                }
                Loaded(spr) => {
                    spr.draw(320, 240, self.frame_index as usize, canvas);
                    if (self.frame_index + 1) < spr.num_frames() as u16 {
                        self.frame_index += 1;
                    } else {
                        self.frame_index = 0;
                    }
                }
            }
        } else {
            r.sprites.insert(id, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadSprite(self.id_major, self.id_minor));
            self.last_draw = None;
        }
    }
}
