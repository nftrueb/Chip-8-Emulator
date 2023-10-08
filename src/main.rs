
mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_WIDTH, SCREEN_HEIGHT, Chip8Input, get_chip8_key_idx}; 

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window; 

use regex::Regex; 

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::time::Instant;
use std::env; 
use std::fs;
use std::fs::File;  
use std::io::Read; 
use std::process; 
use std::rc::Rc; 
use std::cell::RefCell; 
 
const PIXEL_WIDTH  : usize = 10; 
const CANVAS_WIDTH : usize = SCREEN_WIDTH * PIXEL_WIDTH; 
const CANVAS_HEIGHT: usize = SCREEN_HEIGHT * PIXEL_WIDTH; 
const OSA_SCRIPTS_PATH: &str = "/Users/nicktrueb/.osascripts";

#[derive(PartialEq, Debug)]
enum GameState { 
    Running, 
    Paused,
}

#[derive(Eq, PartialEq, Hash, Debug)]
enum OptionalModes { 
    DebugMessages, 
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

fn execute(mut cpu: CPU, modes: HashSet<OptionalModes>) -> Result<(), String> { 

    // get os specific window info
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // get initial window instance
    let init_window: Window = video_subsystem
        .window("CHIP-8", CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32)
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
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

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
                            state = GameState::Paused;
                            cpu.reset(); 
                            canvas.set_draw_color(Color::BLACK);
                            canvas.clear();
                            canvas.present(); 
                        }, 
                        SdlKeycode::Space => {
                            state = match state {  
                                GameState::Paused  => GameState::Running, 
                                GameState::Running => GameState::Paused
                            }; 
                        },
                        SdlKeycode::Return => {
                            if modes.contains(&OptionalModes::ManualStepping) {
                                manual_step_signal = true; 
                            }
                        },
                        SdlKeycode::LShift => { 
                            let (x, y) = canvas.window().size();
                            canvas
                                .window_mut()
                                .set_size((x+64) as u32, y as u32)
                                .expect("Failed to resize window");
                            println!("{} {}", x, y); 
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
                let pressed_keys: Vec<usize> = event_pump.keyboard_state()
                                                            .pressed_scancodes()
                                                            .filter_map(SdlKeycode::from_scancode)
                                                            .map(|keycode| keyboard_to_chip8_input_map.get(&keycode))
                                                            .filter(|keycode| *keycode != None)
                                                            .map(|keycode| keycode.unwrap())
                                                            .map(|key| get_chip8_key_idx(key))
                                                            .collect(); 
                let (needs_rendering, pc, instruction_string) = cpu.step(pressed_keys);
                if !should_render_screen { should_render_screen = needs_rendering };

                // print debug info after executing instruction
                if modes.contains(&OptionalModes::DebugMessages) { 
                    println!("PC: {}  |  Instruction: {}", pc, instruction_string);
                    println!("I: {:#04x}", cpu.reg_i); 
                    for (idx, reg) in cpu.registers.iter().enumerate() { 
                        if idx % 4 == 0 && idx != 0 { print!("\n"); }
                        print!("{:#04x} ", reg); 
                    }
                    print!("\n");
                }  
            }
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
            "-d" | "--debug" => optional_modes.insert(OptionalModes::DebugMessages), 
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
"
    ); 
}

pub fn main() {

    // parse input file name
    let (filename_option, modes)= parse_command_line_args(); 

    // get file from dialog if filename was not parse in arguments
    let filename = match filename_option { 
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

    if modes.contains(&OptionalModes::DebugMessages) { 
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