#![forbid(unsafe_code)]

pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod bus;
pub mod mem;
pub mod cart;
pub mod timing;
pub mod io;
pub mod video;
pub mod audio;

pub struct Emulator;

impl Emulator {
    pub fn new() -> Self { Self }
    pub fn reset(&mut self) {}
    pub fn step(&mut self) {}
}
