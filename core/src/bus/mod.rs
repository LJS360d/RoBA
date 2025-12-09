use crate::mem::{Mem, BIOS_SIZE, EWRAM_SIZE, IWRAM_SIZE, VRAM_SIZE, PALETTE_SIZE, OAM_SIZE};
use crate::io::Io;

fn io_register_name(addr: u32) -> Option<&'static str> {
    match addr {
        0x0400_0000..=0x0400_0001 => Some("DISPCNT"),
        0x0400_0004..=0x0400_0005 => Some("DISPSTAT"),
        0x0400_0006..=0x0400_0007 => Some("VCOUNT"),
        0x0400_0008..=0x0400_0009 => Some("BG0CNT"),
        0x0400_000A..=0x0400_000B => Some("BG1CNT"),
        0x0400_000C..=0x0400_000D => Some("BG2CNT"),
        0x0400_000E..=0x0400_000F => Some("BG3CNT"),
        0x0400_004C..=0x0400_004D => Some("MOSAIC"),
        0x0400_0050..=0x0400_0051 => Some("BLDCNT"),
        0x0400_0200..=0x0400_0201 => Some("IE"),
        0x0400_0202..=0x0400_0203 => Some("IF"),
        0x0400_0208..=0x0400_0209 => Some("IME"),
        _ => None,
    }
}

pub trait BusAccess {
    fn read32(&mut self, addr: u32) -> u32;
    fn read16(&mut self, addr: u32) -> u16;
    fn read8(&mut self, addr: u32) -> u8;
    fn write32(&mut self, addr: u32, value: u32);
    fn write16(&mut self, addr: u32, value: u16);
    fn write8(&mut self, addr: u32, value: u8);
    fn set_ppu_rendering(&mut self, _rendering: bool) {}
}

const EWRAM_BASE: u32 = 0x0200_0000;
const IWRAM_BASE: u32 = 0x0300_0000;
const IO_BASE: u32 = 0x0400_0000;
const PALETTE_BASE: u32 = 0x0500_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const OAM_BASE: u32 = 0x0700_0000;
const SRAM_BASE: u32 = 0x0E00_0000;

pub struct Bus {
    pub mem: Mem,
    pub io: Io,
    ppu_rendering: bool,
    can_access_vram: bool,
    can_access_palette: bool,
    can_access_oam: bool,
    bios_readable: bool,
    last_bios_read: u32,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            mem: Mem::new(),
            io: Io::new(),
            ppu_rendering: false,
            can_access_vram: true,
            can_access_palette: true,
            can_access_oam: true,
            bios_readable: true,
            last_bios_read: 0,
        }
    }
}

impl Bus {
    pub fn new() -> Self { Self::default() }

    pub fn set_ppu_rendering(&mut self, rendering: bool) {
        self.ppu_rendering = rendering;
    }

    pub fn set_access_permissions(&mut self, vram: bool, palette: bool, oam: bool) {
        self.can_access_vram = vram;
        self.can_access_palette = palette;
        self.can_access_oam = oam;
    }

    pub fn set_bios_readable(&mut self, readable: bool) {
        self.bios_readable = readable;
    }

    fn check_vram_access(&self) -> bool {
        self.ppu_rendering || self.can_access_vram
    }

    fn check_palette_access(&self) -> bool {
        self.ppu_rendering || self.can_access_palette
    }

    fn check_oam_access(&self) -> bool {
        self.ppu_rendering || self.can_access_oam
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        log::info!("Bus: loading BIOS ({} bytes)", data.len());
        self.mem.load_bios(data);
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        log::info!("Bus: loading ROM ({} bytes, {} KB)", data.len(), data.len() / 1024);
        self.mem.load_rom(data);
    }
}

impl BusAccess for Bus {
    fn read32(&mut self, addr: u32) -> u32 {
        let aligned = addr & !3;
        let lo = self.read16(aligned) as u32;
        let hi = self.read16(aligned.wrapping_add(2)) as u32;
        let value = lo | (hi << 16);
        let rotation = (addr & 3) * 8;
        value.rotate_right(rotation)
    }

    fn read16(&mut self, addr: u32) -> u16 {
        let aligned = addr & !1;
        let b0 = self.read8(aligned) as u16;
        let b1 = self.read8(aligned + 1) as u16;
        let value = b0 | (b1 << 8);
        if addr & 1 != 0 {
            value.rotate_right(8)
        } else {
            value
        }
    }

    fn read8(&mut self, addr: u32) -> u8 {
        match addr >> 24 {
            0x00 => {
                if addr < BIOS_SIZE as u32 {
                    if self.bios_readable {
                        let v = self.mem.bios[addr as usize];
                        self.last_bios_read = self.read32_direct_bios(addr & !3);
                        v
                    } else {
                        ((self.last_bios_read >> ((addr & 3) * 8)) & 0xFF) as u8
                    }
                } else {
                    0
                }
            }
            0x02 => {
                let off = ((addr - EWRAM_BASE) as usize) % EWRAM_SIZE;
                self.mem.ewram[off]
            }
            0x03 => {
                let off = ((addr - IWRAM_BASE) as usize) % IWRAM_SIZE;
                self.mem.iwram[off]
            }
            0x04 => {
                if addr < IO_BASE + 0x400 {
                    self.io.read8(addr)
                } else {
                    0
                }
            }
            0x05 => {
                if !self.check_palette_access() {
                    return 0;
                }
                let off = ((addr - PALETTE_BASE) as usize) % PALETTE_SIZE;
                self.mem.palette[off]
            }
            0x06 => {
                if !self.check_vram_access() {
                    return 0;
                }
                let raw_off = (addr - VRAM_BASE) as usize;
                let off = if raw_off >= 0x18000 {
                    0x10000 + ((raw_off - 0x10000) % 0x8000)
                } else {
                    raw_off % VRAM_SIZE
                };
                self.mem.vram[off]
            }
            0x07 => {
                if !self.check_oam_access() {
                    return 0;
                }
                let off = ((addr - OAM_BASE) as usize) % OAM_SIZE;
                self.mem.oam[off]
            }
            0x08..=0x0D => {
                let off = (addr & 0x01FF_FFFF) as usize;
                if off < self.mem.rom.len() {
                    self.mem.rom[off]
                } else {
                    let halfword_idx = (addr >> 1) as u16;
                    ((halfword_idx >> ((addr & 1) * 8)) & 0xFF) as u8
                }
            }
            0x0E | 0x0F => {
                let off = ((addr - SRAM_BASE) as usize) % self.mem.sram.len();
                self.mem.sram[off]
            }
            _ => 0,
        }
    }

    fn write32(&mut self, addr: u32, value: u32) {
        let aligned = addr & !3;
        self.write16(aligned, value as u16);
        self.write16(aligned.wrapping_add(2), (value >> 16) as u16);
    }

    fn write16(&mut self, addr: u32, value: u16) {
        let aligned = addr & !1;
        self.write8(aligned, (value & 0xFF) as u8);
        self.write8(aligned.wrapping_add(1), (value >> 8) as u8);
    }

    fn write8(&mut self, addr: u32, value: u8) {
        match addr >> 24 {
            0x00 => {}
            0x02 => {
                let off = ((addr - EWRAM_BASE) as usize) % EWRAM_SIZE;
                self.mem.ewram[off] = value;
            }
            0x03 => {
                let off = ((addr - IWRAM_BASE) as usize) % IWRAM_SIZE;
                self.mem.iwram[off] = value;
            }
            0x04 => {
                if addr < IO_BASE + 0x400 {
                    if let Some(name) = io_register_name(addr) {
                        log::trace!("IO write8 {} ({:#010x}) = {:#04x}", name, addr, value);
                    }
                    self.io.write8(addr, value);
                }
            }
            0x05 => {
                if !self.check_palette_access() {
                    return;
                }
                let off = ((addr - PALETTE_BASE) as usize) % PALETTE_SIZE;
                self.mem.palette[off] = value;
            }
            0x06 => {
                if !self.check_vram_access() {
                    return;
                }
                let raw_off = (addr - VRAM_BASE) as usize;
                let off = if raw_off >= 0x18000 {
                    0x10000 + ((raw_off - 0x10000) % 0x8000)
                } else {
                    raw_off % VRAM_SIZE
                };
                self.mem.vram[off] = value;
            }
            0x07 => {
                if !self.check_oam_access() {
                    return;
                }
                let off = ((addr - OAM_BASE) as usize) % OAM_SIZE;
                self.mem.oam[off] = value;
            }
            0x08..=0x0D => {}
            0x0E | 0x0F => {
                let off = ((addr - SRAM_BASE) as usize) % self.mem.sram.len();
                self.mem.sram[off] = value;
            }
            _ => {}
        }
    }

    fn set_ppu_rendering(&mut self, rendering: bool) {
        Bus::set_ppu_rendering(self, rendering);
    }
}

impl Bus {
    fn read32_direct_bios(&self, addr: u32) -> u32 {
        if addr as usize + 3 < self.mem.bios.len() {
            let b0 = self.mem.bios[addr as usize] as u32;
            let b1 = self.mem.bios[(addr + 1) as usize] as u32;
            let b2 = self.mem.bios[(addr + 2) as usize] as u32;
            let b3 = self.mem.bios[(addr + 3) as usize] as u32;
            b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
        } else {
            0
        }
    }
}
