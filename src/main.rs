
mod cpu; 
pub use cpu::cpu::{CPU, SCREEN_WIDTH, SCREEN_HEIGHT, Chip8Input}; 

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window; 

use std::collections::HashMap; 
 
const PIXEL_WIDTH  : usize = 10; 
const CANVAS_WIDTH : usize = SCREEN_WIDTH * PIXEL_WIDTH; 
const CANVAS_HEIGHT: usize = SCREEN_HEIGHT * PIXEL_WIDTH; 

// maps Keycode value (from sdl2) to Chip8Input value (0-F) to simulate controller input 
fn build_keycode_hashmap() -> HashMap<Keycode, Chip8Input>{ 
    HashMap::from([
        // ROW 1
        (Keycode::Num1, Chip8Input::Num1), 
        (Keycode::Num2, Chip8Input::Num2), 
        (Keycode::Num3, Chip8Input::Num3), 
        (Keycode::Num4, Chip8Input::C),
    
        // ROW 2
        (Keycode::Q, Chip8Input::Num4), 
        (Keycode::W, Chip8Input::Num5), 
        (Keycode::E, Chip8Input::Num6), 
        (Keycode::R, Chip8Input::D), 
    
        // ROW 3
        (Keycode::A, Chip8Input::Num7), 
        (Keycode::S, Chip8Input::Num8), 
        (Keycode::D, Chip8Input::Num9), 
        (Keycode::F, Chip8Input::E),
    
        // ROW 4 
        (Keycode::Z, Chip8Input::A), 
        (Keycode::X, Chip8Input::Num0), 
        (Keycode::C, Chip8Input::B), 
        (Keycode::V, Chip8Input::F),
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
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // define event_pump and frame counter for event polling and framerate control respectively
    let mut event_pump = sdl_context.event_pump()?;
    let mut key_pressed_this_frame: Option<Keycode> = None; 
    let mut frame_counter: usize = 0;
    let max_framerate: usize = 30;
    let keyboard_to_chip8_input_map = build_keycode_hashmap(); 

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
                        Keycode::Num1 
                        | Keycode::Num2
                        | Keycode::Num3 
                        | Keycode::Num4
                        | Keycode::Q 
                        | Keycode::W
                        | Keycode::E 
                        | Keycode::R
                        | Keycode::A
                        | Keycode::S 
                        | Keycode::D 
                        | Keycode::F
                        | Keycode::Z 
                        | Keycode::X
                        | Keycode::C 
                        | Keycode::V 
                        => {
                            key_pressed_this_frame = Some(key); 
                        }, 

                        Keycode::Escape => break 'running, 
                        _ => {}, 
                    }
                },
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => { 
                    println!("mouse clicked at ({},{})", x, y);
                }
                _ => {}
            }
        }

        // update the game at set framerate
        if frame_counter >= max_framerate {
            match key_pressed_this_frame { 
                Some(key) => cpu.step( keyboard_to_chip8_input_map.get(&key)), 
                None => cpu.step(None)
            };
            frame_counter = 0;
            key_pressed_this_frame = None;
        }

        // draw pixels on canvas
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
            canvas.fill_rect(rect).expect("");
        }

        canvas.present();
    }

    Ok(())
}

pub fn main() -> Result<(), String> {

    // load input file 

    // instantiate cpu
    let mut cpu: CPU = CPU::new();  

    // run cpu
    execute(cpu)

    // handle restarting cpu, loading new rom, etc
}