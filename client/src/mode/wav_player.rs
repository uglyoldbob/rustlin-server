use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

/// The screen that allows for user login
pub struct WavPlayer<'a, T> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_wav: u16,
    play_wav: bool,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> WavPlayer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 400, "Go Back", &r.font)));
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

    fn check_sfx(
        &mut self,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
    ) {
        if r.sfx.contains_key(&self.current_wav) {
        } else {
            r.sfx.insert(self.current_wav, Loading);
            let _e = send.send(MessageToAsync::LoadSfx(self.current_wav));
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

    fn process_frame(
        &mut self,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
        self.check_sfx(r, send);
        if self.play_wav {
            if r.sfx.contains_key(&self.current_wav) {
                match &r.sfx[&self.current_wav] {
                    Loading => {}
                    Loaded(s) => {
                        let chan = sdl2::mixer::Channel::all();
                        let _e = chan.play(s, 0);
                    }
                }
            }
            self.play_wav = false;
        }
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
        send: &mut tokio::sync::mpsc::UnboundedSender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

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
