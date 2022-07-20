use crate::mode::*;
use crate::mouse::*;
use crate::resources::*;
use crate::Loadable::Loaded;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mixer::LoaderRWops;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::VecDeque;
use std::fs;
use std::time::Duration;

mod settings;

const EMBEDDED_FONT: &[u8] = include_bytes!("cmsltt10.ttf");

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

pub fn startup(mode: DrawMode) {
    let settings_file = fs::read_to_string("./client-settings.ini");
    let settings_con = match settings_file {
        Ok(con) => con,
        Err(_) => "".to_string(),
    };
    let mut settings = configparser::ini::Ini::new();
    let settings_result = settings.read(settings_con);
    if let Err(e) = settings_result {
        println!("Failed to read settings {}", e);
    }

    let ttf_context = sdl2::ttf::init().unwrap();
    let efont = sdl2::rwops::RWops::from_bytes(EMBEDDED_FONT).unwrap();
    let font = ttf_context.load_font_from_rwops(efont, 14).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut vid_win = video_subsystem.window("l1j-client", 640, 480);
    let mut windowb = vid_win.position_centered();

    let windowed = match settings.get("general", "window").unwrap().as_str() {
        "yes" => true,
        _ => false,
    };

    if !windowed {
        windowb = windowb.fullscreen();
    }
    let window = windowb.opengl().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let i = sdl2::mixer::InitFlag::MP3;
    let _sdl2mixer = sdl2::mixer::init(i).unwrap();
    let audio = sdl2::mixer::open_audio(44100, 16, 2, 1024);
    println!("Audio is {:?}", audio);

    let mut game_resources = GameResources::new(font);
    let mut mode: Box<dyn GameMode> = match mode {
        DrawMode::Explorer => {
            let t = ExplorerMenu::new(&texture_creator, &mut game_resources);
            Box::new(t)
        }
        DrawMode::PngExplorer => Box::new(PngExplorer::new(&texture_creator, &mut game_resources)),
        DrawMode::ImgExplorer => Box::new(ImgExplorer::new(&texture_creator, &mut game_resources)),
        DrawMode::SprExplorer => Box::new(SprExplorer::new(&texture_creator, &mut game_resources)),
        DrawMode::GameLoader => Box::new(GameLoader::new(&texture_creator, &mut game_resources)),
        DrawMode::Login => Box::new(Login::new(&texture_creator)),
        DrawMode::CharacterSelect => {
            Box::new(CharacterSelect::new(&texture_creator, &mut game_resources))
        }
        DrawMode::NewCharacter => {
            Box::new(NewCharacterMode::new(&texture_creator, &mut game_resources))
        }
        DrawMode::Game => Box::new(Game::new(&texture_creator, &mut game_resources)),
        DrawMode::WavPlayer => Box::new(WavPlayer::new(&texture_creator, &mut game_resources)),
    };

    let windowed = match settings.get("general", "window").unwrap().as_str() {
        "yes" => true,
        _ => false,
    };

    let resources = settings.get("general", "resources").unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (mut s1, r1) = tokio::sync::mpsc::channel(100);
    let (s2, mut r2) = tokio::sync::mpsc::channel(100);
    rt.spawn(async_main(r1, s2));

    println!("Loading resources from {}", resources);

    let _e = s1.blocking_send(MessageToAsync::LoadResources(resources.clone()));
    let _e = s1.blocking_send(MessageToAsync::LoadTable("obscene-e.tbl".to_string()));
    let _e = s1.blocking_send(MessageToAsync::LoadFont("Font/eng.fnt".to_string()));
    let _e = s1.blocking_send(MessageToAsync::LoadSpriteTable);
    //load Font/SMALL.FNT

    //load pack files
    //Text, Tile, Sprite0-Sprite15

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let dummy_texture = make_dummy_texture(&texture_creator);

    let flags = sdl2::image::InitFlag::all();
    let _sdl2_image = sdl2::image::init(flags).unwrap();

    let mut mouse = Mouse::new();
    let mut drawmode_commands: VecDeque<DrawModeRequest> = VecDeque::new();

    'running: loop {
        while let Ok(msg) = r2.try_recv() {
            match &msg {
                MessageFromAsync::ResourceStatus(b) => {
                    if !b {
                        println!("Failed to load game resources");
                        let scheme = sdl2::messagebox::MessageBoxColorScheme {
                            background: (0, 0, 255),
                            text: (0, 0, 0),
                            button_border: (0, 0, 0),
                            button_background: (255, 255, 255),
                            button_selected: (0, 255, 255),
                        };
                        let mut vid_win = video_subsystem.window("l1j-client", 320, 240);
                        let mut windowb = vid_win.position_centered();
                        if !windowed {
                            windowb = windowb.fullscreen();
                        }
                        let window = windowb.build().unwrap();
                        let _e = sdl2::messagebox::show_message_box(
                            sdl2::messagebox::MessageBoxFlag::ERROR,
                            &[sdl2::messagebox::ButtonData {
                                flags: sdl2::messagebox::MessageBoxButtonFlag::RETURNKEY_DEFAULT,
                                button_id: 0,
                                text: "OK",
                            }],
                            "ERROR",
                            "Unable to load game resources",
                            &window,
                            scheme,
                        );
                        break 'running;
                    }
                }
                MessageFromAsync::StringTable(name, _data) => {
                    println!("Loaded string table {}", name);
                }
                MessageFromAsync::Png(name, data) => {
                    let png = texture_creator.load_texture_bytes(data);
                    match png {
                        Ok(mut a) => {
                            a.set_blend_mode(sdl2::render::BlendMode::Add);
                            game_resources.pngs.insert(*name, Loaded(a));
                            println!("PNG {} success", name);
                        }
                        Err(e) => {
                            println!("PNG {} fail {}", name, e);
                            println!("PNG DATA {:x?}", &data[0..25]);
                        }
                    }
                }
                MessageFromAsync::Img(name, data) => {
                    println!("Loaded IMG {}", name);
                    let mut data = (*data).clone();
                    let img = data.convert_img_data(&texture_creator);
                    match img {
                        Some(a) => {
                            game_resources.imgs.insert(*name, Loaded(a));
                            println!("IMG{} success", name);
                        }
                        None => {
                            println!("IMG {} fail", name);
                        }
                    }
                }
                MessageFromAsync::Sprite(id, spr) => {
                    let sprite = spr.to_gui(&texture_creator);
                    game_resources.sprites.insert(*id, Loaded(sprite));
                }
                MessageFromAsync::Sfx(id, data) => {
                    let rwops = sdl2::rwops::RWops::from_bytes(&data[..]).unwrap();
                    let chnk = rwops.load_wav().unwrap();
                    game_resources.sfx.insert(*id, Loaded(chnk));
                }
            }
        }
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
                } => {
                    mouse.event(MouseEventInput::Scrolling(y));
                }
                Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    println!("Key down event");
                    if let Some(key) = keycode {
                        mode.process_button(key, true, &mut game_resources);
                    }
                }
                Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    println!("Key up event");
                    if let Some(key) = keycode {
                        mode.process_button(key, false, &mut game_resources);
                    }
                }
                _ => {}
            }
        }
        mouse.parse();
        mode.process_mouse(mouse.events(), &mut drawmode_commands);
        mouse.clear();
        mode.process_frame(&mut game_resources, &mut s1, &mut drawmode_commands);
        while let Some(m) = drawmode_commands.pop_front() {
            match m {
                DrawModeRequest::ChangeDrawMode(m) => {
                    println!("Requested to change the drawmode");
                    mode = match m {
                        DrawMode::Explorer => {
                            let t = ExplorerMenu::new(&texture_creator, &mut game_resources);
                            Box::new(t)
                        }
                        DrawMode::PngExplorer => {
                            Box::new(PngExplorer::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::ImgExplorer => {
                            Box::new(ImgExplorer::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::SprExplorer => {
                            Box::new(SprExplorer::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::GameLoader => {
                            Box::new(GameLoader::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::Login => Box::new(Login::new(&texture_creator)),
                        DrawMode::CharacterSelect => {
                            Box::new(CharacterSelect::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::NewCharacter => {
                            Box::new(NewCharacterMode::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::Game => {
                            Box::new(Game::new(&texture_creator, &mut game_resources))
                        }
                        DrawMode::WavPlayer => {
                            Box::new(WavPlayer::new(&texture_creator, &mut game_resources))
                        }
                    };
                }
            }
        }
        mode.draw(&mut canvas, mouse.cursor(), &mut game_resources, &mut s1);
        let _e = canvas.copy(&dummy_texture, None, None);
        canvas.present();
        let framerate = mode.framerate() as u64;
        ::std::thread::sleep(Duration::from_nanos(1_000_000_000u64 / framerate));
    }
}
