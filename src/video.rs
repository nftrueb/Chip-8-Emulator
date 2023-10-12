pub mod video {
    use sdl2::rect::Rect; 
    use sdl2::rect::Point; 
    use sdl2::pixels::Color; 
    use sdl2::render::Canvas; 
    use sdl2::video::Window; 
    use sdl2::ttf::Font; 

    use crate::cpu::cpu::{CPU, SCREEN_HEIGHT, SCREEN_WIDTH}; 

    // -----------------
    // --- CONSTANTS ---
    // -----------------
 
    pub const PIXEL_WIDTH        : usize = 10; 
    pub const CANVAS_WIDTH       : usize = SCREEN_WIDTH * PIXEL_WIDTH; 
    pub const CANVAS_HEIGHT      : usize = SCREEN_HEIGHT * PIXEL_WIDTH; 
    pub const DEBUG_CANVAS_WIDTH : usize = CANVAS_WIDTH * 2; 
    pub const DEBUG_CANVAS_HEIGHT: usize = CANVAS_HEIGHT * 2; 
    const BACKGROUND_COLOR       : Color = Color::RGB(50, 50, 150);
    const REGION_WIDTH           : i32 = CANVAS_WIDTH as i32; 
    const REGION_HEIGHT          : i32 = CANVAS_HEIGHT as i32; 
    const ROM_REGION             : usize = 0; 
    const REGISTERS_REGION       : usize = 1; 
    const PC_REGION              : usize = 2; 
    const I_REGION               : usize = 3; 
    const REGIONS: [(i32, i32); 4] = [
        (0,            0), 
        (REGION_WIDTH, 0), 
        (0,            REGION_HEIGHT), 
        (REGION_WIDTH, REGION_HEIGHT)
    ];

    // ------------------------
    // --- PUBLIC FUNCTIONS ---
    // ------------------------

    pub fn write_text(text: String, x_off: i32, y_off: i32, color: Color, font: &Font, canvas: &mut Canvas<Window>) { 
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

    pub fn draw_rom_region(canvas: &mut Canvas<Window>, pixels: &[bool; 2048]) { 
        let (region_x, region_y) = REGIONS[ROM_REGION]; 
        for (idx, pixel) in pixels.into_iter().enumerate() { 
            let row: usize = idx / SCREEN_WIDTH; 
            let col: usize = idx % SCREEN_WIDTH;
            let rect = Rect::new(
                region_x + (col * PIXEL_WIDTH) as i32, 
                region_y + (row * PIXEL_WIDTH) as i32, 
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
        }
    }

    pub fn draw_register_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
        let (region_x, region_y) = REGIONS[REGISTERS_REGION]; 
        let row_height = 2*font.height(); 
        let columns = 4; 
        let title_row = 0; 
        let v_register_row = 1; 
        let i_register_row = 5; 
        let pc_row = 5;

        // write title for this region
        write_text("--- REGISTERS ---".to_string(), 
        region_x + (REGION_WIDTH / 2), 
        region_y + row_height * title_row, 
            Color::WHITE, 
            font, 
            canvas);

        // write grid of registers and their values
        for (idx, value) in cpu.registers.iter().enumerate() { 
            let x_off = region_x + (REGION_WIDTH / (columns + 1)) * ((idx as i32 % columns) + 1); 
            let y_off = region_y + (row_height * v_register_row) + (row_height * (idx as i32 / columns));
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
            let x_off = region_x + REGION_WIDTH / (columns + 1); 
            let y_off = region_y + row_height * i_register_row; 
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
            let x_off = region_x + REGION_WIDTH / (columns + 1) * 2; 
            let y_off = region_y + row_height * pc_row; 
            write_text(
                format!("PC: x{:03x}", cpu.pc), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }
    }

    pub fn draw_pc_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
        let (region_x, region_y) = REGIONS[PC_REGION]; 
        let row_height = 2*font.height(); 
        let columns = 4; 
        let title_row = 0; 
        let memory_row = 1; 

        // write title for this region
        write_text("--- PC POINTER ---".to_string(), 
        region_x + (REGION_WIDTH / 2), 
        region_y + row_height * title_row, 
            Color::WHITE, 
            font, 
            canvas);

        // address and memory header
        { 
            let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 2; 
            let y_off = region_y + (row_height * memory_row);
            write_text(
                "ADDR".to_string(), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }

        { 
            let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 3; 
            let y_off = region_y + (row_height * memory_row);
            write_text(
                "MEM_".to_string(), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }

        // draw pc and it's matching memory
        for i in (0..10).step_by(2) { 
            if cpu.pc + (i*2) >= 0x1000 { continue; }
            {
                let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 2; 
                let y_off = region_y + (row_height * (memory_row + 1 + (i/2) as i32));
                write_text(
                    format!("{:#04x}", cpu.pc + i), 
                    x_off, 
                    y_off,
                    Color::WHITE, 
                    font, 
                    canvas); 
            }
            {
                let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 3; 
                let y_off = region_y + (row_height * (memory_row + 1 + (i/2) as i32));
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

    pub fn draw_i_region(cpu: &CPU, canvas: &mut Canvas<Window>, font: &Font) { 
        let (region_x, region_y) = REGIONS[I_REGION]; 
        let row_height = 2*font.height(); 
        let columns = 4; 
        let title_row = 0; 
        let memory_row = 1; 

        // write title for this region
        write_text("--- I REGISTER POINTER ---".to_string(), 
        region_x + (REGION_WIDTH / 2), 
        region_y + row_height * title_row, 
            Color::WHITE, 
            font, 
            canvas);

        // address and memory header
        { 
            let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 2; 
            let y_off = region_y + (row_height * memory_row);
            write_text(
                "ADDR".to_string(), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }

        { 
            let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 3; 
            let y_off = region_y + (row_height * memory_row);
            write_text(
                "MEM_".to_string(), 
                x_off, 
                y_off,
                Color::WHITE, 
                font, 
                canvas); 
        }

        // draw pc and it's matching memory
        for i in (0..10).step_by(2) { 
            if cpu.reg_i + (i*2) >= 0x1000 { continue; }
            {
                let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 2; 
                let y_off = region_y + (row_height * (memory_row + 1 + (i/2) as i32));
                write_text(
                    format!("{:#04x}", cpu.reg_i + i), 
                    x_off, 
                    y_off,
                    Color::WHITE, 
                    font, 
                    canvas); 
            }
            {
                let x_off = region_x + (REGION_WIDTH / (columns + 1)) * 3; 
                let y_off = region_y + (row_height * (memory_row + 1 + (i/2) as i32));
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

    pub fn draw_entire_window(canvas: &mut Canvas<Window>, cpu: &CPU, font: &Font, debug: bool, paused_state: bool) { 
    canvas.set_draw_color(BACKGROUND_COLOR); 
    canvas.clear();

    if debug { 
        draw_register_region(cpu, canvas, font); 
        draw_pc_region(cpu, canvas, font); 
        draw_i_region(cpu, canvas, font); 
    }

    draw_rom_region(canvas, &cpu.pixels); 

    if debug { 
        canvas.set_draw_color(Color::WHITE); 
        canvas.draw_line(Point::new(REGION_WIDTH, 0), Point::new(REGION_WIDTH, REGION_HEIGHT*2)).unwrap();
        canvas.draw_line(Point::new(0, REGION_HEIGHT), Point::new(REGION_WIDTH*2, REGION_HEIGHT)).unwrap(); 
    }

    if paused_state { 
        write_text("PRESS [SPACE] TO START ROM".to_string(),
            REGIONS[ROM_REGION].0 + REGION_WIDTH / 2, 
            REGIONS[ROM_REGION].1 + REGION_HEIGHT / 2, 
            Color::GRAY, 
            font, 
            canvas)
    }

    canvas.present(); 
}
}