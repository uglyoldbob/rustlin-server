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
    GameLoader,
    Login,
    CharacterSelect,
    Game,
}

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

/// All of the various kinds of widgets that can exist in the game
pub enum WidgetEnum<'a> {
    PlainColorButton(PlainColorButton<'a>),
    TextButton(TextButton<'a>),
    DynamicTextWidget(DynamicTextWidget<'a>),
    ImgButton(ImgButton),
    CharacterSelect(CharacterSelectWidget),
}

pub struct Widget<'a> {
	widget: WidgetEnum<'a>,
	last_draw: Option<ImageBox>,
}

impl<'a> Widget<'a> {
    fn new(we: WidgetEnum<'a>) -> Self {
	Self {
		widget: we,
		last_draw: None,
	}
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
	let cursor = if let Some(c) = cursor {
		let (x,y) = c;
		self.contains(x,y)
	}
	else
	{
		false
	};
        self.last_draw = self.widget.draw(canvas, cursor, r, send);
    }
    fn left_click(&mut self) {
        self.widget.left_click();
    }
    fn was_clicked(&mut self) -> bool {
        self.widget.was_clicked()
    }
    fn contains(&self, x: i16, y: i16) -> bool {
	if let Some(t) = &self.last_draw {
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
	}
	else {
		false
	}
    }
}

impl<'a> WidgetEnum<'a> {
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
        match self {
            WidgetEnum::PlainColorButton(button) => {
                button.draw(canvas, cursor, r, send)
            }
	    WidgetEnum::ImgButton(button) => {
		button.draw(canvas, cursor, r, send)
	    }
	    WidgetEnum::DynamicTextWidget(widget) => {
	        widget.draw(canvas, cursor, r, send)
	    }
	    WidgetEnum::CharacterSelect(button) => {
	        button.draw(canvas, cursor, r, send)
	    }
	    WidgetEnum::TextButton(button) => {
		button.draw(canvas, cursor, r, send)
	    }
        }
    }
    
    fn left_click(&mut self) {
        match self {
            WidgetEnum::PlainColorButton(button) => {
                button.clicked();
            }
	    WidgetEnum::ImgButton(button) => {
		button.clicked();
	    }
	    WidgetEnum::DynamicTextWidget(widget) => {
	        widget.clicked();
	    }
	    WidgetEnum::CharacterSelect(button) => {
	        button.clicked();
	    }
	    WidgetEnum::TextButton(button) => {
		button.clicked();
	    }
        }
    }
    
    fn was_clicked(&mut self) -> bool {
        match self {
            WidgetEnum::PlainColorButton(button) => button.was_clicked(),
	    WidgetEnum::ImgButton(button) => button.was_clicked(),
	    WidgetEnum::DynamicTextWidget(widget) => widget.was_clicked(),
	    WidgetEnum::CharacterSelect(button) => button.was_clicked(),
	    WidgetEnum::TextButton(button) => button.was_clicked(),
        }
    }
}

pub struct PlainColorButton<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
}

impl<'a> PlainColorButton<'a> {
    fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, w: u16, h: u16) -> Self {
        let mut data : Vec<u8>= vec![0xff; (w * h * 2) as usize];
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
        }
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
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
	Some(ImageBox{x:self.x,
			y: self.y,
			w: q.width as u16,
			h: q.height as u16,
		})
    }
}

pub struct TextButton<'a> {
    t: Texture<'a>,
    t2: Texture<'a>,
    x: u16,
    y: u16,
    clicked: bool,
}

impl<'a> TextButton<'a> {
    fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, text: &str,
	font: &sdl2::ttf::Font) -> Self {
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
        }
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
	let t = if cursor { &self.t2} else { &self.t };
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
	Some(ImageBox{x:self.x,
			y: self.y,
			w: q.width as u16,
			h: q.height as u16,
		})
    }
}

pub struct ImgButton {
    num: u16,
    x: u16,
    y: u16,
    clicked: bool,
}


impl ImgButton {
	fn new(num: u16, x: u16, y: u16) -> Self {
        Self {
            num: num,
            x: x,
            y: y,
            clicked: false,
        }
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw<'a>(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
	let value = if cursor { self.num + 1} else { self.num };
	if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(self.x as i32,self.y as i32, q.width.into(), q.height.into()),
                );
		Some(ImageBox{x:self.x,
			y: self.y,
			w: q.width as u16,
			h: q.height as u16,
		})
            }
	    else {
		None
	    }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
	    None
        }
    }
}

pub struct DynamicTextWidget<'a> {
    t: Texture<'a>,
    x: u16,
    y: u16,
    s: String,
}

impl<'a> DynamicTextWidget<'a> {
    fn new<T>(tc: &'a TextureCreator<T>, x: u16, y: u16, text: &str,
	font: &sdl2::ttf::Font) -> Self {
	let pr = font.render(text);
	let ft = pr.solid(sdl2::pixels::Color::RED).unwrap();
	
        Self {
            t: Texture::from_surface(&ft, tc).unwrap(),
            x: x,
            y: y,
            s: text.to_string(),
        }
    }
    
    fn update_text<T>(&mut self, tc: &'a TextureCreator<T>, 
        text: &str,
	font: &sdl2::ttf::Font) {
	if (text != self.s) {
	    let pr = font.render(text);
	    let ft = pr.solid(sdl2::pixels::Color::RED).unwrap();
	    self.t = Texture::from_surface(&ft, tc).unwrap();
	    self.s = text.to_string();
	}
    }

    fn was_clicked(&mut self) -> bool {
        false
    }

    fn clicked(&mut self) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
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
	Some(ImageBox{x:self.x,
			y: self.y,
			w: q.width as u16,
			h: q.height as u16,
		})
    }
}

pub struct CharacterSelectWidget {
    plain: u16,
    hover: u16,
    animating: bool,
    x: u16,
    y: u16,
    clicked: bool,
}

impl CharacterSelectWidget {
	fn new(x: u16, y: u16) -> Self {
        Self {
            plain: 0,
	    hover: 1,
	    animating: false,
            x: x,
            y: y,
            clicked: false,
        }
    }

    fn was_clicked(&mut self) -> bool {
        let ret = self.clicked;
        self.clicked = false;
        ret
    }

    fn clicked(&mut self) {
        self.clicked = true;
    }

    fn draw<'a>(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: bool,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) -> Option<ImageBox>{
	let value = if cursor { self.hover} else { self.plain };
	if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(self.x as i32,self.y as i32, q.width.into(), q.height.into()),
                );
		Some(ImageBox{x:self.x,
			y: self.y,
			w: q.width as u16,
			h: q.height as u16,
		})
            }
	    else {
		None
	    }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
	    None
        }
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
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    );
    /// Framerate is specified in frames per second
    fn framerate(&self) -> u8;
}

/// This is for exploring the resources of the game client
pub struct ExplorerMenu<'a> {
    b: Vec<Widget<'a>>,
}

impl<'a> ExplorerMenu<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>,
	r: &mut GameResources) -> Self {
        let mut b = Vec::new();
	b.push(Widget::new(WidgetEnum::TextButton(TextButton::new(
	    tc, 50, 100, "Png browser", &r.font))));
	b.push(Widget::new(WidgetEnum::TextButton(TextButton::new(
	    tc, 50, 114, "Img browser", &r.font))));
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::PngExplorer));
            println!("You clicked the button");
        }
	if self.b[1].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::ImgExplorer));
            println!("You clicked the button");
        }
    }
    
    fn process_button(
	&mut self,
	button: sdl2::keyboard::Keycode,
	down: bool,
	r: &mut GameResources,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
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
    b: Vec<Widget<'a>>,
}

impl<'a> GameLoader<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>,
	r: &mut GameResources) -> Self {
        let mut b = Vec::new();
	b.push(Widget::new(WidgetEnum::PlainColorButton(PlainColorButton::new(
            tc, 50, 50, 50, 50,
        ))));
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Login));
            println!("You clicked the button");
        }
    }
    
    fn process_button(
	&mut self,
	button: sdl2::keyboard::Keycode,
	down: bool,
	r: &mut GameResources,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
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
    b: Vec<Widget<'a>>,
}

impl<'a> Login<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        let mut b = Vec::new();
        b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(53,0x213,0x183))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(65,0x213,0x195))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(55,0x213,0x1a8))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(57,0x213,0x1c2))));
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
            println!("You clicked the button");
        }
    }
    
    fn process_button(
	&mut self,
	button: sdl2::keyboard::Keycode,
	down: bool,
	r: &mut GameResources,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
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
    b: Vec<Widget<'a>>,
}

impl<'a> CharacterSelect<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        let mut b = Vec::new();
        b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(0x6e5,0x0f7,0x10b))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(0x6e7,0x16c,0x10b))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(0x334,0x20d,0x185))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(0x336,0x20d,0x19a))));
	b.push(Widget::new(WidgetEnum::ImgButton(ImgButton::new(0x134,0x20d,0x1b5))));
	b.push(Widget::new(WidgetEnum::CharacterSelect(CharacterSelectWidget::new(0x13, 0x13))));
	b.push(Widget::new(WidgetEnum::CharacterSelect(CharacterSelectWidget::new(0xb0, 0x13))));
	b.push(Widget::new(WidgetEnum::CharacterSelect(CharacterSelectWidget::new(0x14d, 0x13))));
	b.push(Widget::new(WidgetEnum::CharacterSelect(CharacterSelectWidget::new(0x1ea, 0x13))));
        Self { b: b }
    }
}

impl<'a> GameMode for CharacterSelect<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    ) {
        for e in events {
            match e {
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[2].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Game));
            println!("You clicked the button");
        }
    }
    
    fn process_button(
	&mut self,
	button: sdl2::keyboard::Keycode,
	down: bool,
	r: &mut GameResources,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
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

	let value = 0x6e9;
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
	
	let value = 0x6eb;
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
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for selection of which character to play
pub struct Game<'a> {
    b: Vec<Widget<'a>>,
}

impl<'a> Game<'a> {
    pub fn new<T>(tc: &'a TextureCreator<T>) -> Self {
        let mut b = Vec::new();
        b.push(Widget::new(WidgetEnum::PlainColorButton(PlainColorButton::new(
            tc, 50, 50, 50, 50,
        ))));
        Self { b: b }
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::CharacterSelect));
            println!("You clicked the button");
        }
    }
    
    fn process_button(
	&mut self,
	button: sdl2::keyboard::Keycode,
	down: bool,
	r: &mut GameResources,
    ) {
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

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
    b: Vec<Widget<'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_png: u16,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> PngExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>,
        r: &mut GameResources) -> Self {
        let mut b = Vec::new();
	b.push(Widget::new(WidgetEnum::TextButton(TextButton::new(
	    tc, 320, 400, "Go Back", &r.font))));
	let mut disp = Vec::new();
	disp.push(DynamicTextWidget::new(tc, 320, 386, "Displaying 0.png", &r.font));
	
        Self { b: b,
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Explorer));
            println!("You clicked the button");
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
				println!("Pressed left");
			}
			sdl2::keyboard::Keycode::Right => {
				if self.current_png < 65534 {
					r.pngs.remove(&self.current_png);
					self.current_png += 1;
					let words = format!("Displaying {}.png", self.current_png);
					self.disp[0].update_text(self.tc, &words, &r.font);
				}
				println!("Pressed right");
			}
			_ => {}
		}
	}
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let value = self.current_png;
        if r.pngs.contains_key(&value) {
            if let Loaded(t) = &r.pngs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0, 0, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.pngs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadPng(value));
        }

	for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
	for w in &mut self.disp {
	    w.draw(canvas, false, r, send);
	}
    }

    fn framerate(&self) -> u8 {
        20
    }
}

/// The screen that allows for user login
pub struct ImgExplorer<'a, T> {
    b: Vec<Widget<'a>>,
    disp: Vec<DynamicTextWidget<'a>>,
    current_img: u16,
    tc: &'a TextureCreator<T>,
}

impl<'a, T> ImgExplorer<'a, T> {
    pub fn new(tc: &'a TextureCreator<T>,
        r: &mut GameResources) -> Self {
        let mut b = Vec::new();
	b.push(Widget::new(WidgetEnum::TextButton(TextButton::new(
	    tc, 320, 400, "Go Back", &r.font))));
	let mut disp = Vec::new();
	disp.push(DynamicTextWidget::new(tc, 320, 386, "Displaying 0.img", &r.font));
	
        Self { b: b,
		disp: disp,
		current_img: 0,
		tc: tc,
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
                MouseEventOutput::Move((x, y)) => {
                }
                MouseEventOutput::LeftDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::MiddleDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::RightDrag { from, to } => {
                    let (x, y) = to;
                }
                MouseEventOutput::DragStop => {
                }
                MouseEventOutput::LeftClick((x, y)) => {
                    for w in &mut self.b {
                        if w.contains(*x, *y) {
                            w.left_click();
                        }
                    }
                }
                MouseEventOutput::MiddleClick((x, y)) => {
                }
                MouseEventOutput::RightClick((x, y)) => {
                }
                MouseEventOutput::ExtraClick => {
                }
                MouseEventOutput::Extra2Click => {
                }
                MouseEventOutput::Scrolling(amount) => {
                }
            }
        }

        if self.b[0].was_clicked() {
            requests.push_back(DrawModeRequest::ChangeDrawMode(DrawMode::Explorer));
            println!("You clicked the button");
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
					r.imgs.remove(&self.current_img);
					self.current_img -= 1;
					let words = format!("Displaying {}.img", self.current_img);
					self.disp[0].update_text(self.tc, &words, &r.font);
				}
				println!("Pressed left");
			}
			sdl2::keyboard::Keycode::Right => {
				if self.current_img < 65534 {
					r.imgs.remove(&self.current_img);
					self.current_img += 1;
					let words = format!("Displaying {}.img", self.current_img);
					self.disp[0].update_text(self.tc, &words, &r.font);
				}
				println!("Pressed right");
			}
			_ => {}
		}
	}
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
	cursor: Option<(i16,i16)>,
        r: &mut GameResources,
        send: &mut tokio::sync::mpsc::Sender<MessageToAsync>,
    ) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let value = self.current_img;
        if r.imgs.contains_key(&value) {
            if let Loaded(t) = &r.imgs[&value] {
                let q = t.query();
                let _e = canvas.copy(
                    t,
                    None,
                    Rect::new(0, 0, q.width.into(), q.height.into()),
                );
            }
        } else {
            r.imgs.insert(value, Loading);
            let _e = send.blocking_send(MessageToAsync::LoadImg(value));
        }

	for w in &mut self.b {
            w.draw(canvas, cursor, r, send);
        }
	for w in &mut self.disp {
	    w.draw(canvas, false, r, send);
	}
    }

    fn framerate(&self) -> u8 {
        20
    }
}
