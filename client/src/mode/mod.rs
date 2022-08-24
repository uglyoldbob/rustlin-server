use crate::mouse::MouseEventOutput;
use crate::GameResources;
use std::collections::VecDeque;

pub enum DrawMode {
    Explorer,
    PngExplorer,
    ImgExplorer,
    SprExplorer,
    WavPlayer,
    TileExplorer,
    MapExplorer,
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

/// This trait is used to determine what mode of operation the program is in
pub trait GameMode<'a> {
    fn process_mouse(
        &mut self,
        events: &Vec<MouseEventOutput>,
        requests: &mut VecDeque<DrawModeRequest>,
    );
    /// Down is true when the button is pressed, false when released.
    fn process_button(
        &mut self,
        button: sdl2::keyboard::Keycode,
        m: sdl2::keyboard::Mod,
        down: bool,
        r: &mut GameResources<'a, '_, '_>,
    );
    fn process_text(&mut self, s: &String) {}
    /// Perform any additional processing, before drawing, and after receiving all input events
    fn process_frame(
        &mut self,
        r: &mut GameResources<'a, '_, '_>,
        requests: &mut VecDeque<DrawModeRequest>,
    );
    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        cursor: Option<(i16, i16)>,
        r: &mut GameResources<'a, '_, '_>,
    );
    /// Framerate is specified in frames per second
    fn framerate(&self) -> u8;
}

pub mod explorer;
pub use explorer::ExplorerMenu;

pub mod loader;
pub use loader::GameLoader;

pub mod login;
pub use login::Login;

pub mod character_select;
pub use character_select::CharacterSelect;

pub mod game;
pub use game::Game;

pub mod png_explorer;
pub use png_explorer::PngExplorer;

pub mod img_explorer;
pub use img_explorer::ImgExplorer;

pub mod newchar;
pub use newchar::NewCharacterMode;

pub mod spr_explorer;
pub use spr_explorer::SprExplorer;

pub mod wav_player;
pub use wav_player::WavPlayer;

pub mod tile_explorer;
pub use tile_explorer::TileExplorer;

pub mod map_explorer;
pub use map_explorer::MapExplorer;
