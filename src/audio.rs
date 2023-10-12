
pub mod audio { 

    use sdl2::audio::AudioCallback; 

    pub struct SquareWave { 
        pub phase: f32, 
        pub phase_inc: f32,
        pub amplitude: f32,
    }

    impl AudioCallback for SquareWave { 
        type Channel = f32; 

        fn callback(&mut self, out: &mut [f32]) { 
            for x in out.iter_mut() { 
                *x = if self.phase <= 0.5 { self.amplitude } else { -self.amplitude };
                self.phase = (self.phase + self.phase_inc) % 1.0; 
            }
        }
    }
}