mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_WIDTH, SCREEN_HEIGHT, Chip8Input, get_chip8_key_idx}; 

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window; 
use sdl2::ttf::Font;
use regex::Regex; 
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
use std::env; 
use std::fs;
use std::fs::File;  
use std::io::Read; 
use std::process; 
use std::path::Path; 
 
const PIXEL_WIDTH  : usize = 10; 
const CANVAS_WIDTH : usize = SCREEN_WIDTH * PIXEL_WIDTH; 
const CANVAS_HEIGHT: usize = SCREEN_HEIGHT * PIXEL_WIDTH; 
const OSA_SCRIPTS_PATH: &str = "/Users/nicktrueb/.osascripts";
const BACKGROUND_COLOR: Color = Color::RGB(50, 50, 150); 
const DEBUG_CANVAS_WIDTH: usize = CANVAS_WIDTH * 2; 
const DEBUG_CANVAS_HEIGHT: usize = CANVAS_HEIGHT * 2; 
const REGION_WIDTH      : i32 = CANVAS_WIDTH as i32; 
const REGION_HEIGHT     : i32 = CANVAS_HEIGHT as i32; 
// const REGION_ROM_X      : i32 = 0; 
// const REGION_ROM_Y      : i32 = 0; 
const REGION_REGISTER_X : i32 = REGION_WIDTH; 
const REGION_REGISTER_Y : i32 = 0;
const REGION_PC_X       : i32 = 0; 
const REGION_PC_Y       : i32 = REGION_HEIGHT; 
const REGION_I_X        : i32 = REGION_WIDTH; 
const REGION_I_Y        : i32 = REGION_HEIGHT;  

#[derive(PartialEq, Debug)]
enum GameState { 
    Running, 
    Paused,
}

#[derive(Eq, PartialEq, Hash, Debug)]
enum OptionalModes { 
    Debug, 
    ManualStepping
}

// maps Keycode value (from sdl2) to Chip8Input value (0-F) to simulate controller input 
fn build_keycode_hashmap() -> HashMap<SdlKeycode, Chip8Input>{ 
    HashMap::from([
        // ROW 1
        (SdlKeycode::Num1, Chip8Input::Num1), 
        (SdlKeycode::Num2, Chip8Input::Num2), 
        (SdlKeycode::Num3, Chip8Input::Num3), 
        (SdlKeycode::Num4, Chip8Input::C),
    
        // ROW 2
        (SdlKeycode::Q, Chip8Input::Num4), 
        (SdlKeycode::W, Chip8Input::Num5), 
        (SdlKeycode::E, Chip8Input::Num6), 
        (SdlKeycode::R, Chip8Input::D), 
    
        // ROW 3
        (SdlKeycode::A, Chip8Input::Num7), 
        (SdlKeycode::S, Chip8Input::Num8), 
        (SdlKeycode::D, Chip8Input::Num9), 
        (SdlKeycode::F, Chip8Input::E),
    
        // ROW 4 
        (SdlKeycode::Z, Chip8Input::A), 
        (SdlKeycode::X, Chip8Input::Num0), 
        (SdlKeycode::C, Chip8Input::B), 
        (SdlKeycode::V, Chip8Input::F),
    ])
}

fn draw_rom_region(canvas: &mut Canvas<Window>, pixels: &[bool; 2048]) { 
    for (idx, pixel) in pixels.into_iter().enumerate() { 
        let row: usize = idx / SCREEN_WIDTH; 
        let col: usize = idx % SCREEN_WIDTH;
        let rect = Rect::new(
            (col * PIXEL_WIDTH) as i32, 
            (row * PIXEL_WIDTH) as i32, 
            PIXEL_WIDTH as u32, 
            PIXEL_WIDTH as u32
        );
        let color: Color = if *pixel { 
            Color::WHITE
        } else { 
            Color::BLACK
        };
        
        canvas.set_draw_color(color);
        canvas.fill_rect(rect).expect("rect not filled correctly when drawing screen!!");
        // println!("{:?}", rect); 
    }
}

fn draw_register_region(canvas: &mut Canvas<Window>, font: &Font) { 
    // canvas.set_draw_color(Color::BLACK);
    // canvas.draw_rect(Rect::new(REGION_REGISTER_X, REGION_REGISTER_Y, REGION_WIDTH as u32, REGION_HEIGHT as u32)); 
    // text surface 
    let text_surface = 
        font.render("--- REGISTERS ---")
            .blended(Color::WHITE)
            .expect("ERROR:: failed to render text"); 

    // text texture 
    let texture_creator = canvas.texture_creator(); 
    let text_texture = 
        texture_creator.create_texture_from_surface(&text_surface)
            .expect("ERROR: failed to create text texture"); 
         
    // copy texture 
    let title_x = REGION_REGISTER_X + ((REGION_WIDTH / 2) as i32) - ((text_surface.width() / 2) as i32) ;
    canvas.copy(
        &text_texture, 
        None, 
        Rect::new(title_x, REGION_REGISTER_Y, text_surface.width(), text_surface.height()))
        .expect("ERROR:: failed to copy text to canvas");
}

fn draw_pc_region(canvas: &mut Canvas<Window>, font: &Font) { 
    let text_surface = 
        font.render("--- PC POINTER ---")
            .blended(Color::WHITE)
            .expect("ERROR:: failed to render text"); 

    // text texture 
    let texture_creator = canvas.texture_creator(); 
    let text_texture = 
        texture_creator.create_texture_from_surface(&text_surface)
            .expect("ERROR: failed to create text texture"); 
        
    // copy texture 
    let title_x = REGION_PC_X + ((REGION_WIDTH / 2) as i32) - ((text_surface.width() / 2) as i32) ;
    canvas.copy(
        &text_texture, 
        None, 
        Rect::new(title_x, REGION_PC_Y, text_surface.width(), text_surface.height()))
        .expect("ERROR:: failed to copy text to canvas");
} 

fn draw_i_region(canvas: &mut Canvas<Window>, font: &Font) { 
    let text_surface = 
        font.render("--- I REGISTER POINTER ---")
            .blended(Color::WHITE)
            .expect("ERROR:: failed to render text"); 

    // text texture 
    let texture_creator = canvas.texture_creator(); 
    let text_texture = 
        texture_creator.create_texture_from_surface(&text_surface)
            .expect("ERROR: failed to create text texture"); 
        
    // copy texture 
    let title_x = REGION_I_X + ((REGION_WIDTH / 2) as i32) - ((text_surface.width() / 2) as i32) ;
    canvas.copy(
        &text_texture, 
        None, 
        Rect::new(title_x, REGION_I_Y, text_surface.width(), text_surface.height()))
        .expect("ERROR:: failed to copy text to canvas");
}

fn execute(mut cpu: CPU, mut modes: HashSet<OptionalModes>) -> Result<(), String> { 

    // initialize contexts 
    let sdl_context = 
        sdl2::init()
        .expect("ERROR:: failed to initialize SDL context");
    let video_subsystem = 
        sdl_context.video()
            .expect("ERROR: failed to load video subsystem");
    let ttf_context = 
        sdl2::ttf::init()
            .expect("ERROR: failed to load ttf context"); 

    // get initial window instance
    let (width, height) = match modes.contains(&OptionalModes::Debug) { 
        true => { (DEBUG_CANVAS_WIDTH as u32, DEBUG_CANVAS_HEIGHT as u32) },
        false => { (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32) }
    }; 
    let init_window: Window = video_subsystem
        .window("CHIP-8", width, height)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?; 

    // get canvas instance
    let mut canvas: Canvas<Window> = init_window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // reset canvas and update window
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    draw_rom_region(&mut canvas, &cpu.pixels); 
    canvas.present(); 

    println!("{:?}", canvas.window().size()); 

    // load font 
    let point_size = 18; 
    let font = ttf_context.load_font(
        Path::new("/Users/nicktrueb/Programming/chip8/assets/FragmentMono-Regular.ttf"), 
        point_size
    ).expect("ERROR: failed to load external font"); 

    let mut event_pump = sdl_context.event_pump()?;
    let mut state: GameState = GameState::Paused;
    let mut should_render_screen: bool = true; 
    let mut manual_step_signal: bool = false; 
    let mut prev_cpu_cycle_time: Instant = Instant::now(); 
    let mut prev_screen_refresh_time: Instant = Instant::now();
    let cpu_cycle_freq: u128 = 500;  
    let screen_refresh_freq: u128 = 60; 
    let keyboard_to_chip8_input_map: HashMap<SdlKeycode, Chip8Input> = build_keycode_hashmap(); 

    // enter main game loop 
    'running: loop {

        // event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running, 
                Event::KeyUp {
                    keycode: Some(key),
                    ..
                } => { 
                    match key {
                        SdlKeycode::Escape => {
                            cpu.reset(); 
                            state = GameState::Paused;
                            canvas.set_draw_color(BACKGROUND_COLOR); 
                            canvas.clear(); 
                            draw_rom_region(&mut canvas, &cpu.pixels); 
                            canvas.present();
                        }, 
                        SdlKeycode::Space => {
                            state = match state {  
                                GameState::Paused  => GameState::Running, 
                                GameState::Running => GameState::Paused
                            }; 
                        },
                        SdlKeycode::RShift => {
                            if modes.contains(&OptionalModes::ManualStepping) {
                                manual_step_signal = true; 
                            }
                        },
                        SdlKeycode::D => { 

                            // toggle 'Debug' in modes hashset 
                            if modes.contains(&OptionalModes::Debug) { 
                                modes.remove(&OptionalModes::Debug); 
                            } else { 
                                modes.insert(OptionalModes::Debug); 
                            }

                            // get dimensions of new window based on 'Debug' status
                            let (width, height) = match modes.contains(&OptionalModes::Debug) { 
                                true => { (DEBUG_CANVAS_WIDTH as u32, DEBUG_CANVAS_HEIGHT as u32) },
                                false => { (CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32) }
                            }; 

                            // set window dimensions and redraw canvas
                            canvas
                                .window_mut()
                                .set_size(width, height)
                                .expect("Failed to resize window"); 
                            canvas.set_draw_color(BACKGROUND_COLOR);
                            canvas.clear();
                            draw_rom_region(&mut canvas, &cpu.pixels); 
                            canvas.present(); 
                        } 
                        _ => {}, 
                    }
                },
                _ => {}
            }
        }

        // do not step with cpu if game is paused
        if state == GameState::Paused { continue; }

        // process cpu instructions at specified cycle frequency
        if prev_cpu_cycle_time.elapsed().as_millis() >= 1_000 / cpu_cycle_freq { 
            prev_cpu_cycle_time = Instant::now(); 

            // do not update if manual stepping is enabled and signal has not been sent
            if !modes.contains(&OptionalModes::ManualStepping) || 
                (modes.contains(&OptionalModes::ManualStepping) && manual_step_signal) {
                manual_step_signal = false; 

                // get vector of pressed keys to be matched in cpu
                let pressed_keys: Vec<usize> = 
                    event_pump.keyboard_state()
                        .pressed_scancodes()
                        .filter_map(SdlKeycode::from_scancode)
                        .map(|keycode| keyboard_to_chip8_input_map.get(&keycode))
                        .filter(|keycode| *keycode != None)
                        .map(|keycode| keycode.unwrap())
                        .map(|key| get_chip8_key_idx(key))
                        .collect(); 

                let needs_rendering = cpu.step(pressed_keys);
                if !should_render_screen { should_render_screen = needs_rendering };
            }
        }

        // draw pixels on canvas at specified screen refresh frequency
        if prev_screen_refresh_time.elapsed().as_millis() >= 1_000 / screen_refresh_freq { 
            prev_screen_refresh_time = Instant::now(); 

            canvas.set_draw_color(BACKGROUND_COLOR); 
            canvas.clear(); 
            
            if modes.contains(&OptionalModes::Debug) { 
                draw_register_region(&mut canvas, &font);
                draw_pc_region(&mut canvas, &font); 
                draw_i_region(&mut canvas, &font);

                // region debug lines
                canvas.set_draw_color(Color::RED); 
                canvas.draw_line(Point::new(CANVAS_WIDTH as i32, 0), Point::new(CANVAS_WIDTH as i32, DEBUG_CANVAS_HEIGHT as i32)); 
                canvas.draw_line(Point::new(0, CANVAS_HEIGHT as i32), Point::new(DEBUG_CANVAS_WIDTH as i32, CANVAS_HEIGHT as i32)); 
            }
            draw_rom_region(&mut canvas, &cpu.pixels); 
            canvas.present();
        }
    }

    Ok(())
}

fn get_file_from_file_chooser_dialog() -> String { 
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(format!("osascript {}/file-dialog.scpt", OSA_SCRIPTS_PATH))
        .output()
        .expect("ERROR: failed to get output from file chooser dialog command"); 

    let output_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string(); 

    let err_str = String::from_utf8_lossy(&output.stderr)
        .trim()
        .to_string(); 

    if err_str.len() != 0 { 
        println!("ERROR: failed to get file from file chooser dialog:\n{}", err_str); 
        std::process::exit(1); 
    }

    output_str
}

fn parse_command_line_args() -> (Option<String>, HashSet<OptionalModes>) { 

    let argv: Vec<String> = env::args().collect(); 
    let mut filename: Option<String> = None; 
    let mut optional_modes: HashSet<OptionalModes> = HashSet::new(); 
    let ch8_re_pattern = Regex::new("\\.ch8$").expect("ERROR: regex was not created successfully");

    // loop through args and mark flags / parse filename
    for (idx, value) in argv.iter().enumerate() { 
        if idx == 0 { continue; }
        match value.as_str() { 
            "-d" | "--debug" => optional_modes.insert(OptionalModes::Debug), 
            "-h" | "--help" => { 
                print_usage(); 
                std::process::exit(0); 
            }, 
            "-s" | "--step" => optional_modes.insert(OptionalModes::ManualStepping), 
            _ if ch8_re_pattern.is_match(value.as_str()) => {
                filename = Some(value.to_owned()); 
                true
            }, 
            _ => panic!("ERROR: encountered unknown value: {}", value.as_str())
        }; 
    }

    // return filename and parsed modes
    (filename, optional_modes)
}

fn print_usage() { 
    print!(
"  USAGE:: ./chip8 [-d | -s | {{filename}}.ch8]

   DESCRIPTION: This is a Chip-8 interpreter built in rust 

   OPTIONS: 
     -d | --debug -> turns on debugging information about current instructions and memory
     -h | --help  -> print usage and return
     -s | --step  -> turns on manual stepping

   KEY COMMANDS (while program is running): 
     ESCAPE  : reset ROM and set CPU to PAUSED
     D       : toggle debugging interface in window
     RETURN  : manually step through CPU
     SPACE   : toggle CPU state between PAUSED and RUNNING
"
    ); 
}

pub fn main() {

    // parse input file name
    let (filename_option, modes)= parse_command_line_args(); 

    // get file from dialog if filename was not parse in arguments
    let filename: String = match filename_option { 
        Some(value) => value, 
        None => get_file_from_file_chooser_dialog()
    };

    if !Regex::new("\\.ch8$").unwrap().is_match(&filename) { 
        println!("ERROR: invalid filename: {}", filename); 
        std::process::exit(1); 
    }

    // open file 
    let file: File = match fs::File::open(&filename) { 
        Ok(f) => f, 
        Err(_) => panic!("Failed to open file: {filename}")
    };

    // read bytes into byte vector
    let mut rom_bytes = Vec::new(); 
    for byte in file.bytes() { 
        rom_bytes.push(byte.unwrap()); 
    }

    // instantiate cpu and load rom bytes into memory
    let mut cpu: CPU = CPU::new();  
    cpu.load_rom(rom_bytes);

    if modes.contains(&OptionalModes::Debug) { 
        println!("{}", cpu.dump_memory()); 
        println!("Starting CPU with the following modes: {:?}", modes); 
    }  

    // run cpu
    let _result = execute(cpu, modes); 
    // match result { 
    //     Ok(()) => {}, 
    //     Err(_msg) => {}
    // }

    // handle restarting cpu, loading new rom, etc
}