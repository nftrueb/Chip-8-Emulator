pub mod cpu { 

    use rand::prelude::*; 

    // -------------------
    // ---- CONSTANTS ----
    // ------------------- 

    pub const SCREEN_WIDTH : usize = 64; 
    pub const SCREEN_HEIGHT: usize = 32;
    const MAX_STACK_SIZE: usize = 12;
    const ROM_START_ADDR: usize = 0x200; 

    // -------------------------
    // ---- STRUCTS / ENUMS ----
    // -------------------------

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

    pub struct CPU { 
        pub pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], 
        pub memory: [u8; 0x1000], 
        pub registers: [u8; 0xF], 
        reg_i: usize, 
        pub stack: [usize; MAX_STACK_SIZE], 
        _delay_timer: usize, 
        _sound_timer: usize,
        pub pc: usize, // PROGRAM COUNTER
        pub sp: usize, // STACK POINTER
    }

    impl CPU { 

        pub fn new() -> Self { 
            CPU { 
                pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
                memory: [0; 0x1000], 
                registers: [0; 0xF], 
                reg_i: 0, 
                stack: [0; MAX_STACK_SIZE], 
                _delay_timer: 0, 
                _sound_timer: 0, 
                pc: 0, 
                sp: 0,
            }
        }

        pub fn reset(&mut self) { 

        }

        pub fn load_rom(&mut self, rom: Vec<u8>) { 
            for i in 0..rom.len() { 
                self.memory[ROM_START_ADDR+i] = rom[i]; 
            }
        }

        fn parse_pressed_keys(target: usize, pressed_keys: Vec<usize>) -> bool { 
            for key in pressed_keys { 
                if target == key { return true }
            }
            false
        }

        pub fn step(&mut self, pressed_keys: Vec<usize>) { 

            // get next instruction 
            let instruction: usize = ((self.memory[self.pc] as usize) << 8) + self.memory[self.pc+1] as usize; 
            let mut pc_inc: bool = true;

            // execute instruction 
            match instruction & 0xF000 { 
                0x0000 => {
                    match instruction & 0x00FF { 
                        0x00E0 => self.opcode_00e0(),
                        0x00EE => self.opcode_00ee(), 
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                }, 
                0x1000 => self.opcode_1nnn(instruction), 
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
                0xD000 => self.opcode_dxyn(instruction), 
                0xE000 => {
                    match instruction & 0x00FF { 
                        0x009E => self.opcode_ex9e(instruction, pressed_keys),
                        0x00A1 => self.opcode_exa1(instruction, pressed_keys),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                0xF000 => {
                    match instruction & 0x00FF { 
                        0x0007 => self.opcode_1nnn(instruction),
                        0x000A => pc_inc = self.opcode_fx0a(instruction, pressed_keys),
                        0x0015 => self.opcode_1nnn(instruction),
                        0x0018 => self.opcode_1nnn(instruction),
                        0x001E => self.opcode_1nnn(instruction),
                        0x0029 => self.opcode_1nnn(instruction),
                        0x0033 => self.opcode_1nnn(instruction),
                        0x0055 => self.opcode_1nnn(instruction),
                        0x0065 => self.opcode_1nnn(instruction),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                _ => panic!("invalid opcode found! 0x{:X}", instruction) 
            }

            // increment pc 
            self.pc += if pc_inc { 2 } else { 0 }
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