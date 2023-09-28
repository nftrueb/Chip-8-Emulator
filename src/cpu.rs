
mod operations;
pub use operations::*; 

pub mod cpu { 

    // -------------------
    // ---- CONSTANTS ----
    // ------------------- 

    pub const SCREEN_WIDTH : usize = 64; 
    pub const SCREEN_HEIGHT: usize = 32;

    // -------------------------
    // ---- STRUCTS / ENUMS ----
    // -------------------------

    pub enum State { 
        Paused, 
        Running
    }

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
        pub state: State, 
    }

    impl CPU { 

        pub fn new() -> Self { 
            CPU { pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT], state: State::Paused }
        }

        pub fn step(&mut self, input: Option<&Chip8Input>) { 

        }

    }

}