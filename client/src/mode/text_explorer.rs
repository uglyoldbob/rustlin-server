use crate::mouse::MouseEventOutput;
use crate::resources::stringtable::StringTable;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameModeTrait;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

pub struct Explorer<'a, T> {
    b: Vec<Widget<'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_string: u32,
    tc: &'a TextureCreator<T>,
    descs: StringTable,
}

impl<'a, T> Explorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Widget<'a>> = Vec::new();
        b.push(Widget::TextButton(TextButton::new(
            tc, 32, 414, "Go Back", &r.font,
        )));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            32,
            386,
            "Displaying string 0",
            &r.font,
            sdl2::pixels::Color::RED,
        ));
        disp.push(DynamicTextWidget::new(
            tc,
            32,
            400,
            "dummy text",
            &r.font,
            sdl2::pixels::Color::RED,
        ));
        let mut s = Self {
            b: b,
            disp: disp,
            current_string: 0,
            tc: tc,
            descs: r.get_descs().unwrap(),
        };
        s.update_text(r);
        s
    }

    /// Update dynamic text for the game mode
    fn update_text(&mut self, r: &mut GameResources<'a, '_, '_>) {
        let words = format!("Displaying {} text string", self.current_string);
        self.disp[0].update_text(self.tc, &words, &r.font);
        if let Some(t) = self.descs.get(self.current_string as usize) {
            if t.is_empty() {
                self.disp[1].update_text(self.tc, "<empty string>", &r.font);
            } else {
                self.disp[1].update_text(self.tc, t, &r.font);
            }
        } else {
            self.disp[1].update_text(self.tc, "<Missing>", &r.font);
        }
    }
}

impl<'a, T> GameModeTrait<'a, T> for Explorer<'a, T> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        for e in events {
            match e {
                MouseEventOutput::Move((_x, _y)) => {}
                MouseEventOutput::LeftDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::MiddleDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::RightDrag { from: _, to } => {
                    let (_x, _y) = to;
                }
                MouseEventOutput::DragStop => {}
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.clicked();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((_x, _y)) => {}
                MouseEventOutput::RightClick((_x, _y)) => {}
                MouseEventOutput::ExtraClick => {}
                MouseEventOutput::Extra2Click => {}
                MouseEventOutput::Scrolling(_amount) => {}
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Explorer));
        }
    }

    fn process_button(
        &mut self,
        button: sdl2::keyboard::Keycode,
        _m: sdl2::keyboard::Mod,
        down: bool,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        if down {
            match button {
                sdl2::keyboard::Keycode::Left => {
                    if self.current_string > 0 {
                        if true {
                            self.current_string -= 1;
                            self.update_text(r);
                        }
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_string < 65534 {
                        if true {
                            self.current_string += 1;
                            self.update_text(r);
                        }
                    }
                }
                sdl2::keyboard::Keycode::Down => {
                    if self.current_string > 100 {
                        if true {
                            self.current_string -= 100;
                            self.update_text(r);
                        }
                    }
                }
                sdl2::keyboard::Keycode::Up => {
                    if self.current_string < 65434 {
                        if true {
                            self.current_string += 100;
                            self.update_text(r);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn process_frame(
        &mut self,
        _r: &mut GameResources<'a, '_, '_>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
        for w in &mut self.disp {
            w.draw(canvas, cursor, r);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
