
mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_WIDTH, SCREEN_HEIGHT, Chip8Input, get_chip8_key_idx}; 

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::libc::EAI_MEMORY;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window; 

use std::collections::HashMap;
use std::time::Instant;
use std::env; 
use std::fs;
use std::fs::File;  
use std::io::Read; 
 
const PIXEL_WIDTH  : usize = 10; 
const CANVAS_WIDTH : usize = SCREEN_WIDTH * PIXEL_WIDTH; 
const CANVAS_HEIGHT: usize = SCREEN_HEIGHT * PIXEL_WIDTH; 

#[derive(PartialEq)]
enum GameState { 
    Running, 
    Paused,
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

fn execute(mut cpu: CPU) -> Result<(), String> { 

    // get os specific window info
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // get window instance
    let window: Window = video_subsystem
        .window("CHIP-8", CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // get canvas instance
    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // reset canvas and update window
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let mut state: GameState = GameState::Paused;
    let mut should_render_screen: bool = true; 
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
                            state = GameState::Paused ;
                            cpu.reset(); 
                        }, 
                        SdlKeycode::Space => {
                            state = match state {  
                                GameState::Paused  => GameState::Running, 
                                GameState::Running => GameState::Paused
                            }
                        },
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

            // get vector of pressed keys to be matched in cpu
            let pressed_keys: Vec<usize> = event_pump.keyboard_state()
                                                        .pressed_scancodes()
                                                        .filter_map(SdlKeycode::from_scancode)
                                                        .map(|keycode| keyboard_to_chip8_input_map.get(&keycode))
                                                        .filter(|keycode| *keycode != None)
                                                        .map(|keycode| keycode.unwrap())
                                                        .map(|key| get_chip8_key_idx(key))
                                                        .collect(); 
            should_render_screen = if should_render_screen { 
                true 
            } else { 
                let (render_bool, instruction_string) = cpu.step(pressed_keys); 
                render_bool
            };
        }

        // draw pixels on canvas at specified screen refresh frequency
        if prev_screen_refresh_time.elapsed().as_millis() >= 1_000 / screen_refresh_freq { 
            prev_screen_refresh_time = Instant::now(); 
            if !should_render_screen { continue; }

            should_render_screen = false; 
            canvas.clear(); 
            for (idx, pixel) in (&cpu.pixels).into_iter().enumerate() { 
                let row: usize = idx / SCREEN_WIDTH; 
                let col: usize = idx % SCREEN_WIDTH;
                let rect = Rect::new((col * PIXEL_WIDTH) as i32 , (row * PIXEL_WIDTH) as i32, PIXEL_WIDTH as u32, PIXEL_WIDTH as u32);
                let color: Color = if *pixel { 
                    Color::WHITE
                } else { 
                    Color::BLACK
                };
                
                canvas.set_draw_color(color);
                canvas.fill_rect(rect).expect("rect not filled correctly when drawing screen!!");
            }
            canvas.present();
        }
    }

    Ok(())
}

fn parse_command_line_args() -> Result<String, String> { 
    let argv: Vec<String> = env::args().collect(); 
    let argc: usize = argv.len(); 
    if argc != 2 { 
        return Err(String::from("Arguments of incorrect length provided"));
    }

    let filename: &String = &argv[1]; 
    if filename.len() < 4 { 
        return Err(String::from("Argument too short to contain .ch8")); 
    }
    if &filename[filename.len()-4..filename.len()] != ".ch8" { 
        return Err(String::from("Argument does not contain .ch8 extension")); 
    }

    Ok(filename.to_owned())
}

fn print_usage(error_message: String) { 
    println!("ERROR: {}", error_message); 
    print!(
"USAGE:: ./chip8 {{filename}}.ch8

   DESCRIPTION: This is a Chip-8 interpreter built in rust 

   FLAGS: 
     -d -> turns on debugging information about current instructions and memory
"
    ); 
}

pub fn main() {

    // parse input file name
    let filename = match parse_command_line_args() {
        Ok(value) => value, 
        Err(message) => { 
            print_usage(message); 
            std::process::exit(1); 
        }
    };

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

    let memory_dump: String = cpu.dump_rom();
    println!("{}", memory_dump);   

    std::process::exit(1); 

    // run cpu
    let result = execute(cpu); 
    match result { 
        Ok(()) => {}, 
        Err(_msg) => {}
    }

    // handle restarting cpu, loading new rom, etc
}