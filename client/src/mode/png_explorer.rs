use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;
use std::rc::Rc;

/// The screen that allows for user login
pub struct PngExplorer<'a, T> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_png: u16,
    previous_png_object: Option<Rc<Texture<'a>>>,
    current_png_object: Option<Rc<Texture<'a>>>,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> PngExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 400, "Go Back", &r.font)));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            386,
            "Displaying 0.png",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        let cur = r.get_or_load_png(0);

        Self {
            b: b,
            disp: disp,
            current_png: 0,
            tc: tc,
            previous_png_object: None,
            current_png_object: cur,
        }
    }
}

impl<'a, T> GameMode<'a> for PngExplorer<'a, T> {
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
                    if self.current_png > 0 {
                        self.previous_png_object = r.get_or_load_png(self.current_png);
                        self.current_png -= 1;
                        self.current_png_object = r.get_or_load_png(self.current_png);
                        let words = format!("Displaying {}.png", self.current_png);
                        self.disp[0].update_text(self.tc, &words, &r.font);
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_png < 65534 {
                        self.previous_png_object = r.get_or_load_png(self.current_png);
                        self.current_png += 1;
                        self.current_png_object = r.get_or_load_png(self.current_png);
                        let words = format!("Displaying {}.png", self.current_png);
                        self.disp[0].update_text(self.tc, &words, &r.font);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_frame(&mut self, _r: &mut GameResources, _requests: &mut VecDeque<DrawModeRequest>) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        if let Some(t) = &self.current_png_object {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
        } else if let Some(t) = &self.previous_png_object {
            let q = t.query();
            let _e = canvas.copy(&t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
        }

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
