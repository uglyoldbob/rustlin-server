use crate::mode::*;
use crate::mouse::*;
use crate::resources::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::collections::VecDeque;
use std::fs;
use std::time::Duration;

pub mod settings;

pub const EMBEDDED_FONT: &[u8] = include_bytes!("cmsltt10.ttf");

fn make_dummy_texture<'a, T>(tc: &'a TextureCreator<T>) -> Texture<'a> {
    let mut data: Vec<u8> = vec![0; (4 * 4 * 2) as usize];
    let mut surf = sdl2::surface::Surface::from_data(
        data.as_mut_slice(),
        4,
        4,
        (2 * 4) as u32,
        PixelFormatEnum::RGB555,
    )
    .unwrap();
    let _e = surf.set_color_key(true, sdl2::pixels::Color::BLACK);
    Texture::from_surface(&surf, tc).unwrap()
}

fn mode_maker<'a, T: 'a>(
    mode: DrawMode,
    texture_creator: &'a TextureCreator<T>,
    game_resources: &mut GameResources<'a, '_, '_>,
) -> GameMode<'a, T> {
    match mode {
        DrawMode::Explorer => {
            GameMode::Explorer(ExplorerMenu::new(texture_creator, game_resources))
        }
        DrawMode::PngExplorer => {
            GameMode::PngExplorer(PngExplorer::new(texture_creator, game_resources))
        }
        DrawMode::ImgExplorer => {
            GameMode::ImgExplorer(ImgExplorer::new(texture_creator, game_resources))
        }
        DrawMode::SprExplorer => {
            GameMode::SprExplorer(SprExplorer::new(texture_creator, game_resources))
        }
        DrawMode::TileExplorer => {
            GameMode::TileExplorer(TileExplorer::new(texture_creator, game_resources))
        }
        DrawMode::MapExplorer => {
            GameMode::MapExplorer(MapExplorer::new(texture_creator, game_resources))
        }
        DrawMode::GameLoader => GameMode::Loader(GameLoader::new(texture_creator, game_resources)),
        DrawMode::Login => GameMode::Login(Login::new::<WindowContext>(game_resources)),
        DrawMode::CharacterSelect => {
            GameMode::CharacterSelect(CharacterSelect::new(texture_creator, game_resources))
        }
        DrawMode::NewCharacter => {
            GameMode::NewCharacter(NewCharacterMode::new(texture_creator, game_resources))
        }
        DrawMode::Game => GameMode::Game(Game::new(texture_creator, game_resources)),
        DrawMode::WavPlayer => GameMode::WavPlayer(WavPlayer::new(texture_creator, game_resources)),
    }
}

pub fn startup(mode: DrawMode) {
    let settings_file =
        fs::read_to_string("./client-settings.ini").expect("Failed to read client-settings.ini");
    let settings_result = toml::from_str(&settings_file);
    if let Err(e) = &settings_result {
        println!("Failed to read settings {}", e);
    }
    let settings: settings::Settings = settings_result.unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();
    let efont = sdl2::rwops::RWops::from_bytes(EMBEDDED_FONT).unwrap();
    let font = ttf_context.load_font_from_rwops(efont, 14).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    video_subsystem.text_input().start();

    let mut vid_win = video_subsystem.window("l1j-client", 640, 480);
    let mut windowb = vid_win.position_centered();

    if !settings.window {
        windowb = windowb.fullscreen();
    }
    let window = windowb.opengl().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let i = sdl2::mixer::InitFlag::MP3;
    let _sdl2mixer = sdl2::mixer::init(i).unwrap();
    let audio = sdl2::mixer::open_audio(44100, 16, 2, 1024);
    println!("Audio is {:?}", audio);

    let mut game_resources = GameResources::new(font, settings.game_resources, &texture_creator);

    //let sprtable = SpriteTable::load_embedded_table();

    let mut mode: GameMode<WindowContext> = mode_maker(mode, &texture_creator, &mut game_resources);

    println!(
        "Struct GameResources has size {}",
        std::mem::size_of::<GameResources>()
    );
    dbg!(stacker::remaining_stack());

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let dummy_texture = make_dummy_texture(&texture_creator);

    let flags = sdl2::image::InitFlag::all();
    let _sdl2_image = sdl2::image::init(flags).unwrap();

    let mut mouse = Mouse::new();
    let mut drawmode_commands: VecDeque<DrawModeRequest> = VecDeque::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x,
                    y,
                    xrel: _,
                    yrel: _,
                } => {
                    mouse.event(MouseEventInput::Move(x as i16, y as i16));
                }
                Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x: _,
                    y: _,
                } => match mouse_btn {
                    MouseButton::Left => {
                        mouse.event(MouseEventInput::LeftDown);
                    }
                    MouseButton::Middle => {
                        mouse.event(MouseEventInput::MiddleDown);
                    }
                    MouseButton::Right => {
                        mouse.event(MouseEventInput::RightDown);
                    }
                    MouseButton::X1 => {
                        mouse.event(MouseEventInput::ExtraDown);
                    }
                    MouseButton::X2 => {
                        mouse.event(MouseEventInput::Extra2Down);
                    }
                    _ => {}
                },
                Event::MouseButtonUp {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x: _,
                    y: _,
                } => match mouse_btn {
                    MouseButton::Left => {
                        mouse.event(MouseEventInput::LeftUp);
                    }
                    MouseButton::Middle => {
                        mouse.event(MouseEventInput::MiddleUp);
                    }
                    MouseButton::Right => {
                        mouse.event(MouseEventInput::RightUp);
                    }
                    _ => {}
                },
                Event::MouseWheel {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    x: _,
                    y,
                    direction: _,
                    precise_x: _,
                    precise_y: _,
                } => {
                    mouse.event(MouseEventInput::Scrolling(y));
                }
                Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: m,
                    repeat: _,
                } => {
                    if let Some(key) = keycode {
                        mode.process_button(key, m, true, &mut game_resources);
                    }
                }
                Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: m,
                    repeat: _,
                } => {
                    if let Some(key) = keycode {
                        mode.process_button(key, m, false, &mut game_resources);
                    }
                }
                Event::TextInput {
                    timestamp: _,
                    window_id: _,
                    text,
                } => {
                    mode.process_text(&text);
                }
                _ => {}
            }
        }
        mouse.parse();
        mode.process_mouse(mouse.events(), &mut drawmode_commands);
        mouse.clear();
        mode.process_frame(&mut game_resources, &mut drawmode_commands);
        while let Some(m) = drawmode_commands.pop_front() {
            match m {
                DrawModeRequest::ChangeDrawMode(m) => {
                    mode = mode_maker(m, &texture_creator, &mut game_resources);
                }
            }
        }
        mode.draw(&mut canvas, mouse.cursor(), &mut game_resources);
        let _e = canvas.copy(&dummy_texture, None, None);
        canvas.present();
        let framerate = mode.framerate() as u64;
        std::thread::sleep(Duration::from_nanos(1_000_000_000u64 / framerate));
    }
}
