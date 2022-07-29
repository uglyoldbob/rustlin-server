use crate::mouse::MouseEventOutput;
use crate::widgets::dynamic_text::DynamicTextWidget;
use crate::widgets::map_widget::MapWidget;
use crate::widgets::text_button::TextButton;
use crate::widgets::Widget;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

pub struct TileExplorer<'a, T> {
    b: Vec<Box<dyn Widget + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_tile: u16,
    current_subtile: u16,
    tc: &'a TextureCreator<T>,
    displayed: bool,
}

impl<'a, T> TileExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 400, "Go Back", &r.font)));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            386,
            "Displaying 0.til subtile 0",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        Self {
            b: b,
            disp: disp,
            current_tile: 0,
            current_subtile: 0,
            tc: tc,
            displayed: false,
        }
    }
}

impl<'a, T> GameMode for TileExplorer<'a, T> {
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
                    if self.current_tile > 0 {
                        if true {
                            self.current_tile -= 1;
                            self.current_subtile = 0;
                            let words = format!(
                                "Displaying {}.til subtile {}",
                                self.current_tile, self.current_subtile
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_tile < 65534 {
                        if true {
                            self.current_tile += 1;
                            self.current_subtile = 0;
                            let words = format!(
                                "Displaying {}.til subtile {}",
                                self.current_tile, self.current_subtile
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Down => {
                    if self.current_subtile > 0 {
                        if true {
                            self.current_subtile -= 1;
                            let words = format!(
                                "Displaying {}.til subtile {}",
                                self.current_tile, self.current_subtile
                            );
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Up => {
                    if self.current_subtile < 65534 {
                        if true {
                            self.current_subtile += 1;
                            let words = format!(
                                "Displaying {}.til subtile {}",
                                self.current_tile, self.current_subtile
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
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
        match r.tilesets.get(&self.current_tile) {
            None => {
                r.tilesets.insert(self.current_tile, Loading);
                let _e = send.blocking_send(MessageToAsync::LoadTileset(self.current_tile));
            }
            _ => {}
        }
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.disp {
            w.draw(canvas, cursor, r, send);
        }

        match r.tilesets.get(&self.current_tile) {
            Some(ts) => match ts {
                Loaded(t) => {
                    t.draw_left(320, 240, self.current_subtile, canvas);
                    t.draw_right(320, 240, self.current_subtile, canvas);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}
