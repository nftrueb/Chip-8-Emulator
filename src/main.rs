mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_HEIGHT, SCREEN_WIDTH, Chip8Input, get_chip8_key_idx}; 

mod video;
pub use video::video::{
    write_text,
    draw_entire_window, 
    draw_register_region, 
    draw_pc_region, 
    draw_i_region, 
    CANVAS_WIDTH, 
    CANVAS_HEIGHT,
    DEBUG_CANVAS_WIDTH, 
    DEBUG_CANVAS_HEIGHT
};

mod audio; 
pub use audio::audio::SquareWave;

extern crate sdl2;

use regex::Regex;
use sdl2::audio::AudioSpecDesired; 
use sdl2::event::Event;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::render::Canvas;
use sdl2::video::Window; 
use std::collections::HashMap;
use std::collections::HashSet;
use std::env; 
use std::fs;
use std::fs::File;  
use std::io::Read; 
use std::path::Path; 
use std::process; 
use std::time::Instant;
 
const OSA_SCRIPTS_PATH: &str = "/Users/nicktrueb/.osascripts";
const FONT_PATH: &str = "/Users/nicktrueb/Programming/chip8/assets/FragmentMono-Regular.ttf";

#[derive(Eq, PartialEq, Hash, Debug)]
enum OptionalModes { 
    Debug, 
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

fn execute(mut cpu: CPU, modes: &mut HashSet<OptionalModes>) -> Result<(), String> { 

    // initialize contexts 
    let sdl_context = 
        sdl2::init()
        .expect("ERROR:: failed to initialize SDL context");
    let video_subsystem = 
        sdl_context.video()
            .expect("ERROR:: failed to initialize video subsystem");
    let audio_subsystem = 
        sdl_context.audio()
            .expect("ERROR:: failed to initialize audio subsystem");
    let ttf_context = 
        sdl2::ttf::init()
            .expect("ERROR:: failed to load ttf context"); 

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

    // initialize desired audio spec and open actual audio spec
    let desired_spec = AudioSpecDesired { 
        freq: Some(44100), 
        channels: Some(1), 
        samples: None
    };

    let audio_device = 
        audio_subsystem.open_playback(
            None, 
            &desired_spec, 
            |spec| { 
                let phase_inc = 261.0 / spec.freq as f32; 
                SquareWave { 
                    phase: 0.0, 
                    phase_inc,
                    amplitude: 0.1,
                }
            }
        ).expect("ERROR:: failed to initialize sound struct"); 

    // load font 
    let point_size = 18; 
    let font = ttf_context.load_font(Path::new(FONT_PATH), point_size)
        .expect("ERROR: failed to load external font"); 

    // reset canvas and update window
    let mut paused_state = true; 
    draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), paused_state); 

    let mut event_pump = sdl_context.event_pump()?;
    let mut manual_step_signal: bool = false; 
    let mut prev_cpu_cycle_time: Instant = Instant::now(); 
    let mut prev_screen_refresh_time: Instant = Instant::now();
    let mut timer_refresh_time: Instant = Instant::now(); 
    let cpu_cycle_freq:u128 = 60;  
    let screen_refresh_freq: u128 = 60; 
    let timer_update_freq: u128 = 60; 
    let keyboard_to_chip8_input_map: HashMap<SdlKeycode, Chip8Input> = build_keycode_hashmap(); 

    // enter main game loop 
    'running: loop {

        // event handling
        for event in event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'running, 
                Event::KeyUp {
                    keycode: Some(key),
                    ..
                } => { 
                    match key {
                        SdlKeycode::Escape => {
                            cpu.reset(); 
                            paused_state = true;
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), paused_state); 
                        }, 
                        SdlKeycode::Space => {
                            paused_state = !paused_state; 
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), paused_state);
                        },
                        SdlKeycode::RShift => {
                            if paused_state { manual_step_signal = true; }
                        },
                        SdlKeycode::LShift => { 

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
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), paused_state); 
                        } 
                        _ => {}, 
                    }
                },
                _ => {}
            }
        }

        // update cpu timers
        if timer_refresh_time.elapsed().as_millis() >= 1_000 / timer_update_freq { 
            timer_refresh_time = Instant::now(); 
            
            if paused_state { 
                audio_device.pause(); 
            } else {
                cpu.update_timers();
                if cpu.sound_timer > 0 { 
                    audio_device.resume();
                } else { 
                    audio_device.pause();
                }
            }
        }

        // process cpu instructions at specified cycle frequency
        if prev_cpu_cycle_time.elapsed().as_millis() >= 1_000 / cpu_cycle_freq {
            prev_cpu_cycle_time = Instant::now(); 

            // do not update if game is paused and manual step button has not been pressed
            if !paused_state || (paused_state && manual_step_signal) {
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

                cpu.step(pressed_keys);
            }
        }

        // draw pixels on canvas at specified screen refresh frequency
        if prev_screen_refresh_time.elapsed().as_millis() >= 1_000 / screen_refresh_freq { 
            prev_screen_refresh_time = Instant::now(); 
            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), paused_state); 
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
        println!("file not loaded from file chooser dialog ... exiting now"); 
        std::process::exit(0); 
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

   KEY COMMANDS (while program is running): 
     ESCAPE  : reset ROM and set CPU to PAUSED
     L_SHIFT : toggle debugging interface in window
     RETURN  : manually step through CPU
     SPACE   : toggle CPU state between PAUSED and RUNNING
"
    ); 
}

pub fn main() {

    // parse input file name
    let (filename_option, mut modes)= parse_command_line_args(); 

    // get file from dialog if filename was not parse in arguments
    let filename: String = match filename_option { 
        Some(value) => value, 
        None => get_file_from_file_chooser_dialog()
    };

    // match file to correct extension
    if !Regex::new("\\.ch8$").unwrap().is_match(&filename) { 
        println!("ERROR:: file does not have correct extension"); 
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
    execute(cpu, &mut modes).unwrap();
}