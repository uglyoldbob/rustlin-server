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

/// This is for exploring the resources of the game client
pub struct NewCharacterMode<'a, T> {
    tc: &'a TextureCreator<T>,
    b: Vec<Box<dyn Widget + 'a>>,
    c: CharacterSelectWidget,
    options: Vec<SelectableWidget>,
    selected_class: u8,
    /// true is male, false is female
    selected_gender: bool,
    disp: Vec<DynamicTextWidget<'a>>,
    base_stats: CharacterStats,
    current_stats: CharacterStats,
    max_stats: CharacterStats,
}

impl<'a, T> NewCharacterMode<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget>> = Vec::new();
        b.push(Box::new(ImgButton::new(825, 476, 403)));
        b.push(Box::new(ImgButton::new(827, 476, 430)));
        b.push(Box::new(ImgButton::new(556, 424, 317)));
        b.push(Box::new(ImgButton::new(554, 435, 317)));
        b.push(Box::new(ImgButton::new(556, 424, 332)));
        b.push(Box::new(ImgButton::new(554, 435, 332)));
        b.push(Box::new(ImgButton::new(556, 424, 347)));
        b.push(Box::new(ImgButton::new(554, 435, 347)));
        b.push(Box::new(ImgButton::new(556, 498, 317)));
        b.push(Box::new(ImgButton::new(554, 509, 317)));
        b.push(Box::new(ImgButton::new(556, 498, 332)));
        b.push(Box::new(ImgButton::new(554, 509, 332)));
        b.push(Box::new(ImgButton::new(556, 498, 347)));
        b.push(Box::new(ImgButton::new(554, 509, 347)));
        let mut c = CharacterSelectWidget::new(410, 0);
        c.set_animating(true);
        let mut o = Vec::new();
        o.push(SelectableWidget::new(1753, 332, 11));
        o.push(SelectableWidget::new(1755, 542, 11));
        o.push(SelectableWidget::new(1757, 332, 67));
        o.push(SelectableWidget::new(1759, 542, 67));
        o.push(SelectableWidget::new(1761, 332, 118));
        o.push(SelectableWidget::new(1749, 542, 118));
        o.push(SelectableWidget::new(1751, 332, 166));
        o.push(SelectableWidget::new(306, 348, 248));
        o.push(SelectableWidget::new(304, 533, 248));
        o[7].set_selected(true);
        let mut d = Vec::new();
        d.push(DynamicTextWidget::new(
            tc,
            468,
            334,
            "0",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            406,
            317,
            "1",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            406,
            332,
            "2",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            406,
            347,
            "3",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            525,
            317,
            "4",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            525,
            332,
            "5",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            525,
            347,
            "6",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        let bs = c.t.get_base_stats();
        let ms = c.t.get_max_stats();
        let mut s = Self {
            tc: tc,
            b: b,
            c: c,
            options: o,
            selected_class: 0,
            selected_gender: true,
            disp: d,
            base_stats: bs,
            current_stats: bs,
            max_stats: ms,
        };
        s.update_stats(r);
        s
    }

    fn compute_remain(&self) -> u8 {
        75 - self.current_stats.str
            - self.current_stats.dex
            - self.current_stats.con
            - self.current_stats.wis
            - self.current_stats.cha
            - self.current_stats.int
    }

    fn update_stats(&mut self, r: &mut GameResources) {
        let remain = self.compute_remain();
        let word = format!("{}", remain);
        self.disp[0].update_text(self.tc, &word, &r.font);

        let word = format!("{}", self.current_stats.str);
        self.disp[1].update_text(self.tc, &word, &r.font);
        let word = format!("{}", self.current_stats.dex);
        self.disp[2].update_text(self.tc, &word, &r.font);
        let word = format!("{}", self.current_stats.con);
        self.disp[3].update_text(self.tc, &word, &r.font);
        let word = format!("{}", self.current_stats.wis);
        self.disp[4].update_text(self.tc, &word, &r.font);
        let word = format!("{}", self.current_stats.cha);
        self.disp[5].update_text(self.tc, &word, &r.font);
        let word = format!("{}", self.current_stats.int);
        self.disp[6].update_text(self.tc, &word, &r.font);
    }

    fn update_selected_char(&mut self) {
        let newtype = match self.selected_class {
            0 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleRoyal
                } else {
                    CharacterDisplayType::FemaleRoyal
                }
            }
            1 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleKnight
                } else {
                    CharacterDisplayType::FemaleKnight
                }
            }
            2 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleElf
                } else {
                    CharacterDisplayType::FemaleElf
                }
            }
            3 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleWizard
                } else {
                    CharacterDisplayType::FemaleWizard
                }
            }
            4 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleDarkElf
                } else {
                    CharacterDisplayType::FemaleDarkElf
                }
            }
            5 => {
                if self.selected_gender {
                    CharacterDisplayType::MaleDragonKnight
                } else {
                    CharacterDisplayType::FemaleDragonKnight
                }
            }
            _ => {
                if self.selected_gender {
                    CharacterDisplayType::MaleIllusionist
                } else {
                    CharacterDisplayType::FemaleIllusionist
                }
            }
        };
        self.c.set_type(newtype);
        self.base_stats = self.c.t.get_base_stats();
        self.current_stats = self.base_stats;
        self.max_stats = self.c.t.get_max_stats();
    }
}

impl<'a, T> GameMode for NewCharacterMode<'a, T> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        _requests: &mut VecDeque<DrawModeRequest>,
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
                    for w in &mut self.options {
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
    }

    fn process_button(
        &mut self,
        _button: sdl2::keyboard::Keycode,
        _down: bool,
        _r: &mut GameResources,
    ) {
    }

    fn process_frame(
        &mut self,
        r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        if self.b[1].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
        }
        if self.b[0].was_clicked() {
            let remain = self.compute_remain();
            if remain == 0 {
                requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
            }
        }

        if self.b[3].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.str < self.max_stats.str {
                self.current_stats.str += 1;
                self.update_stats(r);
            }
        }
        if self.b[2].was_clicked() {
            if self.current_stats.str > self.base_stats.str {
                self.current_stats.str -= 1;
                self.update_stats(r);
            }
        }
        if self.b[5].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.dex < self.max_stats.dex {
                self.current_stats.dex += 1;
                self.update_stats(r);
            }
        }
        if self.b[4].was_clicked() {
            if self.current_stats.dex > self.base_stats.dex {
                self.current_stats.dex -= 1;
                self.update_stats(r);
            }
        }

        if self.b[7].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.con < self.max_stats.con {
                self.current_stats.con += 1;
                self.update_stats(r);
            }
        }
        if self.b[6].was_clicked() {
            if self.current_stats.con > self.base_stats.con {
                self.current_stats.con -= 1;
                self.update_stats(r);
            }
        }
        if self.b[9].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.wis < self.max_stats.wis {
                self.current_stats.wis += 1;
                self.update_stats(r);
            }
        }
        if self.b[8].was_clicked() {
            if self.current_stats.wis > self.base_stats.wis {
                self.current_stats.wis -= 1;
                self.update_stats(r);
            }
        }
        if self.b[11].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.cha < self.max_stats.cha {
                self.current_stats.cha += 1;
                self.update_stats(r);
            }
        }
        if self.b[10].was_clicked() {
            if self.current_stats.cha > self.base_stats.cha {
                self.current_stats.cha -= 1;
                self.update_stats(r);
            }
        }

        if self.b[13].was_clicked() {
            let remain = self.compute_remain();
            if remain > 0 && self.current_stats.int < self.max_stats.int {
                self.current_stats.int += 1;
                self.update_stats(r);
            }
        }
        if self.b[12].was_clicked() {
            if self.current_stats.int > self.base_stats.int {
                self.current_stats.int -= 1;
                self.update_stats(r);
            }
        }

        for i in 0..=6 {
            if self.options[i].was_clicked() {
                self.options[0].set_selected(false);
                self.options[1].set_selected(false);
                self.options[2].set_selected(false);
                self.options[3].set_selected(false);
                self.options[4].set_selected(false);
                self.options[5].set_selected(false);
                self.options[6].set_selected(false);
                self.options[i].set_selected(true);
                self.selected_class = i as u8;
                self.update_selected_char();
                self.update_stats(r);
            }
        }
        for i in 7..=8 {
            if self.options[i].was_clicked() {
                self.options[7].set_selected(false);
                self.options[8].set_selected(false);
                self.options[i].set_selected(true);
                self.selected_gender = if i == 7 { true } else { false };
                self.update_selected_char();
                self.update_stats(r);
            }
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
        let value = 824;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.options {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.disp {
            w.draw(canvas, cursor, r, send);
        }
        self.c.draw(canvas, cursor, r, send);
    }

    fn framerate(&self) -> u8 {
        20
    }
}
