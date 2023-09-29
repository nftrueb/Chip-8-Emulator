pub mod cpu { 

    // -------------------
    // ---- CONSTANTS ----
    // ------------------- 

    pub const SCREEN_WIDTH : usize = 64; 
    pub const SCREEN_HEIGHT: usize = 32;
    const MAX_STACK_SIZE: usize = 12;

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
        memory: [u8; 0x1000], 
        registers: [u8; 0xF], 
        reg_i: usize, 
        stack: [usize; MAX_STACK_SIZE], 
        delay_timer: usize, 
        sound_timer: usize,
        pc: usize, // PROGRAM COUNTER
        sp: usize, // STACK POINTER
    }

    impl CPU { 

        pub fn new() -> Self { 
            CPU { 
                pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
                memory: [0; 0x1000], 
                registers: [0; 0xF], 
                reg_i: 0, 
                stack: [0; MAX_STACK_SIZE], 
                delay_timer: 0, 
                sound_timer: 0, 
                pc: 0, 
                sp: 0,
            }
        }

        pub fn reset(&mut self) { 

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
                        _ => self.opcode_0nnn(instruction)
                    }
                }, 
                0x1000 => self.opcode_1nnn(instruction), 
                0x2000 => self.opcode_1nnn(instruction), 
                0x3000 => self.opcode_1nnn(instruction), 
                0x4000 => self.opcode_1nnn(instruction), 
                0x5000 => self.opcode_1nnn(instruction), 
                0x6000 => self.opcode_1nnn(instruction), 
                0x7000 => self.opcode_1nnn(instruction),
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

        fn opcode_0nnn(&mut self, instruction: usize) { 
            if self.sp >= MAX_STACK_SIZE { 
                panic!("Stack is too full to push new return addresses"); 
            }

            self.stack[self.sp] = self.pc; 
            self.pc = instruction & 0xFFF; 
        }

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

    }

}