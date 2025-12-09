#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::video::{framebuffer_rgb555_to_rgba, GBA_SCREEN_H, GBA_SCREEN_W};
use crate::bus::Bus;

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cart;
pub mod cpu;
pub mod io;
pub mod log_buffer;
pub mod mem;
pub mod ppu;
pub mod timing;
pub mod video;

const CYCLES_PER_SCANLINE: usize = 1232;
const SCANLINES_PER_FRAME: usize = 228;
const VISIBLE_SCANLINES: usize = 160;
const HBLANK_START_CYCLE: usize = 960;

pub struct Emulator {
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
    rgba_frame: Vec<u8>,
    cycles: usize,
    frame_count: u64,
    frame_ready: bool,
    bios_loaded: bool,
    rom_loaded: bool,
}

impl Emulator {
    pub fn new() -> Self {
        log::info!("Emulator instance created");
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            bus: Bus::new(),
            rgba_frame: vec![0u8; GBA_SCREEN_W * GBA_SCREEN_H * 4],
            cycles: 0,
            frame_count: 0,
            frame_ready: false,
            bios_loaded: false,
            rom_loaded: false,
        }
    }

    pub fn reset(&mut self) {
        log::info!("Emulator reset");
        self.cpu = Cpu::new();
        self.ppu = Ppu::new();
        self.cycles = 0;
        self.frame_count = 0;
        self.frame_ready = false;

        if self.bios_loaded {
            self.cpu.set_entry_point(&mut self.bus, 0x0000_0000);
            log::info!("Entry point: BIOS (0x00000000)");
        } else if self.rom_loaded {
            self.cpu.set_entry_point(&mut self.bus, 0x0800_0000);
            log::info!("Entry point: ROM (0x08000000)");
        }
    }

    pub fn load_bios(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let data = std::fs::read(path)?;
        log::info!("BIOS loaded: {} bytes from {:?}", data.len(), path);
        self.bus.load_bios(&data);
        self.bios_loaded = true;
        self.cpu.set_entry_point(&mut self.bus, 0x0000_0000);
        Ok(())
    }

    pub fn load_rom(&mut self, rom_path: &PathBuf) {
        match std::fs::read(rom_path) {
            Ok(data) => {
                log::info!("ROM loaded: {} bytes from {:?}", data.len(), rom_path);
                self.bus.load_rom(&data);
                self.rom_loaded = true;

                if !self.bios_loaded {
                    self.init_without_bios();
                    log::info!("Entry point: ROM (0x08000000) - no BIOS");
                }
            }
            Err(e) => {
                log::error!("Failed to load ROM {:?}: {}", rom_path, e);
            }
        }
    }

    fn init_without_bios(&mut self) {
        use crate::cpu::CpuMode;

        self.cpu.set_swi_hle(true);

        self.cpu.set_mode(CpuMode::Supervisor);
        self.cpu.write_reg(13, 0x0300_7FE0);

        self.cpu.set_mode(CpuMode::Irq);
        self.cpu.write_reg(13, 0x0300_7FA0);

        self.cpu.set_mode(CpuMode::System);
        self.cpu.write_reg(13, 0x0300_7F00);

        self.cpu.set_entry_point(&mut self.bus, 0x0800_0000);
    }

    pub fn step_cpu(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn run_frame(&mut self) {
        self.frame_ready = false;

        self.bus.set_access_permissions(true, true, true);

        for scanline in 0..SCANLINES_PER_FRAME {
            self.bus.io.vcount = scanline as u16;

            let in_vblank = scanline >= VISIBLE_SCANLINES;
            let lyc = (self.bus.io.dispstat >> 8) as usize;
            let vcounter_match = scanline == lyc;

            self.bus.io.dispstat = (self.bus.io.dispstat & 0xFFF8)
                | (if in_vblank { 1 } else { 0 })
                | (if vcounter_match { 4 } else { 0 });

            for cycle_in_line in 0..CYCLES_PER_SCANLINE {
                let in_hblank = cycle_in_line >= HBLANK_START_CYCLE;
                if in_hblank {
                    self.bus.io.dispstat |= 2;
                } else {
                    self.bus.io.dispstat &= !2;
                }
                self.step_cpu();
            }
        }

        self.ppu.render_frame_with_bus(&mut self.bus);
        self.frame_ready = true;
        self.frame_count += 1;

        if self.frame_count.is_multiple_of(60) {
            log::debug!(
                "Frame {} complete | PC={:#010x} | DISPCNT={:#06x}",
                self.frame_count,
                self.cpu.read_reg(15),
                self.bus.io.dispcnt
            );
        }

        framebuffer_rgb555_to_rgba(&mut self.rgba_frame, self.ppu.framebuffer());
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu { &mut self.ppu }
    pub fn bus_mut(&mut self) -> &mut Bus { &mut self.bus }
    pub fn cpu_mut(&mut self) -> &mut Cpu { &mut self.cpu }
    pub fn framebuffer_rgba(&self) -> &[u8] { &self.rgba_frame }
    pub fn is_frame_ready(&self) -> bool { self.frame_ready }
    pub fn is_rom_loaded(&self) -> bool { self.rom_loaded }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::bus::BusAccess;

    #[test]
    fn emulator_loads_rom_and_executes() {
        let mut emu = Emulator::new();
        let rom_path = PathBuf::from("../test-roms/stripes.gba");

        if !rom_path.exists() {
            return;
        }

        emu.load_rom(&rom_path);
        assert!(emu.is_rom_loaded());

        let rom_len = emu.bus.mem.rom.len();
        assert!(rom_len > 0, "ROM should be loaded: len={}", rom_len);

        let first_instr = emu.bus.read32(0x0800_0000);
        eprintln!("First instruction at 0x08000000: {:#010x}", first_instr);
        assert_ne!(first_instr, 0, "First instruction at ROM should be readable");

        for i in 0..10 {
            let pc = emu.cpu.read_reg(15);
            let r0 = emu.cpu.read_reg(0);
            let r1 = emu.cpu.read_reg(1);
            let dispcnt_before = emu.bus.io.dispcnt;
            eprintln!("Step {}: PC={:#010x} R0={:#010x} R1={:#010x} DISPCNT={:#06x}", i, pc, r0, r1, dispcnt_before);
            emu.step_cpu();
            let dispcnt_after = emu.bus.io.dispcnt;
            if dispcnt_after != dispcnt_before {
                eprintln!("  -> DISPCNT changed to {:#06x}", dispcnt_after);
            }
        }

        eprintln!("After 10 steps:");
        eprintln!("  DISPCNT = {:#06x}", emu.bus.io.dispcnt);

        for _ in 10..1000 {
            emu.step_cpu();
        }

        let dispcnt = emu.bus.io.dispcnt;
        eprintln!("Final DISPCNT = {:#06x}", dispcnt);
        assert_ne!(dispcnt, 0, "DISPCNT should have been written by ROM code");
    }

    #[test]
    fn bus_writes_to_dispcnt() {
        let mut bus = Bus::new();
        assert_eq!(bus.io.dispcnt, 0);
        bus.write8(0x0400_0000, 0x00);
        bus.write8(0x0400_0001, 0x01);
        assert_eq!(bus.io.dispcnt, 0x0100, "DISPCNT should be 0x0100 after byte writes");

        bus.io.dispcnt = 0;
        bus.write16(0x0400_0000, 0x0203);
        assert_eq!(bus.io.dispcnt, 0x0203, "DISPCNT should be 0x0203 after u16 write");

        bus.io.dispcnt = 0;
        bus.write32(0x0400_0000, 0x0405);
        assert_eq!(bus.io.dispcnt, 0x0405, "DISPCNT should be 0x0405 after u32 write");
    }

    #[test]
    fn cpu_str_writes_to_io() {
        let mut emu = Emulator::new();

        emu.cpu.write_reg(0, 0x0100);
        emu.cpu.write_reg(1, 0x04000000);

        let str_instr: u32 = 0xe5810000;
        let nop_instr: u32 = 0xe1a00000;

        fn write_rom_word(rom: &mut Vec<u8>, offset: usize, value: u32) {
            if rom.len() < offset + 4 {
                rom.resize(offset + 4, 0);
            }
            rom[offset] = value as u8;
            rom[offset + 1] = (value >> 8) as u8;
            rom[offset + 2] = (value >> 16) as u8;
            rom[offset + 3] = (value >> 24) as u8;
        }

        write_rom_word(&mut emu.bus.mem.rom, 0, str_instr);
        write_rom_word(&mut emu.bus.mem.rom, 4, nop_instr);
        write_rom_word(&mut emu.bus.mem.rom, 8, nop_instr);

        emu.cpu.set_entry_point(&mut emu.bus, 0x0800_0000);

        let read_back = emu.bus.read32(0x0800_0000);
        eprintln!("Read back from ROM: {:#x} (expected {:#x})", read_back, str_instr);
        eprintln!("Before step: R0={:#x} R1={:#x} DISPCNT={:#x}",
            emu.cpu.read_reg(0), emu.cpu.read_reg(1), emu.bus.io.dispcnt);

        emu.step_cpu();

        eprintln!("After step: R0={:#x} R1={:#x} DISPCNT={:#x}",
            emu.cpu.read_reg(0), emu.cpu.read_reg(1), emu.bus.io.dispcnt);

        assert_eq!(emu.bus.io.dispcnt, 0x0100, "STR R0, [R1] should write to DISPCNT");
    }

    #[test]
    fn emulator_renders_something() {
        let mut emu = Emulator::new();
        let rom_path = PathBuf::from("../test-roms/stripes.gba");

        if !rom_path.exists() {
            return;
        }

        emu.load_rom(&rom_path);
        emu.run_frame();

        let fb = emu.ppu_mut().framebuffer();
        let non_zero = fb.iter().any(|&px| px != 0);
        assert!(non_zero, "Framebuffer should have some non-zero pixels");
    }

    #[test]
    fn shades_rom_renders_multiple_colors() {
        let mut emu = Emulator::new();
        let rom_path = PathBuf::from("../test-roms/shades.gba");

        if !rom_path.exists() {
            return;
        }

        emu.load_rom(&rom_path);
        emu.run_frame();

        let fb = emu.ppu_mut().framebuffer();
        let mut unique_colors: std::collections::HashSet<u16> = std::collections::HashSet::new();
        for &px in fb.iter() {
            unique_colors.insert(px);
        }

        assert!(unique_colors.len() > 1, "shades.gba should render multiple colors, got {}", unique_colors.len());
        assert!(unique_colors.len() >= 10, "Expected at least 10 colors, got {}", unique_colors.len());
    }

}
