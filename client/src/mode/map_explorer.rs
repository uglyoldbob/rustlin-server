use crate::mouse::MouseEventOutput;
use crate::widgets::Widget;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;
pub struct MapExplorer<'a, T> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    map: MapWidget<'a>,
    current_map: u16,
    current_x: u16,
    current_y: u16,
    tc: &'a TextureCreator<T>,
    displayed: bool,
}

impl<'a, T> MapExplorer<'a, T> {
    pub fn new(
        tc: &'a TextureCreator<T>,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Self {
        let mut b: Vec<Box<dyn Widget<'a> + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 420, "Go Back", &r.font)));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            406,
            "Displaying map 0, coordinate 32768, 32768",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        Self {
            b: b,
            disp: disp,
            current_map: 4,
            current_x: 32768,
            current_y: 32768,
            tc: tc,
            displayed: false,
            map: MapWidget::new(tc, 0, 0, 640, 400, r, send),
        }
    }
}

impl<'a, T> GameMode<'a> for MapExplorer<'a, T> {
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
        down: bool,
        r: &mut GameResources,
    ) {
        if down {
            match button {
                sdl2::keyboard::Keycode::Left => {
                    if self.current_map > 0 {
                        if true {
                            self.current_map -= 1;
                            self.current_x = 32768;
                            self.current_y = 32768;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_map < 65534 {
                        if true {
                            self.current_map += 1;
                            self.current_x = 32768;
                            self.current_y = 32768;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::A => {
                    if self.current_x > 0 {
                        if true {
                            self.current_x -= 1;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::D => {
                    if self.current_x < 65534 {
                        if true {
                            self.current_x += 1;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::S => {
                    if self.current_y < 65534 {
                        if true {
                            self.current_y += 1;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::W => {
                    if self.current_y > 0 {
                        if true {
                            self.current_y -= 1;
                            let words = format!(
                                "Displaying map {}, coordinate {}, {}",
                                self.current_map, self.current_x, self.current_y
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn process_frame(
        &mut self,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
        self.map
            .set_map_coord_center(self.current_x, self.current_y);
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        self.map.check_segments(r, send);
        self.map.provide_cursor(cursor);
        self.map.draw(canvas, cursor, r, send);
        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.disp {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
