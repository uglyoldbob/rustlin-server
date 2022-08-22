use crate::mouse::MouseEventOutput;
use crate::widgets::*;
use crate::DrawMode;
use crate::DrawModeRequest;
use crate::GameMode;
use crate::GameResources;
use sdl2::pixels::Color;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

pub struct SprExplorer<'a, T> {
    b: Vec<Box<dyn Widget<'a> + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_spr_a: u16,
    current_spr_b: u16,
    sprite: SpriteWidget,
    tc: &'a TextureCreator<T>,
    displayed: bool,
}

impl<'a, T> SprExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 400, "Go Back", &r.font)));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            386,
            "Displaying 0-0.spr",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        let mut spr = SpriteWidget::new(tc, 320, 240);
        let initial_id = 0;
        spr.set_sprite_major(initial_id);
        Self {
            b: b,
            sprite: spr,
            disp: disp,
            current_spr_a: initial_id,
            current_spr_b: 0,
            tc: tc,
            displayed: false,
        }
    }
}

impl<'a, T> GameMode<'a> for SprExplorer<'a, T> {
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
                    if self.current_spr_a > 0 {
                        if true {
                            self.current_spr_a -= 1;
                            let words = format!(
                                "Displaying {}-{}.spr",
                                self.current_spr_a, self.current_spr_b
                            );
                            self.sprite.set_sprite_major(self.current_spr_a);
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_spr_a < 65534 {
                        if true {
                            self.current_spr_a += 1;
                            let words = format!(
                                "Displaying {}-{}.spr",
                                self.current_spr_a, self.current_spr_b
                            );
                            self.sprite.set_sprite_major(self.current_spr_a);
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Down => {
                    if self.current_spr_b > 0 {
                        if true {
                            self.current_spr_b -= 1;
                            let words = format!(
                                "Displaying {}-{}.spr",
                                self.current_spr_a, self.current_spr_b
                            );
                            self.sprite.set_sprite_minor(self.current_spr_b);
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Up => {
                    if self.current_spr_b < 65534 {
                        if true {
                            self.current_spr_b += 1;
                            let words = format!(
                                "Displaying {}-{}.spr",
                                self.current_spr_a, self.current_spr_b
                            );
                            self.sprite.set_sprite_minor(self.current_spr_b);
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::P => {
                    println!(
                        "The current animation has {} frames",
                        self.sprite.num_frames(r)
                    );
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

        for w in &mut self.b {
            w.draw(canvas, cursor, r);
        }
        for w in &mut self.disp {
            w.draw(canvas, cursor, r);
        }
        self.sprite.draw(canvas, cursor, r);
    }

    fn framerate(&self) -> u8 {
        20
    }
}
