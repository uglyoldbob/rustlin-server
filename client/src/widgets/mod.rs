use crate::GameResources;
use crate::ImageBox;
use crate::MessageToAsync;

pub trait Widget {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let hover = if let Some(c) = cursor {
            let (x, y) = c;
            self.contains(x, y)
        } else {
            false
        };
        self.draw_hover(canvas, hover, r, send);
    }
    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    );
    fn was_clicked(&mut self) -> bool;
    fn clicked(&mut self);
    fn contains(&self, x: i16, y: i16) -> bool {
        if let Some(t) = &self.last_draw() {
            let x = if x < 0 { 0 as u16 } else { x as u16 };
            let y = if y < 0 { 0 as u16 } else { y as u16 };
            if x >= t.x && y >= t.y {
                if x < (t.x + t.w) && y < (t.y + t.h) {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    fn last_draw(&self) -> Option<ImageBox>;
}

pub mod plain_color_button;
use plain_color_button::PlainColorButton;

pub mod text_button;
use text_button::TextButton;

pub mod img_button;
use img_button::ImgButton;

pub mod map_widget;
use map_widget::MapWidget;

pub mod selectable;
use selectable::SelectableWidget;

pub mod dynamic_text;
use dynamic_text::DynamicTextWidget;

pub mod character_select;
use character_select::*;

pub mod sprite_widget;
use sprite_widget::SpriteWidget;
