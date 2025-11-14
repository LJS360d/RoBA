#![forbid(unsafe_code)]

use std::path::PathBuf;

use crate::ppu::Ppu;
use crate::video::{framebuffer_rgb555_to_rgba, GBA_SCREEN_H, GBA_SCREEN_W};
use crate::bus::Bus;

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
    ppu: Ppu,
    rgba_frame: Vec<u8>,
    bus: Bus,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            running: false,
            ppu: Ppu::new(),
            rgba_frame: vec![0u8; GBA_SCREEN_W * GBA_SCREEN_H * 4],
            bus: Bus::new(),
        }
    }
    pub fn reset(&mut self) {}
    pub fn step(&mut self) {
        // Advance enough cycles to reach VBlank and render a frame in minimal PPU
        let cycles_to_vblank = self.ppu.cycles_until_vblank();
        self.ppu.step(cycles_to_vblank + 4);
        // Update Bus access permissions based on current PPU state
        self.bus.set_access_permissions(
            self.ppu.can_access_vram(),
            self.ppu.can_access_palette(),
            self.ppu.can_access_oam(),
        );
        // Seed palette[0] if empty to see a visible color (temporary until IO wiring)
        if self.bus.mem.palette[0] == 0 && self.bus.mem.palette[1] == 0 {
            self.bus.mem.palette[0] = 0x00;
            self.bus.mem.palette[1] = 0x7C; // RGB555 red 0x7C00 in LE
        }
        self.ppu.render_frame_with_bus(&mut self.bus);
        // Update permissions again after rendering (PPU may have changed state)
        self.bus.set_access_permissions(
            self.ppu.can_access_vram(),
            self.ppu.can_access_palette(),
            self.ppu.can_access_oam(),
        );
        framebuffer_rgb555_to_rgba(&mut self.rgba_frame, self.ppu.framebuffer());
    }
    pub fn load_rom(&mut self, rom_path: &PathBuf) {
        let rom = std::fs::read(rom_path).unwrap();
        self.rom = rom;
    }
    pub fn run_frame(&mut self) {
        // Minimal frame render
        self.step();
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu { &mut self.ppu }
    pub fn framebuffer_rgba(&self) -> &[u8] { &self.rgba_frame }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
