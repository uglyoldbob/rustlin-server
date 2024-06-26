use crate::GameResources;
use crate::ImageBox;

#[enum_dispatch::enum_dispatch]
pub enum Widget<'a> {
    CharacterSelect(character_select::CharacterSelectWidget<'a>),
    DynamicText(dynamic_text::DynamicTextWidget<'a>),
    ImgButton(img_button::ImgButton<'a>),
    Map(map_widget::MapWidget<'a>),
    PlainColorButton(plain_color_button::PlainColorButton<'a>),
    Selectable(selectable::SelectableWidget<'a>),
    Sprite(sprite_widget::SpriteWidget),
    TextButton(text_button::TextButton<'a>),
    TextInput(text_input::TextInput<'a>),
}

#[enum_dispatch::enum_dispatch(Widget)]
pub trait WidgetTrait<'a> {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        let hover = if let Some(c) = cursor {
            let (x, y) = c;
            self.contains(x, y)
        } else {
            false
        };
        self.draw_hover(canvas, hover, r);
    }
    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        r: &mut GameResources<'a, '_, '_>,
    );
    fn was_clicked(&mut self) -> bool;
    fn clicked(&mut self);
    fn contains(&mut self, x: i16, y: i16) -> bool {
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

mod plain_color_button;
pub use plain_color_button::PlainColorButton;

mod text_button;
pub use text_button::TextButton;

mod img_button;
pub use img_button::ImgButton;

mod map_widget;
pub use map_widget::MapWidget;

mod selectable;
pub use selectable::SelectableWidget;

mod dynamic_text;
pub use dynamic_text::DynamicTextWidget;

mod character_select;
pub use character_select::*;

mod sprite_widget;
pub use sprite_widget::SpriteWidget;

mod text_input;
pub use text_input::*;
