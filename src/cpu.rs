
mod operations;
pub use operations::*; 

pub mod cpu { 

    // -------------------
    // ---- CONSTANTS ----
    // ------------------- 

    pub const SCREEN_WIDTH : usize = 64; 
    pub const SCREEN_HEIGHT: usize = 32;

    pub struct CPU { 
        pub pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], 
    }

    impl CPU { 

        pub fn new() -> Self { 
            CPU { pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT] }
        }

        pub fn step(&mut self) { 

        }

    }

}