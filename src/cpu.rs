pub mod cpu { 

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

    pub struct CPU { 
        pub pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], 
        pub memory: [u8; 0x1000], 
        pub registers: [u8; 0xF], 
        _reg_i: usize, 
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
                _reg_i: 0, 
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

        pub fn step(&mut self, _input: Option<&Chip8Input>) { 

            // get next instruction 
            let instruction: usize = ((self.memory[self.pc] as usize) << 8) + self.memory[self.pc+1] as usize; 

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
                        0x0 => self.opcode_1nnn(instruction),
                        0x1 => self.opcode_1nnn(instruction),
                        0x2 => self.opcode_1nnn(instruction),
                        0x3 => self.opcode_1nnn(instruction),
                        0x4 => self.opcode_1nnn(instruction),
                        0x5 => self.opcode_1nnn(instruction),
                        0x6 => self.opcode_1nnn(instruction),
                        0x7 => self.opcode_1nnn(instruction),
                        0xE => self.opcode_1nnn(instruction),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                0xA000 => self.opcode_1nnn(instruction), 
                0xB000 => self.opcode_1nnn(instruction), 
                0xC000 => self.opcode_1nnn(instruction), 
                0xD000 => self.opcode_1nnn(instruction), 
                0xE000 => {
                    match instruction & 0x00FF { 
                        0x009E => self.opcode_1nnn(instruction),
                        0x00A1 => self.opcode_1nnn(instruction),
                        _ => panic!("invalid opcode found! 0x{:X}", instruction)
                    }
                },
                0xF000 => {
                    match instruction & 0x00FF { 
                        0x0007 => self.opcode_1nnn(instruction),
                        0x000A => self.opcode_1nnn(instruction),
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
            self.pc += 2;
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

        cpu.step(None); 

        assert!(!cpu.pixels[0]);
    }

    #[test] 
    fn should_pop_return_stack_when_opcode_00ee() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x00; 
        cpu.memory[1] = 0xee;
        cpu.stack[0] = 0xFFC; 
        cpu.sp = 1; 

        cpu.step(None); 

        assert!(cpu.sp == 0); 
        assert!(cpu.pc == 0xFFC + 2); 
    }

    #[test] 
    fn should_jump_when_opcode_1nnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x11; 
        cpu.memory[1] = 0x20;

        cpu.step(None); 

        assert!(cpu.pc == 0x122); 
    }

    #[test]
    fn should_push_return_addr_and_jump_to_nnn_when_opcode_2nnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x2F; 
        cpu.memory[1] = 0xFC;

        cpu.step(None); 

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

        cpu.step(None); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_should_skip_next_instr_when_opcode_4xnn_and_vx_doesnt_equal_nn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x41; 
        cpu.memory[1] = 0x23;
        cpu.registers[1] = 0x00; 

        cpu.step(None); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_should_skip_next_instr_when_opcode_5xy0_and_vx_equals_vy() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x51; 
        cpu.memory[1] = 0x20;
        cpu.registers[1] = 0x8; 
        cpu.registers[2] = 0x8; 

        cpu.step(None); 

        assert!(cpu.pc == 0x0004); 
    }

    #[test]
    fn should_set_vx_to_nn_when_opcode_6xnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x61; 
        cpu.memory[1] = 0x20;

        cpu.step(None); 

        assert!(cpu.registers[1] == 0x20); 
    }

    #[test]
    fn should_add_nn_to_vx_when_opcode_6xnn() { 
        let mut cpu = CPU::new(); 
        cpu.memory[0] = 0x71; 
        cpu.memory[1] = 0x20;
        cpu.registers[1] = 0xF; 

        cpu.step(None); 

        assert!(cpu.registers[1] == 0x2F); 
    }

    // reset 

    // load rom 

    // step
}