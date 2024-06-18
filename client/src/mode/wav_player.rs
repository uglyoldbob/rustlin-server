use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

/// The screen that allows for user login
pub struct WavPlayer<'a, T> {
    b: Vec<Widget<'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_wav: u16,
    play_wav: bool,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> WavPlayer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Widget<'a>> = Vec::new();
        b.push(Widget::TextButton(TextButton::new(
            tc, 320, 400, "Go Back", &r.font,
        )));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            386,
            "Ready to play 1.wav",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        Self {
            b: b,
            disp: disp,
            current_wav: 1,
            tc: tc,
            play_wav: false,
        }
    }
}

impl<'a, T> GameMode<'a> for WavPlayer<'a, T> {
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
        r: &mut GameResources,
    ) {
        if down {
            match button {
                sdl2::keyboard::Keycode::Left => {
                    if self.current_wav > 1 {
                        r.sfx.remove(&self.current_wav);
                        self.current_wav -= 1;
                        let words = format!("Ready to play {}.wav", self.current_wav);
                        self.disp[0].update_text(self.tc, &words, &r.font);
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_wav < 65534 {
                        r.sfx.remove(&self.current_wav);
                        self.current_wav += 1;
                        let words = format!("Ready to play {}.wav", self.current_wav);
                        self.disp[0].update_text(self.tc, &words, &r.font);
                    }
                }
                sdl2::keyboard::Keycode::P => {
                    self.play_wav = true;
                }
                _ => {}
            }
        }
    }

    fn process_frame(&mut self, r: &mut GameResources, _requests: &mut VecDeque<DrawModeRequest>) {
        if self.play_wav {
            if let Some(c) = r.get_or_load_sfx(self.current_wav) {
                let chan = sdl2::mixer::Channel::all();
                let _e = chan.play(c, 0);
            }
            self.play_wav = false;
        }
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
