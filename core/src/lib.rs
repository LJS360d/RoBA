#![forbid(unsafe_code)]

use std::path::PathBuf;

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cart;
pub mod cpu;
pub mod io;
pub mod mem;
pub mod ppu;
pub mod timing;
pub mod video;

pub struct Emulator {
    rom: Vec<u8>,
    running: bool,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            running: false,
        }
    }
    pub fn reset(&mut self) {}
    pub fn step(&mut self) {}
    pub fn load_rom(&mut self, rom_path: &PathBuf) {
        let rom = std::fs::read(rom_path).unwrap();
        self.rom = rom;
    }
    pub fn run_frame(&mut self) {
        while self.running == true {
            self.step();
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
