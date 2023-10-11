mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_WIDTH, SCREEN_HEIGHT, Chip8Input, get_chip8_key_idx}; 

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::pixels::Color;
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
const REGION_ROM_X      : i32 = 0; 
const REGION_ROM_Y      : i32 = 0; 
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

fn write_text(text: String, x_off: i32, y_off: i32, color: Color, font: &Font, canvas: &mut Canvas<Window>) { 
    // text surface 
    let surface = 
        font.render(&text)
            .blended(color)
            .expect("ERROR:: failed to render text"); 

    // text texture 
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .expect("ERROR: failed to create text texture");
    let (width, height) = surface.size(); 

    // copy text to canvas
    let x = x_off - ((width/ 2) as i32);
    canvas.copy(&texture, None, Rect::new(x, y_off, width, height))
        .expect("ERROR:: failed to copy text to canvas");
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

fn draw_register_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
    let row_height = 2*font.height(); 
    let columns = 4; 
    let title_row = 0; 
    let v_register_row = 1; 
    let i_register_row = 5; 
    let pc_row = 5;

    // write title for this region
    write_text("--- REGISTERS ---".to_string(), 
        REGION_REGISTER_X + (REGION_WIDTH / 2), 
        REGION_REGISTER_Y + row_height * title_row, 
        Color::WHITE, 
        font, 
        canvas);

    // write grid of registers and their values
    for (idx, value) in cpu.registers.iter().enumerate() { 
        let x_off = REGION_REGISTER_X + (REGION_WIDTH / (columns + 1)) * ((idx as i32 % columns) + 1); 
        let y_off = REGION_REGISTER_Y + (row_height * v_register_row) + (row_height * (idx as i32 / columns));
        write_text(
            format!("V{:X}: {:#04x}", idx, value), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    // write I register value
    {
        let x_off = REGION_REGISTER_X + REGION_WIDTH / (columns + 1); 
        let y_off = REGION_REGISTER_Y + row_height * i_register_row; 
        write_text(
            format!("I : x{:03x}", cpu.reg_i), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    // write PC value
    {
        let x_off = REGION_REGISTER_X + REGION_WIDTH / (columns + 1) * 2; 
        let y_off = REGION_REGISTER_Y + row_height * pc_row; 
        write_text(
            format!("PC: x{:03x}", cpu.pc), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }
}

fn draw_pc_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
    let row_height = 2*font.height(); 
    let columns = 4; 
    let title_row = 0; 
    let memory_row = 1; 

    // write title for this region
    write_text("--- PC POINTER ---".to_string(), 
        REGION_PC_X + (REGION_WIDTH / 2), 
        REGION_PC_Y + row_height * title_row, 
        Color::WHITE, 
        font, 
        canvas);

    // address and memory header
    { 
        let x_off = REGION_PC_X + (REGION_WIDTH / (columns + 1)) * 2; 
        let y_off = REGION_PC_Y + (row_height * memory_row);
        write_text(
            "ADDR".to_string(), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    { 
        let x_off = REGION_PC_X + (REGION_WIDTH / (columns + 1)) * 3; 
        let y_off = REGION_PC_Y + (row_height * memory_row);
        write_text(
            "MEM_".to_string(), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    // draw pc and it's matching memory
    for i in 0..5 { 
        if cpu.pc + (i*2) >= 0x1000 { continue; }
        {
            let x_off = REGION_PC_X + (REGION_WIDTH / (columns + 1)) * 2; 
            let y_off = REGION_PC_Y + (row_height * (memory_row + 1 + i as i32));
            write_text(
                format!("{:#04x}", cpu.pc + (i*2)), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }
        {
            let x_off = REGION_PC_X + (REGION_WIDTH / (columns + 1)) * 3; 
            let y_off = REGION_PC_Y + (row_height * (memory_row + 1 + i as i32));
            write_text(
                format!("{:02x}{:02x}", cpu.memory[cpu.pc+i], cpu.memory[cpu.pc+i+1]), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }
    }
} 

fn draw_i_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
    let row_height = 2*font.height(); 
    let columns = 4; 
    let title_row = 0; 
    let memory_row = 1; 

    // write title for this region
    write_text("--- I REGISTER POINTER ---".to_string(), 
        REGION_I_X + (REGION_WIDTH / 2), 
        REGION_I_Y + row_height * title_row, 
        Color::WHITE, 
        font, 
        canvas);

    // address and memory header
    { 
        let x_off = REGION_I_X + (REGION_WIDTH / (columns + 1)) * 2; 
        let y_off = REGION_I_Y + (row_height * memory_row);
        write_text(
            "ADDR".to_string(), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    { 
        let x_off = REGION_I_X + (REGION_WIDTH / (columns + 1)) * 3; 
        let y_off = REGION_I_Y + (row_height * memory_row);
        write_text(
            "MEM_".to_string(), 
            x_off, 
            y_off,
            Color::WHITE, 
            font, 
            canvas); 
    }

    // draw pc and it's matching memory
    for i in 0..5 { 
        if cpu.reg_i + (i*2) >= 0x1000 { continue; }
        {
            let x_off = REGION_I_X + (REGION_WIDTH / (columns + 1)) * 2; 
            let y_off = REGION_I_Y + (row_height * (memory_row + 1 + i as i32));
            write_text(
                format!("{:#04x}", cpu.reg_i + (i*2)), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }
        {
            let x_off = REGION_I_X + (REGION_WIDTH / (columns + 1)) * 3; 
            let y_off = REGION_I_Y + (row_height * (memory_row + 1 + i as i32));
            write_text(
                format!("{:02x}{:02x}", cpu.memory[cpu.reg_i+i], cpu.memory[cpu.reg_i+i+1]), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }
    }
}

fn draw_entire_window(canvas: &mut Canvas<Window>, cpu: &CPU, font: &Font, debug: bool, state: &GameState) { 
    canvas.set_draw_color(BACKGROUND_COLOR); 
    canvas.clear();

    if debug { 
        draw_register_region(cpu, canvas, font); 
        draw_pc_region(cpu, canvas, font); 
        draw_i_region(cpu, canvas, font); 
    }

    draw_rom_region(canvas, &cpu.pixels); 

    if state == &GameState::Paused { 
        write_text("PRESS [SPACE] TO START ROM".to_string(),
            REGION_ROM_X + REGION_WIDTH / 2, 
            REGION_ROM_Y + REGION_HEIGHT / 2, 
            Color::GRAY, 
            font, 
            canvas)
    }

    canvas.present(); 
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

    // load font 
    let point_size = 18; 
    let font = ttf_context.load_font(
        Path::new("/Users/nicktrueb/Programming/chip8/assets/FragmentMono-Regular.ttf"), 
        point_size
    ).expect("ERROR: failed to load external font"); 

    // reset canvas and update window
    let mut state: GameState = GameState::Paused;
    draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), &state); 

    let mut event_pump = sdl_context.event_pump()?;
    let mut should_render_screen: bool = true; 
    let mut manual_step_signal: bool = false; 
    let mut prev_cpu_cycle_time: Instant = Instant::now(); 
    let mut prev_screen_refresh_time: Instant = Instant::now();
    let cpu_cycle_freq: u128 = 500;  
    let screen_refresh_freq: u128 = 1000; 
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
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), &state); 
                        }, 
                        SdlKeycode::Space => {
                            state = match state {  
                                GameState::Paused  => GameState::Running, 
                                GameState::Running => GameState::Paused
                            }; 
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), &state);
                        },
                        SdlKeycode::RShift => {
                            match state {
                                GameState::Paused => manual_step_signal = true, 
                                _ => { }
                            }
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
                            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), &state); 
                        } 
                        _ => {}, 
                    }
                },
                _ => {}
            }
        }

        // process cpu instructions at specified cycle frequency
        if prev_cpu_cycle_time.elapsed().as_millis() >= 1_000 / cpu_cycle_freq { 
            prev_cpu_cycle_time = Instant::now(); 

            // do not update if game is paused and manual step button has not been pressed
            if state != GameState::Paused || (state == GameState::Paused && manual_step_signal) {
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
            draw_entire_window(&mut canvas, &cpu, &font, modes.contains(&OptionalModes::Debug), &state); 
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