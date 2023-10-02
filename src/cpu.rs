pub mod cpu { 

    use rand::prelude::*; 

    // -------------------
    // ---- CONSTANTS ----
    // ------------------- 

    pub const SCREEN_WIDTH : usize = 64; 
    pub const SCREEN_HEIGHT: usize = 32;
    const MAX_STACK_SIZE: usize = 12;
    const ROM_START_ADDR: usize = 0x200; 

    const FONT_HEIGHT: usize = 5; 
    const FONT_START: usize = 0x0000; 
    const FONT_SPRITE_START_ADDRS: [usize; 16] = [
        FONT_START,                     // FONT_0
        FONT_START + FONT_HEIGHT * 0x1, // FONT_1
        FONT_START + FONT_HEIGHT * 0x2, // FONT_2
        FONT_START + FONT_HEIGHT * 0x3, // FONT_3
        FONT_START + FONT_HEIGHT * 0x4, // FONT_4
        FONT_START + FONT_HEIGHT * 0x5, // FONT_5
        FONT_START + FONT_HEIGHT * 0x6, // FONT_6
        FONT_START + FONT_HEIGHT * 0x7, // FONT_7
        FONT_START + FONT_HEIGHT * 0x8, // FONT_8
        FONT_START + FONT_HEIGHT * 0x9, // FONT_9
        FONT_START + FONT_HEIGHT * 0xA, // FONT_A
        FONT_START + FONT_HEIGHT * 0xB, // FONT_B
        FONT_START + FONT_HEIGHT * 0xC, // FONT_C
        FONT_START + FONT_HEIGHT * 0xD, // FONT_D
        FONT_START + FONT_HEIGHT * 0xE, // FONT_E
        FONT_START + FONT_HEIGHT * 0xF, // FONT_F
    ]; 

    const FONT_DATAS: [[u8; FONT_HEIGHT]; 16] = [ 
        [0xF0, 0x90, 0x90, 0x90, 0xF0], // FONT_0
        [0x20, 0x60, 0x20, 0x20, 0x70], // FONT_1
        [0xF0, 0x10, 0xF0, 0x80, 0xF0], // FONT_2
        [0xF0, 0x10, 0xF0, 0x10, 0xF0], // FONT_3
        [0x90, 0x90, 0xF0, 0x10, 0x10], // FONT_4
        [0xF0, 0x80, 0xF0, 0x10, 0xF0], // FONT_5
        [0xF0, 0x80, 0xF0, 0x90, 0xF0], // FONT_6
        [0xF0, 0x10, 0x20, 0x40, 0x40], // FONT_7
        [0xF0, 0x90, 0xF0, 0x90, 0xF0], // FONT_8
        [0xF0, 0x90, 0xF0, 0x10, 0xF0], // FONT_9
        [0xF0, 0x90, 0xF0, 0x90, 0x90], // FONT_A
        [0xE0, 0x90, 0xE0, 0x90, 0xE0], // FONT_B
        [0xF0, 0x80, 0x80, 0x80, 0xF0], // FONT_C
        [0xE0, 0x90, 0x90, 0x90, 0xE0], // FONT_D
        [0xF0, 0x80, 0xF0, 0x80, 0xF0], // FONT_E
        [0xF0, 0x80, 0xF0, 0x80, 0x80], // FONT_F
    ];

    // -------------------------------------
    // ---- STRUCTS / ENUMS / HELPER FN ----
    // -------------------------------------

    #[derive(PartialEq, Eq, Hash)]
    pub enum Chip8Input { 
        Num0,  
        Num1, 
        Num2, 
        Num3, 
        Num4, 
        Num5, 
        Num6, 
        Num7, 
        Num8, 
        Num9, 
        A, 
        B, 
        C, 
        D,
        E, 
        F
    }

    pub fn get_chip8_key_idx(key: &Chip8Input) -> usize { 
        match key {
            Chip8Input::Num0 => 0x0, 
            Chip8Input::Num1 => 0x1, 
            Chip8Input::Num2 => 0x2, 
            Chip8Input::Num3 => 0x3, 
            Chip8Input::Num4 => 0x4, 
            Chip8Input::Num5 => 0x5, 
            Chip8Input::Num6 => 0x6, 
            Chip8Input::Num7 => 0x7, 
            Chip8Input::Num8 => 0x8, 
            Chip8Input::Num9 => 0x9, 
            Chip8Input::A => 0xA, 
            Chip8Input::B => 0xB, 
            Chip8Input::C => 0xC, 
            Chip8Input::D => 0xD, 
            Chip8Input::E => 0xE, 
            Chip8Input::F => 0xF, 
        }
    }

    fn get_fresh_memory_with_font_data() -> [u8; 0x1000] { 
        let mut memory: [u8; 0x1000] = [0; 0x1000];
        for i in 0..0xF { 
            for j in 0..FONT_HEIGHT { 
                memory[FONT_SPRITE_START_ADDRS[i]+j] = FONT_DATAS[i][j];
            }
        }
        memory
    }

    pub struct CPU { 
        pub pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], 
        pub memory: [u8; 0x1000], 
        pub registers: [u8; 16], 
        pub reg_i: usize, 
        pub stack: [usize; MAX_STACK_SIZE], 
        delay_timer: u8, 
        pub sound_timer: u8,
        pub pc: usize, // PROGRAM COUNTER
        pub sp: usize, // STACK POINTER
        rom_len: usize, 
    }

    impl CPU { 

        pub fn new() -> Self { 
            let memory: [u8; 0x1000] = get_fresh_memory_with_font_data(); 
            CPU { 
                pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
                memory, 
                registers: [0; 16], 
                reg_i: 0, 
                stack: [0; MAX_STACK_SIZE], 
                delay_timer: 0, 
                sound_timer: 0, 
                pc: ROM_START_ADDR, 
                sp: 0,
                rom_len: 0, 
            }
        }

        pub fn reset(&mut self) {
            for idx in 0..self.pixels.len() { 
                self.pixels[idx] = false; 
            } 
            for idx in 0..self.registers.len() { 
                self.registers[idx] = 0x00; 
            }
            for idx in 0..MAX_STACK_SIZE { 
                self.stack[idx] = 0x0000; 
            }
            self.reg_i = 0x0000; 
            self.delay_timer = 0; 
            self.sound_timer = 0; 
            self.pc = ROM_START_ADDR; 
            self.sp = 0;
        }

        pub fn load_rom(&mut self, rom: Vec<u8>) { 
            self.rom_len = rom.len(); 
            for i in 0..rom.len() { 
                self.memory[ROM_START_ADDR+i] = rom[i]; 
            }
        }

        pub fn dump_memory(&self) -> String { 
            let mut memory_dump: String = String::from("---- CHIP-8 MEMORY ----"); 
            for (idx, byte) in self.memory.iter().enumerate() { 
                if idx < ROM_START_ADDR { continue; }
                if idx > ROM_START_ADDR + self.rom_len { break; }

                let new_byte = if idx % 16 == 0 { 
                    format!("\n{:#06x} :: {:02x}", idx, byte)
                } else if idx % 2 == 0 { 
                    format!("{:02x}", byte)
                } else { 
                    format!("{:02x} ", byte)
                };

                memory_dump.push_str(new_byte.as_str()); 
            }
            memory_dump.push_str("\n");
            memory_dump 
        }

        fn parse_pressed_keys(target: usize, pressed_keys: Vec<usize>) -> bool { 
            for key in pressed_keys { 
                if target == key { return true }
            }
            false
        }

        pub fn step(&mut self, pressed_keys: Vec<usize>) -> (bool, String, String) { 

            // decrement timers
            if self.delay_timer > 0 { self.delay_timer -= 1; }
            if self.sound_timer > 0 { self.sound_timer -= 1; }

            // get next instruction 
            let instruction: usize = ((self.memory[self.pc] as usize) << 8) + self.memory[self.pc+1] as usize; 
            let mut pc_inc: bool = true;
            let mut should_render_screen: bool = false; 

            // execute instruction 
            match instruction & 0xF000 { 
                0x0000 => {
                    match instruction & 0x00FF { 
                        0x00E0 => self.opcode_00e0(),
                        0x00EE => self.opcode_00ee(), 
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                }, 
                0x1000 => { 
                    pc_inc = false; 
                    self.opcode_1nnn(instruction)
                }, 
                0x2000 => self.opcode_2nnn(instruction), 
                0x3000 => self.opcode_3xnn(instruction), 
                0x4000 => self.opcode_4xnn(instruction), 
                0x5000 => self.opcode_5xy0(instruction), 
                0x6000 => self.opcode_6xnn(instruction), 
                0x7000 => self.opcode_7xnn(instruction),
                0x8000 => {
                    match instruction & 0x000F { 
                        0x0 => self.opcode_8xy0(instruction),
                        0x1 => self.opcode_8xy1(instruction),
                        0x2 => self.opcode_8xy2(instruction),
                        0x3 => self.opcode_8xy3(instruction),
                        0x4 => self.opcode_8xy4(instruction),
                        0x5 => self.opcode_8xy5(instruction),
                        0x6 => self.opcode_8xy6(instruction),
                        0x7 => self.opcode_8xy7(instruction),
                        0xE => self.opcode_8xye(instruction),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                0x9000 => self.opcode_9xy0(instruction),
                0xA000 => self.opcode_annn(instruction), 
                0xB000 => self.opcode_bnnn(instruction), 
                0xC000 => self.opcode_cxnn(instruction), 
                0xD000 => {
                    self.opcode_dxyn(instruction);
                    should_render_screen = true; 
                }, 
                0xE000 => {
                    match instruction & 0x00FF { 
                        0x009E => self.opcode_ex9e(instruction, pressed_keys),
                        0x00A1 => self.opcode_exa1(instruction, pressed_keys),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                0xF000 => {
                    match instruction & 0x00FF { 
                        0x0007 => self.opcode_fx07(instruction),
                        0x000A => pc_inc = self.opcode_fx0a(instruction, pressed_keys),
                        0x0015 => self.opcode_fx15(instruction),
                        0x0018 => self.opcode_fx18(instruction),
                        0x001E => self.opcode_fx1e(instruction),
                        0x0029 => self.opcode_fx29(instruction),
                        0x0033 => self.opcode_fx33(instruction),
                        0x0055 => self.opcode_fx55(instruction),
                        0x0065 => self.opcode_fx65(instruction),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                _ => panic!("invalid opcode found! 0x{:X}", instruction) 
            }

            // increment pc 
            let old_pc: usize = self.pc; 
            self.pc += if pc_inc { 2 } else { 0 }; 

            return (should_render_screen, 
                format!("{:#04x}", old_pc), 
                format!("{:04x}", instruction)
            ); 
        }

        // ------------------------
        // ---- OPCODE HELPERS ----
        // ------------------------

        // clear screen
        fn opcode_00e0(&mut self) { 
            for i in 0..(SCREEN_HEIGHT*SCREEN_WIDTH) { 
                self.pixels[i] = false; 
            }
        }

        // return from subroutine
        fn opcode_00ee(&mut self) {
            self.sp -= 1; 
            self.pc = self.stack[self.sp];
        }

        // jump to nnn
        fn opcode_1nnn(&mut self, instruction: usize) { 
            self.pc = instruction & 0x0FFF;
        }   

        // call subroutine 
        fn opcode_2nnn(&mut self, instruction: usize) { 
            if self.sp >= MAX_STACK_SIZE { 
                panic!("Stack is too full to push new return addresses"); 
            }
            self.stack[self.sp] = self.pc; 
            self.sp += 1; 
            self.pc = instruction & 0xFFF; 
        }

        fn opcode_3xnn(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let nn: usize = instruction & 0x00FF;
            if self.registers[x] == nn as u8 { self.pc += 2; }
        }

        fn opcode_4xnn(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let nn: usize = instruction & 0x00FF;
            if self.registers[x] != nn as u8 { self.pc += 2; }
        }

        fn opcode_5xy0(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            if self.registers[x] == self.registers[y] { self.pc += 2; }
        }

        fn opcode_6xnn(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let nn: usize = instruction & 0x00FF;
            self.registers[x] = nn as u8; 
        }

        // TODO: check possibility of overflow
        fn opcode_7xnn(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let nn: usize = instruction & 0x00FF;
            self.registers[x] += nn as u8; 
        }

        // Vx = Vy
        fn opcode_8xy0(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] = self.registers[y];
        }

        // Vx |= Vy
        fn opcode_8xy1(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] |= self.registers[y]; 
        }

        // Vx &= Vy
        fn opcode_8xy2(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] &= self.registers[y];
        }

        // Vx ^= Vy
        fn opcode_8xy3(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] ^= self.registers[y];
        }

        // TODO: check overflow
        // Vx += Vy
        fn opcode_8xy4(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] += self.registers[y];
        }

        // TODO: check underflow
        // Vx -= Vy
        fn opcode_8xy5(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] -= self.registers[y];
        }

        // Vx >>= 1
        fn opcode_8xy6(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.registers[0xF] = self.registers[x] & 0x1; 
            self.registers[x] >>= 1;  
        }

        // Vx = Vy - Vx
        fn opcode_8xy7(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            self.registers[x] = self.registers[y] - self.registers[x];
        }

        // Vx <<= 1
        fn opcode_8xye(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.registers[0xF] = if self.registers[x] & 0x80 == 0 { 0 } else { 1 }; 
            self.registers[x] <<= 1;  
        }

        // skip if Vx != Vy
        fn opcode_9xy0(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8; 
            let y: usize = (instruction & 0x00F0) >> 4;
            if self.registers[x] != self.registers[y] { self.pc += 2; }
        }

        // I = NNN
        fn opcode_annn(&mut self, instruction: usize) { 
            self.reg_i = instruction & 0xFFF; 
        }

        // PC = V0 + NNN
        fn opcode_bnnn(&mut self, instruction: usize) { 
            self.pc = self.registers[0] as usize + instruction & 0xFFF; 
        }

        // Vx = rand() & NN
        fn opcode_cxnn(&mut self, instruction: usize) { 
            let x: usize = (instruction & 0x0F00) >> 8;
            let rand_byte: u8 = random(); 
            self.registers[x] = rand_byte & (instruction & 0xFF) as u8;
        }

        // draw(Vx, Vy), N rows of 8 pixels
        fn opcode_dxyn(&mut self, instruction: usize) { 
            let x = (instruction & 0x0F00) >> 8; 
            let y = (instruction & 0x00F0) >> 4; 
            let n = instruction & 0xF;

            let start_x = self.registers[x] as usize; 
            let start_y = self.registers[y] as usize; 
            let sprite_start = start_y * SCREEN_WIDTH + start_x; 

            self.registers[0xF] = 0; 
            for row_idx in 0..n { 
                let mut row: u8 = self.memory[self.reg_i + row_idx]; 
                for col_idx in 0..8 { 
                    let pixels_idx = sprite_start + row_idx * SCREEN_WIDTH + col_idx; 
                    let sprite_pixel = (row >> 7) & 0x1; 
                    let existing_pixel = self.pixels[pixels_idx] as u8; 
                    if existing_pixel == 1 && sprite_pixel == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.pixels[pixels_idx] = existing_pixel ^ sprite_pixel == 1; 
                    row <<= 1; 
                }
            }
        }

        // skip if key() == Vx
        fn opcode_ex9e(&mut self, instruction: usize, pressed_keys: Vec<usize>) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            if CPU::parse_pressed_keys(x, pressed_keys) { 
                self.pc += 2; 
            }
        }

        // skip if key() != Vx
        fn opcode_exa1(&mut self, instruction: usize, pressed_keys: Vec<usize>) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            if !CPU::parse_pressed_keys(x, pressed_keys) { 
                self.pc += 2; 
            }
        }

        // Vx = key()
        // returns true if key press was found and recorded in Vx
        fn opcode_fx0a(&mut self, instruction: usize, pressed_keys: Vec<usize>) -> bool {
            if pressed_keys.len() == 0 { 
                return false;
            }
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.registers[x] = pressed_keys[0] as u8;
            true 
        }

        // Vx = delay_timer
        fn opcode_fx07(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.registers[x] = self.delay_timer;
        }

        // delay_timer = Vx
        fn opcode_fx15(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.delay_timer = self.registers[x];
        }

        // sound_timer = Vx
        fn opcode_fx18(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.sound_timer = self.registers[x];
        }

        // TODO: check overflow
        // I += Vx
        fn opcode_fx1e(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            self.reg_i += self.registers[x] as usize;
        }

        // I = char_sprite_addr[Vx]
        fn opcode_fx29(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            let vx: usize = (self.registers[x] & 0xF) as usize; 
            self.reg_i = FONT_SPRITE_START_ADDRS[vx]; 
        }

        // BCD of Vx stored in I -> if Vx 123 then I = 1, I+1 = 2, I+2 = 3
        fn opcode_fx33(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            let mut vx: u8 = self.registers[x] & 0xF; 
            
            self.memory[self.reg_i+2] = vx % 10;
            vx /= 10; 

            self.memory[self.reg_i+1] = vx % 10;
            vx /= 10; 

            self.memory[self.reg_i] = vx % 10; 
        }

        // LD [I], Vx -> store V0-Vx in memory starting at I
        fn opcode_fx55(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            for i in 0..x { 
                self.memory[self.reg_i+i] = self.registers[i]; 
            }
        }

        // LD Vx, [I] -> load V0-Vx with memory starting at I
        fn opcode_fx65(&mut self, instruction: usize) {
            let x: usize = (instruction & 0x0F00) >> 8; 
            for i in 0..x { 
                self.registers[i] = self.memory[self.reg_i+i]; 
            }
        }

    }

}


#[cfg(test)]
mod tests { 
    use super::cpu::CPU; 

    #[test] 
    fn should_clear_screen_when_opcode_00e0() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x00; 
        cpu.memory[1] = 0xe0;
        cpu.pixels[0] = true; 

        cpu.step(Vec::new()); 

        assert!(!cpu.pixels[0]);
    }

    #[test] 
    fn should_pop_return_stack_when_opcode_00ee() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x00; 
        cpu.memory[1] = 0xee;
        cpu.stack[0] = 0xFFC; 
        cpu.sp = 1; 

        cpu.step(Vec::new()); 

        assert!(cpu.sp == 0); 
        assert!(cpu.pc == 0xFFC + 2); 
    }

    #[test] 
    fn should_jump_when_opcode_1nnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x11; 
        cpu.memory[1] = 0x20;

        cpu.step(Vec::new()); 

        assert!(cpu.pc == 0x122); 
    }

    #[test]
    fn should_push_return_addr_and_jump_to_nnn_when_opcode_2nnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x2F; 
        cpu.memory[1] = 0xFC;

        cpu.step(Vec::new()); 

        assert!(cpu.stack[0] == 0x0000);
        assert!(cpu.sp == 1); 
        assert!(cpu.pc == 0xFFC + 2); 
    }

    #[test]
    fn should_should_skip_next_instr_when_opcode_3xnn_and_vx_equals_nn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x31; 
        cpu.memory[1] = 0x23;
        cpu.registers[1] = 0x23; 

        cpu.step(Vec::new()); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_should_skip_next_instr_when_opcode_4xnn_and_vx_doesnt_equal_nn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x41; 
        cpu.memory[1] = 0x23;
        cpu.registers[1] = 0x00; 

        cpu.step(Vec::new()); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_should_skip_next_instr_when_opcode_5xy0_and_vx_equals_vy() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x51; 
        cpu.memory[1] = 0x20;
        cpu.registers[1] = 0x8; 
        cpu.registers[2] = 0x8; 

        cpu.step(Vec::new()); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_set_vx_to_nn_when_opcode_6xnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x61; 
        cpu.memory[1] = 0x20;

        cpu.step(Vec::new()); 

        assert!(cpu.registers[1] == 0x20); 
    }

    #[test]
    fn should_add_nn_to_vx_when_opcode_6xnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x71; 
        cpu.memory[1] = 0x20;
        cpu.registers[1] = 0xF; 

        cpu.step(Vec::new()); 

        assert!(cpu.registers[1] == 0x2F); 
    }

    #[test]
    fn should_set_vx_to_vy_when_opcode_8xy0() { }

    #[test]
    fn should_set_vx_to_or_with_vy_when_opcode_8xy1() { }

    #[test]
    fn should_set_vx_to_and_with_vy_when_opcode_8xy2() { }

    #[test]
    fn should_set_vx_to_xor_with_vy_when_opcode_8xy3() { }

    #[test]
    fn should_add_vy_to_vx_when_opcode_8xy4() { }

    #[test]
    fn should_subtract_vy_from_vx_when_opcode_8xy5() { }

    #[test]
    fn should_right_shift_vx_when_opcode_8xy6() { }

    #[test]
    fn should_set_vx_to_vx_subtracted_from_vy_when_opcode_8xy7() { }

    #[test]
    fn should_left_shift_vx_when_opcode_8xye() { }

    // reset 

    // load rom 

    // step
}