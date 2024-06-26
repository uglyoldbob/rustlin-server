use std::rc::Rc;

use crate::widgets::WidgetTrait;
use crate::GameResources;
use crate::ImageBox;
use sdl2::rect::Rect;
use sdl2::render::Texture;

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
pub struct CharacterStats {
    pub str: u8,
    pub dex: u8,
    pub con: u8,
    pub wis: u8,
    pub cha: u8,
    pub int: u8,
}

impl CharacterDisplayType {
    pub fn get_base_stats(&self) -> CharacterStats {
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

    pub fn get_max_stats(&self) -> CharacterStats {
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

pub struct CharacterSelectWidget<'a> {
    plain: u16,
    plain_obj: Option<Rc<Texture<'a>>>,
    hover: u16,
    hover_obj: Option<Rc<Texture<'a>>>,
    locked_obj: Option<Rc<Texture<'a>>>,
    images: Vec<Option<Rc<Texture<'a>>>>,
    last_png: u16,
    pub t: CharacterDisplayType,
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

impl<'a> CharacterSelectWidget<'a> {
    pub fn new(x: u16, y: u16, r: &mut GameResources<'a, '_, '_>) -> Self {
        let mut s = Self {
            images: Vec::new(),
            plain: 0,
            plain_obj: r.get_or_load_png(0),
            hover: 1,
            hover_obj: r.get_or_load_png(1),
            locked_obj: r.get_or_load_img(1764),
            last_png: 0,
            t: CharacterDisplayType::Locked,
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
        };
        s.set_type(CharacterDisplayType::Blank, r);
        s
    }
}

impl<'a> CharacterSelectWidget<'a> {
    pub fn set_animating(&mut self, a: bool) {
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

    pub fn set_type(&mut self, t: CharacterDisplayType, r: &mut GameResources<'a, '_, '_>) {
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
            self.images.clear();
            for i in 0..self.animate_quantity {
                let img = r.get_or_load_png(self.animate_start + i);
                self.images.push(img);
            }
            self.plain_obj = r.get_or_load_png(self.plain);
            self.hover_obj = r.get_or_load_png(self.hover);
        }
    }
}

impl<'a> WidgetTrait<'a> for CharacterSelectWidget<'a> {
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
        _r: &mut GameResources,
    ) {
        if self.animating {
            if self.animate_index >= (self.animate_quantity - 1) {
                self.animate_index = 0;
            }
        }

        let value = if self.locked {
            &self.locked_obj
        } else if self.animating {
            let val: u16 = self.animate_index;
            &self.images[val as usize]
        } else {
            if cursor {
                if let Some(_i) = &self.hover_obj {
                    &self.hover_obj
                } else {
                    &self.plain_obj
                }
            } else {
                &self.plain_obj
            }
        };

        self.last_draw = if let Some(t) = value {
            let q = t.query();
            self.animate_index += 1;
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
        };
    }
}
