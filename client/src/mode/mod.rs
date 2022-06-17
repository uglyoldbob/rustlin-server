use crate::GameResources;
use crate::MessageFromAsync;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode {
    fn parse_message(&mut self, m: &MessageFromAsync, r: &mut GameResources);
    fn parse_event(&mut self, e: sdl2::event::Event, r: &mut GameResources);
    fn draw(&mut self, canvas: &mut sdl2::render::WindowCanvas, r: &mut GameResources);
    /// Framerate is specified in frames per second
    fn framerate(&self) -> u8;
}

/// This is for exploring the resources of the game client
pub struct ExplorerMenu {
}

impl ExplorerMenu {
    pub fn new() -> Self {
        Self { }
    }
}

impl GameMode for ExplorerMenu {
    fn parse_message(&mut self, m: &MessageFromAsync, r: &mut GameResources) {
        match m {
            MessageFromAsync::ResourceStatus(_b) => {}
            MessageFromAsync::StringTable(_name, _data) => {}
            MessageFromAsync::Png(_name, _data) => {}
        }
    }

    fn parse_event(&mut self, e: sdl2::event::Event, r: &mut GameResources) {
        match e {
            _ => {}
        }
    }

    fn draw(&mut self, canvas: &mut sdl2::render::WindowCanvas, r: &mut GameResources) {
        canvas.set_draw_color(Color::RGB(0,0,0));
	canvas.clear();
	if r.pngs.contains_key(&811) {
		println!("draw 811.png");
		let p = &r.pngs[&811];
		let result = canvas.copy(p, None, None);
		if let Err(e) = result {
			println!("FAILED to draw {:?}", e);
		}
	}
    }

    fn framerate(&self) -> u8 {
        20
    }
}
