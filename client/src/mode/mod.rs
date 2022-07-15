use crate::mouse::MouseEventOutput;
use crate::GameResources;
use crate::Loadable::*;
use crate::MessageToAsync;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;

pub enum DrawMode {
    Explorer,
    PngExplorer,
    ImgExplorer,
    SprExplorer,
    GameLoader,
    Login,
    CharacterSelect,
    NewCharacter,
    Game,
}

#[derive(Clone, Copy)]
pub struct ImageBox {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

/// The kind of request that can be issued by a draw mode
pub enum DrawModeRequest {
    ChangeDrawMode(DrawMode),
}

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

pub struct PlainColorButton<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
    last_draw: Option<ImageBox>,
}

impl<'a> PlainColorButton<'a> {
    fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, w: u16, h: u16) -> Self {
        let mut data: Vec<u8> = vec![0xff; (w * h * 2) as usize];
        data[2] = 0xee;
        data[3] = 0xee;
        let surf = sdl2::surface::Surface::from_data(
            data.as_mut_slice(),
            w as u32,
            h as u32,
            (2 * w) as u32,
            PixelFormatEnum::RGB555,
        )
        .unwrap();
        Self {
            t: Texture::from_surface(&surf, tc).unwrap(),
            x: x,
            y: y,
            clicked: false,
            last_draw: None,
        }
    }
}

impl<'a> Widget for PlainColorButton<'a> {
    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let q = self.t.query();
        let _e = canvas.copy(
            &self.t,
            None,
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
        self.last_draw = Some(ImageBox {
            x: self.x,
            y: self.y,
            w: q.width as u16,
            h: q.height as u16,
        });
    }
}

pub struct TextButton<'a> {
    t: Texture<'a>,
    t2: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
    last_draw: Option<ImageBox>,
}

impl<'a> TextButton<'a> {
    fn new<T>(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        text: &str,
        font: &sdl2::ttf::Font,
    ) -> Self {
        let pr = font.render(text);
        let ft = pr.solid(sdl2::pixels::Color::RED).unwrap();
        let pr = font.render(text);
        let ft2 = pr.solid(sdl2::pixels::Color::YELLOW).unwrap();

        Self {
            t: Texture::from_surface(&ft, tc).unwrap(),
            t2: Texture::from_surface(&ft2, tc).unwrap(),
            x: x,
            y: y,
            clicked: false,
            last_draw: None,
        }
    }
}

impl<'a> Widget for TextButton<'a> {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let t = if cursor { &self.t2 } else { &self.t };
        let q = t.query();
        let _e = canvas.copy(
            &t,
            None,
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
        self.last_draw = Some(ImageBox {
            x: self.x,
            y: self.y,
            w: q.width as u16,
            h: q.height as u16,
        });
    }
}

pub struct ImgButton {
    num: u16,
    x: u16,
    y: u16,
    clicked: bool,
    last_draw: Option<ImageBox>,
}

impl ImgButton {
    fn new(num: u16, x: u16, y: u16) -> Self {
        Self {
            num: num,
            x: x,
            y: y,
            clicked: false,
            last_draw: None,
        }
    }
}

impl Widget for ImgButton {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let value = if cursor {
            if let Some(i) = r.imgs.get(&(self.num + 1)) {
                if let Loaded(_) = i {
                    self.num + 1
                } else {
                    self.num
                }
            } else {
                r.imgs.insert(self.num + 1, Loading);
                let _e = send.blocking_send(MessageToAsync::LoadImg(self.num + 1));
                self.num
            }
        } else {
            self.num
        };

        self.last_draw = if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(
                        self.x as i32,
                        self.y as i32,
                        q.width.into(),
                        q.height.into(),
                    ),
                );
                Some(ImageBox {
                    x: self.x,
                    y: self.y,
                    w: q.width as u16,
                    h: q.height as u16,
                })
            } else {
                None
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
            None
        };
    }
}

pub struct SelectableWidget {
    num: u16,
    x: u16,
    y: u16,
    clicked: bool,
    selected: bool,
    last_draw: Option<ImageBox>,
}

impl SelectableWidget {
    fn new(num: u16, x: u16, y: u16) -> Self {
        Self {
            num: num,
            x: x,
            y: y,
            clicked: false,
            selected: false,
            last_draw: None,
        }
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, s: bool) {
        self.selected = s;
    }
}

impl Widget for SelectableWidget {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let value = if self.selected {
            if let Some(i) = r.imgs.get(&(self.num + 1)) {
                if let Loaded(_) = i {
                    self.num + 1
                } else {
                    self.num
                }
            } else {
                r.imgs.insert(self.num + 1, Loading);
                let _e = send.blocking_send(MessageToAsync::LoadImg(self.num + 1));
                self.num
            }
        } else {
            self.num
        };

        self.last_draw = if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(
                        self.x as i32,
                        self.y as i32,
                        q.width.into(),
                        q.height.into(),
                    ),
                );
                Some(ImageBox {
                    x: self.x,
                    y: self.y,
                    w: q.width as u16,
                    h: q.height as u16,
                })
            } else {
                None
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
            None
        };
    }
}

pub struct DynamicTextWidget<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    s: String,
    color: sdl2::pixels::Color,
    last_draw: Option<ImageBox>,
}

impl<'a> DynamicTextWidget<'a> {
    fn new<T>(
        tc: &'a TextureCreator<T>,
        x: u16,
        y: u16,
        text: &str,
        font: &sdl2::ttf::Font,
        color: sdl2::pixels::Color,
    ) -> Self {
        let pr = font.render(text);
        let ft = pr.solid(color).unwrap();

        Self {
            t: Texture::from_surface(&ft, tc).unwrap(),
            x: x,
            y: y,
            s: text.to_string(),
            color: color,
            last_draw: None,
        }
    }

    fn update_text<T>(&mut self, tc: &'a TextureCreator<T>, text: &str, font: &sdl2::ttf::Font) {
        if text != self.s {
            let pr = font.render(text);
            let ft = pr.solid(self.color).unwrap();
            self.t = Texture::from_surface(&ft, tc).unwrap();
            self.s = text.to_string();
        }
    }
}

impl<'a> Widget for DynamicTextWidget<'a> {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        false
    }

    fn clicked(&mut self) {}

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let t = &self.t;
        let q = t.query();
        let _e = canvas.copy(
            &t,
            None,
            Rect::new(
                self.x.into(),
                self.y.into(),
                q.width.into(),
                q.height.into(),
            ),
        );
        self.last_draw = Some(ImageBox {
            x: self.x,
            y: self.y,
            w: q.width as u16,
            h: q.height as u16,
        });
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CharacterDisplayType {
    Blank,
    Locked,
    NewCharacter,
    MaleRoyal,
    FemaleRoyal,
    MaleKnight,
    FemaleKnight,
    MaleElf,
    FemaleElf,
    MaleWizard,
    FemaleWizard,
    MaleDarkElf,
    FemaleDarkElf,
    MaleDragonKnight,
    FemaleDragonKnight,
    MaleIllusionist,
    FemaleIllusionist,
}

#[derive(Clone, Copy)]
struct CharacterStats {
    pub str: u8,
    pub dex: u8,
    pub con: u8,
    pub wis: u8,
    pub cha: u8,
    pub int: u8,
}

impl CharacterDisplayType {
    fn get_base_stats(&self) -> CharacterStats {
        match self {
            CharacterDisplayType::MaleRoyal | CharacterDisplayType::FemaleRoyal => CharacterStats {
                str: 13,
                dex: 10,
                con: 10,
                wis: 11,
                cha: 13,
                int: 10,
            },
            CharacterDisplayType::MaleKnight | CharacterDisplayType::FemaleKnight => {
                CharacterStats {
                    str: 16,
                    dex: 12,
                    con: 14,
                    wis: 9,
                    cha: 12,
                    int: 8,
                }
            }
            CharacterDisplayType::MaleElf | CharacterDisplayType::FemaleElf => CharacterStats {
                str: 11,
                dex: 12,
                con: 12,
                wis: 12,
                cha: 9,
                int: 12,
            },
            CharacterDisplayType::MaleWizard | CharacterDisplayType::FemaleWizard => {
                CharacterStats {
                    str: 8,
                    dex: 7,
                    con: 12,
                    wis: 12,
                    cha: 8,
                    int: 12,
                }
            }
            CharacterDisplayType::MaleDarkElf | CharacterDisplayType::FemaleDarkElf => {
                CharacterStats {
                    str: 12,
                    dex: 15,
                    con: 8,
                    wis: 10,
                    cha: 9,
                    int: 11,
                }
            }
            CharacterDisplayType::MaleDragonKnight | CharacterDisplayType::FemaleDragonKnight => {
                CharacterStats {
                    str: 13,
                    dex: 11,
                    con: 14,
                    wis: 12,
                    cha: 8,
                    int: 11,
                }
            }
            CharacterDisplayType::MaleIllusionist | CharacterDisplayType::FemaleIllusionist => {
                CharacterStats {
                    str: 11,
                    dex: 10,
                    con: 12,
                    wis: 12,
                    cha: 8,
                    int: 12,
                }
            }
            _ => CharacterStats {
                str: 0,
                dex: 0,
                con: 0,
                wis: 0,
                cha: 0,
                int: 0,
            },
        }
    }

    fn get_max_stats(&self) -> CharacterStats {
        match self {
            CharacterDisplayType::MaleRoyal | CharacterDisplayType::FemaleRoyal => CharacterStats {
                str: 18,
                dex: 18,
                con: 18,
                wis: 18,
                cha: 18,
                int: 18,
            },
            CharacterDisplayType::MaleKnight | CharacterDisplayType::FemaleKnight => {
                CharacterStats {
                    str: 20,
                    dex: 18,
                    con: 18,
                    wis: 18,
                    cha: 18,
                    int: 18,
                }
            }
            CharacterDisplayType::MaleElf | CharacterDisplayType::FemaleElf => CharacterStats {
                str: 18,
                dex: 18,
                con: 18,
                wis: 18,
                cha: 18,
                int: 18,
            },
            CharacterDisplayType::MaleWizard | CharacterDisplayType::FemaleWizard => {
                CharacterStats {
                    str: 18,
                    dex: 18,
                    con: 18,
                    wis: 18,
                    cha: 18,
                    int: 18,
                }
            }
            CharacterDisplayType::MaleDarkElf | CharacterDisplayType::FemaleDarkElf => {
                CharacterStats {
                    str: 18,
                    dex: 18,
                    con: 18,
                    wis: 18,
                    cha: 18,
                    int: 18,
                }
            }
            CharacterDisplayType::MaleDragonKnight | CharacterDisplayType::FemaleDragonKnight => {
                CharacterStats {
                    str: 18,
                    dex: 18,
                    con: 18,
                    wis: 18,
                    cha: 18,
                    int: 18,
                }
            }
            CharacterDisplayType::MaleIllusionist | CharacterDisplayType::FemaleIllusionist => {
                CharacterStats {
                    str: 18,
                    dex: 18,
                    con: 18,
                    wis: 18,
                    cha: 18,
                    int: 18,
                }
            }
            _ => CharacterStats {
                str: 18,
                dex: 18,
                con: 18,
                wis: 18,
                cha: 18,
                int: 18,
            },
        }
    }
}

pub struct CharacterSelectWidget {
    plain: u16,
    hover: u16,
    last_png: u16,
    t: CharacterDisplayType,
    animate_start: u16,
    animate_quantity: u16,
    animate_index: u16,
    animating: bool,
    drawn: bool,
    x: u16,
    y: u16,
    clicked: bool,
    no_draw: bool,
    locked: bool,
    last_draw: Option<ImageBox>,
}

impl CharacterSelectWidget {
    fn new(x: u16, y: u16) -> Self {
        Self {
            plain: 0,
            hover: 1,
            last_png: 0,
            t: CharacterDisplayType::Blank,
            animating: false,
            drawn: false,
            animate_start: 1,
            animate_quantity: 24,
            animate_index: 0,
            x: x,
            y: y,
            clicked: false,
            last_draw: None,
            no_draw: true,
            locked: false,
        }
    }
}

impl CharacterSelectWidget {
    fn set_animating(&mut self, a: bool) {
        if a {
            if !self.animating {
                self.animate_index = 0;
                self.animating = true;
                self.drawn = false;
            }
        } else {
            self.animating = false;
            self.drawn = false;
        }
    }

    fn set_type(&mut self, t: CharacterDisplayType) {
        if self.t != t {
            self.drawn = false;
            self.t = t;
            self.animate_index = 0;
            match t {
                CharacterDisplayType::Blank => {
                    self.no_draw = true;
                    self.locked = false;
                }
                CharacterDisplayType::Locked => {
                    self.no_draw = false;
                    self.locked = true;
                }
                CharacterDisplayType::NewCharacter => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 0;
                    self.hover = 1;
                    self.animate_start = 1;
                    self.animate_quantity = 24;
                }
                CharacterDisplayType::MaleRoyal => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 799;
                    self.hover = 801;
                    self.animate_start = 714;
                    self.animate_quantity = 86;
                }
                CharacterDisplayType::FemaleRoyal => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 711;
                    self.hover = 713;
                    self.animate_start = 629;
                    self.animate_quantity = 82;
                }
                CharacterDisplayType::MaleKnight => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 449;
                    self.hover = 451;
                    self.animate_start = 378;
                    self.animate_quantity = 71;
                }
                CharacterDisplayType::FemaleKnight => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 375;
                    self.hover = 377;
                    self.animate_start = 315;
                    self.animate_quantity = 60;
                }
                CharacterDisplayType::MaleElf => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 312;
                    self.hover = 314;
                    self.animate_start = 245;
                    self.animate_quantity = 67;
                }
                CharacterDisplayType::FemaleElf => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 242;
                    self.hover = 244;
                    self.animate_start = 166;
                    self.animate_quantity = 76;
                }
                CharacterDisplayType::MaleWizard => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 626;
                    self.hover = 628;
                    self.animate_start = 531;
                    self.animate_quantity = 95;
                }
                CharacterDisplayType::FemaleWizard => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 528;
                    self.hover = 530;
                    self.animate_start = 452;
                    self.animate_quantity = 76;
                }
                CharacterDisplayType::MaleDarkElf => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 163;
                    self.hover = 165;
                    self.animate_start = 90;
                    self.animate_quantity = 73;
                }
                CharacterDisplayType::FemaleDarkElf => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 87;
                    self.hover = 89;
                    self.animate_start = 25;
                    self.animate_quantity = 62;
                }
                CharacterDisplayType::MaleDragonKnight => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 906;
                    self.hover = 907;
                    self.animate_start = 841;
                    self.animate_quantity = 65;
                }
                CharacterDisplayType::FemaleDragonKnight => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 966;
                    self.hover = 967;
                    self.animate_start = 908;
                    self.animate_quantity = 58;
                }
                CharacterDisplayType::MaleIllusionist => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 1037;
                    self.hover = 1038;
                    self.animate_start = 969;
                    self.animate_quantity = 68;
                }
                CharacterDisplayType::FemaleIllusionist => {
                    self.no_draw = false;
                    self.locked = false;
                    self.plain = 1126;
                    self.hover = 1127;
                    self.animate_start = 1039;
                    self.animate_quantity = 87;
                }
            }
        }
    }
}

impl Widget for CharacterSelectWidget {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
        self.animating = true;
    }

    fn draw_hover(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        let value = if self.animating {
            let val: u16 = self.animate_start + self.animate_index;
            val
        } else {
            if cursor {
                if let Some(i) = r.pngs.get(&self.hover) {
                    if let Loaded(_) = i {
                        self.hover
                    } else {
                        self.plain
                    }
                } else {
                    r.pngs.insert(self.hover, Loading);
                    let _e = send.blocking_send(MessageToAsync::LoadPng(self.hover));
                    self.plain
                }
            } else {
                self.plain
            }
        };
        if self.animating {
            let mut check_val = self.animate_index + 1;
            if check_val == self.animate_quantity {
                check_val = 0;
            }
            if let Some(i) = r.pngs.get(&check_val) {
                if let Loaded(_) = i {
                    if self.drawn {
                        self.drawn = false;
                        self.animate_index += 1;
                    }
                }
            } else {
                r.pngs.insert(check_val, Loading);
                let _e = send.blocking_send(MessageToAsync::LoadPng(check_val));
            }
            if self.animate_index == self.animate_quantity {
                self.animate_index = 0;
            }
        }

        self.last_draw = if !self.no_draw {
            if self.locked {
                let value = 1764;
                if r.imgs.contains_key(&value) {
                    if let Loaded(t) = &r.imgs[&value] {
                        let q = t.query();
                        let _e = canvas.copy(
                            t,
                            None,
                            Rect::new(
                                self.x.into(),
                                (self.y + 0x10).into(),
                                q.width.into(),
                                q.height.into(),
                            ),
                        );
                    }
                } else {
                    r.imgs.insert(value, Loading);
                    let _e = send.blocking_send(MessageToAsync::LoadImg(value));
                }
                None
            } else {
                if r.pngs.contains_key(&value) {
                    if let Loaded(t) = &r.pngs[&value] {
                        let q = t.query();
                        self.last_png = value;
                        self.drawn = true;
                        let _e = canvas.copy(
                            t,
                            None,
                            Rect::new(
                                self.x as i32,
                                self.y as i32,
                                q.width.into(),
                                q.height.into(),
                            ),
                        );
                        Some(ImageBox {
                            x: self.x,
                            y: self.y,
                            w: q.width as u16,
                            h: q.height as u16,
                        })
                    } else {
                        if let Some(i) = r.pngs.get(&self.last_png) {
                            if let Loaded(t) = i {
                                let q = t.query();
                                let _e = canvas.copy(
                                    t,
                                    None,
                                    Rect::new(
                                        self.x as i32,
                                        self.y as i32,
                                        q.width.into(),
                                        q.height.into(),
                                    ),
                                );
                                Some(ImageBox {
                                    x: self.x,
                                    y: self.y,
                                    w: q.width as u16,
                                    h: q.height as u16,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                } else {
                    r.pngs.insert(value, Loading);
                    let _e = send.blocking_send(MessageToAsync::LoadPng(value));
                    if let Some(i) = r.pngs.get(&self.last_png) {
                        if let Loaded(t) = i {
                            let q = t.query();
                            let _e = canvas.copy(
                                t,
                                None,
                                Rect::new(
                                    self.x as i32,
                                    self.y as i32,
                                    q.width.into(),
                                    q.height.into(),
                                ),
                            );
                            Some(ImageBox {
                                x: self.x,
                                y: self.y,
                                w: q.width as u16,
                                h: q.height as u16,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };
    }
}

pub struct SpriteWidget {
    clicked: bool,
    last_draw: Option<ImageBox>,
}

impl SpriteWidget {
    fn new<T>(
        _tc: &TextureCreator<T>,
        _x: u16,
        _y: u16,
        _text: &str,
        _font: &sdl2::ttf::Font,
    ) -> Self {
        Self {
            clicked: false,
            last_draw: None,
        }
    }
}

impl Widget for SpriteWidget {
    fn last_draw(&self) -> Option<ImageBox> {
        self.last_draw
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw_hover(
        &mut self,
        _canvas: &mut sdl2::render::WindowCanvas,
        _cursor: bool,
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        self.last_draw = None;
    }
}

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    );
    /// Down is true when the button is pressed, false when released.
    fn process_button(
        &mut self,
        button: sdl2::keyboard::Keycode,
        down: bool,
        r: &mut GameResources,
    );
    /// Perform any additional processing, before drawing, and after receiving all input events
    fn process_frame(
        &mut self,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        requests: &mut VecDeque<DrawModeRequest>,
    );
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    );
    /// Framerate is specified in frames per second
    fn framerate(&self) -> u8;
}

/// This is for exploring the resources of the game client
pub struct ExplorerMenu<'a> {
    b: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget>> = Vec::new();
        b.push(Box::new(TextButton::new(
            tc,
            50,
            100,
            "Png browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            114,
            "Img browser",
            &r.font,
        )));
        b.push(Box::new(TextButton::new(
            tc,
            50,
            128,
            "Sprite browser",
            &r.font,
        )));
        Self { b: b }
    }
}

impl<'a> GameMode for ExplorerMenu<'a> {
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
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::PngExplorer));
        }
        if self.b[1].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::ImgExplorer));
        }
        if self.b[2].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::SprExplorer));
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
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
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
        let value = 811;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// This is for exploring the resources of the game client
pub struct GameLoader<'a> {
    b: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> GameLoader<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, _r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(PlainColorButton::new(tc, 50, 50, 50, 50)));
        Self { b: b }
    }
}

impl<'a> GameMode for GameLoader<'a> {
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
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Login));
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
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
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
        let value = 811;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = 330;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(241, 385, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }
        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for user login
pub struct Login<'a> {
    b: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> Login<'a> {
    pub fn new<T>(_tc: &'a TextureCreator<T>) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(ImgButton::new(53, 0x213, 0x183)));
        b.push(Box::new(ImgButton::new(65, 0x213, 0x195)));
        b.push(Box::new(ImgButton::new(55, 0x213, 0x1a8)));
        b.push(Box::new(ImgButton::new(57, 0x213, 0x1c2)));
        Self { b: b }
    }
}

impl<'a> GameMode for Login<'a> {
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
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
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
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
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
        let value = 814;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = 59;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0x1a9, 0x138, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for selection of which character to play
pub struct CharacterSelect<'a> {
    b: Vec<Box<dyn Widget + 'a>>,
    char_sel: Vec<CharacterSelectWidget>,
    page: u8,
    selection: Option<u8>,
    //1764.img for disabled slot
}

impl<'a> CharacterSelect<'a> {
    pub fn new<T>(_tc: &'a TextureCreator<T>, _r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(ImgButton::new(0x6e5, 0x0f7, 0x10b)));
        b.push(Box::new(ImgButton::new(0x6e7, 0x16c, 0x10b)));
        b.push(Box::new(ImgButton::new(0x334, 0x20d, 0x185)));
        b.push(Box::new(ImgButton::new(0x336, 0x20d, 0x19a)));
        b.push(Box::new(ImgButton::new(0x134, 0x20d, 0x1b5)));
        let mut ch = Vec::new();

        ch.push(CharacterSelectWidget::new(0x13, 0));
        ch.push(CharacterSelectWidget::new(0xb0, 0));
        ch.push(CharacterSelectWidget::new(0x14d, 0));
        ch.push(CharacterSelectWidget::new(0x1ea, 0));
        Self {
            b: b,
            char_sel: ch,
            page: 0,
            selection: None,
        }
    }
}

impl<'a> GameMode for CharacterSelect<'a> {
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
                    for w in &mut self.char_sel {
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
        self.char_sel[0].set_type(r.characters[(0 + self.page * 4) as usize].t);
        self.char_sel[1].set_type(r.characters[(1 + self.page * 4) as usize].t);
        self.char_sel[2].set_type(r.characters[(2 + self.page * 4) as usize].t);
        self.char_sel[3].set_type(r.characters[(3 + self.page * 4) as usize].t);

        if self.b[0].was_clicked() {
            if self.page > 0 {
                self.page -= 1;
                //todo update the animation data for each char_sel widget
                self.char_sel[0].set_animating(false);
                self.char_sel[1].set_animating(false);
                self.char_sel[2].set_animating(false);
                self.char_sel[3].set_animating(false);
                self.selection = None;
            }
        }
        if self.b[1].was_clicked() {
            if self.page < 1 {
                self.page += 1;
                //todo update the animation data for each char_sel widget
                self.selection = None;
                self.char_sel[0].set_animating(false);
                self.char_sel[1].set_animating(false);
                self.char_sel[2].set_animating(false);
                self.char_sel[3].set_animating(false);
            }
        }

        if self.char_sel[0].was_clicked() {
            self.selection = Some(4 * self.page + 0);
            self.char_sel[0].set_animating(true);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[1].was_clicked() {
            self.selection = Some(4 * self.page + 1);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(true);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[2].was_clicked() {
            self.selection = Some(4 * self.page + 2);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(true);
            self.char_sel[3].set_animating(false);
        } else if self.char_sel[3].was_clicked() {
            self.selection = Some(4 * self.page + 3);
            self.char_sel[0].set_animating(false);
            self.char_sel[1].set_animating(false);
            self.char_sel[2].set_animating(false);
            self.char_sel[3].set_animating(true);
        }

        if self.b[2].was_clicked() {
            if let Some(c) = self.selection {
                match r.characters[c as usize].t {
                    CharacterDisplayType::NewCharacter => {
                        requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::NewCharacter));
                    }
                    _ => {
                        requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Game));
                    }
                }
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
        let value = 815;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let _e = canvas.copy(t, None, None);
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

        let value = if self.page == 0 { 0x6ea } else { 0x6e9 };
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0x127, 0x10f, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        let value = if self.page == 1 { 0x6ec } else { 0x6eb };
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0x146, 0x10f, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
        for w in &mut self.char_sel {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for selection of which character to play
pub struct Game<'a> {
    b: Vec<Box<dyn Widget + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    sprites: Vec<SpriteWidget>,
}

impl<'a> Game<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(PlainColorButton::new(tc, 50, 50, 50, 50)));
        let mut d = Vec::new();
        d.push(DynamicTextWidget::new(
            tc,
            35,
            390,
            "0",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            35,
            407,
            "1",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));
        d.push(DynamicTextWidget::new(
            tc,
            35,
            426,
            "2",
            &r.font,
            sdl2::pixels::Color::WHITE,
        ));

        Self {
            b: b,
            disp: d,
            sprites: Vec::new(),
        }
    }
}

impl<'a> GameMode for Game<'a> {
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
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
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
        _r: &mut GameResources,
        _send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
        _requests: &mut VecDeque<DrawModeRequest>,
    ) {
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

        let value = 1028;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(t, None, Rect::new(0, 368, q.width.into(), q.height.into()));
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        let value = 1019;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(485, 366, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        let value = 1029;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(t, None, Rect::new(3, 386, q.width.into(), q.height.into()));
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        let value = 1030;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(t, None, Rect::new(3, 402, q.width.into(), q.height.into()));
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        let value = 1031;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(t, None, Rect::new(3, 423, q.width.into(), q.height.into()));
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

        for w in &mut self.disp {
            w.draw(canvas, cursor, r, send);
        }

        for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for user login
pub struct PngExplorer<'a, T> {
    b: Vec<Box<dyn Widget + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_png: u16,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> PngExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
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

        Self {
            b: b,
            disp: disp,
            current_png: 0,
            tc: tc,
        }
    }
}

impl<'a, T> GameMode for PngExplorer<'a, T> {
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
                    if self.current_png > 0 {
                        r.pngs.remove(&self.current_png);
                        self.current_png -= 1;
                        let words = format!("Displaying {}.png", self.current_png);
                        self.disp[0].update_text(self.tc, &words, &r.font);
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_png < 65534 {
                        r.pngs.remove(&self.current_png);
                        self.current_png += 1;
                        let words = format!("Displaying {}.png", self.current_png);
                        self.disp[0].update_text(self.tc, &words, &r.font);
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
        let value = self.current_png;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let q = t.query();
                let _e = canvas.copy(t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

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

/// The screen that allows for user login
pub struct ImgExplorer<'a, T> {
    b: Vec<Box<dyn Widget + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_img: u16,
    prev_img: u16,
    tc: &'a TextureCreator<T>,
    displayed: bool,
}

impl<'a, T> ImgExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>, r: &mut GameResources) -> Self {
        let mut b: Vec<Box<dyn Widget + 'a>> = Vec::new();
        b.push(Box::new(TextButton::new(tc, 320, 400, "Go Back", &r.font)));
        let mut disp = Vec::new();
        disp.push(DynamicTextWidget::new(
            tc,
            320,
            386,
            "Displaying 0.img",
            &r.font,
            sdl2::pixels::Color::RED,
        ));

        Self {
            b: b,
            disp: disp,
            current_img: 0,
            prev_img: 0,
            tc: tc,
            displayed: false,
        }
    }
}

impl<'a, T> GameMode for ImgExplorer<'a, T> {
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
                    if self.current_img > 0 {
                        if self.displayed {
                            self.prev_img = self.current_img;
                            self.current_img -= 1;
                            let words = format!("Displaying {}.img", self.current_img);
                            self.disp[0].update_text(self.tc, &words, &r.font);
                            self.displayed = false;
                        }
                    }
                }
                sdl2::keyboard::Keycode::Right => {
                    if self.current_img < 65534 {
                        if self.displayed {
                            self.prev_img = self.current_img;
                            self.current_img += 1;
                            let words = format!("Displaying {}.img", self.current_img);
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
        let value = self.current_img;
        let mut remove_prev = false;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                if self.prev_img != self.current_img {
                    remove_prev = true;
                }
                let _e = canvas.copy(t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
                self.displayed = true;
            } else {
                let value = self.prev_img;
                if r.imgs.contains_key(&value) {
                    if let Loaded(t) = &r.imgs[&value] {
                        let q = t.query();
                        let _e =
                            canvas.copy(t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
                        self.displayed = true;
                    }
                }
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
            let value = self.prev_img;
            if r.imgs.contains_key(&value) {
                if let Loaded(t) = &r.imgs[&value] {
                    let q = t.query();
                    let _e = canvas.copy(t, None, Rect::new(0, 0, q.width.into(), q.height.into()));
                    self.displayed = true;
                }
            }
        }
        if remove_prev {
            r.imgs.remove(&self.prev_img);
        }

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

pub struct SprExplorer<'a, T> {
    b: Vec<Box<dyn Widget + 'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_spr_a: u16,
    current_spr_b: u16,
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

        Self {
            b: b,
            disp: disp,
            current_spr_a: 0,
            current_spr_b: 0,
            tc: tc,
            displayed: false,
        }
    }
}

impl<'a, T> GameMode for SprExplorer<'a, T> {
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
    }

    fn framerate(&self) -> u8 {
        20
    }
}
