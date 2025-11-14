use crate::mem::Mem;
use crate::io::Io;

pub trait BusAccess {
    fn read32(&mut self, addr: u32) -> u32;
    fn read16(&mut self, addr: u32) -> u16;
    fn read8(&mut self, addr: u32) -> u8;
    fn write32(&mut self, addr: u32, value: u32);
    fn write16(&mut self, addr: u32, value: u16);
    fn write8(&mut self, addr: u32, value: u8);
    fn set_ppu_rendering(&mut self, _rendering: bool) {}
}

pub struct Bus {
    pub mem: Mem,
    pub io: Io,
    ppu_rendering: bool,
    can_access_vram: bool,
    can_access_palette: bool,
    can_access_oam: bool,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            mem: Mem::new(),
            io: Io::new(),
            ppu_rendering: false,
            can_access_vram: true,
            can_access_palette: true,
            can_access_oam: true,
        }
    }

    pub fn set_ppu_rendering(&mut self, rendering: bool) {
        self.ppu_rendering = rendering;
    }

    pub fn set_access_permissions(&mut self, vram: bool, palette: bool, oam: bool) {
        self.can_access_vram = vram;
        self.can_access_palette = palette;
        self.can_access_oam = oam;
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
}
const IO_BASE: u32 = 0x0400_0000;

const PALETTE_BASE: u32 = 0x0500_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const VRAM_SIZE: usize = 96 * 1024;
const OAM_BASE: u32 = 0x0700_0000;
const OAM_SIZE: usize = 1024;
const PALETTE_SIZE: usize = 512;

impl BusAccess for Bus {
    fn read32(&mut self, addr: u32) -> u32 {
        let lo = self.read16(addr) as u32;
        let hi = self.read16(addr.wrapping_add(2)) as u32;
        (hi << 16) | lo
    }
    fn read16(&mut self, addr: u32) -> u16 {
        let a = addr & !1;
        let b0 = self.read8(a) as u16;
        let b1 = self.read8(a + 1) as u16;
        b0 | (b1 << 8)
    }
    fn read8(&mut self, addr: u32) -> u8 {
        match addr {
            a if a >= IO_BASE && a < IO_BASE + 0x400 => {
                self.io.read8(a)
            }
            a if a >= PALETTE_BASE && a < PALETTE_BASE + 0x400 => {
                if !self.check_palette_access() {
                    return 0;
                }
                let off = ((a - PALETTE_BASE) as usize) % PALETTE_SIZE;
                self.mem.palette[off]
            }
            a if a >= VRAM_BASE && a < VRAM_BASE + 0x18000 => {
                if !self.check_vram_access() {
                    return 0;
                }
                let off = ((a - VRAM_BASE) as usize) % VRAM_SIZE;
                self.mem.vram[off]
            }
            a if a >= OAM_BASE && a < OAM_BASE + 0x400 => {
                if !self.check_oam_access() {
                    return 0;
                }
                let off = ((a - OAM_BASE) as usize) % OAM_SIZE;
                self.mem.oam[off]
            }
            _ => 0,
        }
    }
    fn write32(&mut self, addr: u32, value: u32) {
        self.write16(addr, value as u16);
        self.write16(addr.wrapping_add(2), (value >> 16) as u16);
    }
    fn write16(&mut self, addr: u32, value: u16) {
        self.write8(addr, (value & 0xFF) as u8);
        self.write8(addr.wrapping_add(1), (value >> 8) as u8);
    }
    fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            a if a >= IO_BASE && a < IO_BASE + 0x400 => {
                self.io.write8(a, value);
            }
            a if a >= PALETTE_BASE && a < PALETTE_BASE + 0x400 => {
                if !self.check_palette_access() {
                    return;
                }
                let off = ((a - PALETTE_BASE) as usize) % PALETTE_SIZE;
                self.mem.palette[off] = value;
            }
            a if a >= VRAM_BASE && a < VRAM_BASE + 0x18000 => {
                if !self.check_vram_access() {
                    return;
                }
                let off = ((a - VRAM_BASE) as usize) % VRAM_SIZE;
                self.mem.vram[off] = value;
            }
            a if a >= OAM_BASE && a < OAM_BASE + 0x400 => {
                if !self.check_oam_access() {
                    return;
                }
                let off = ((a - OAM_BASE) as usize) % OAM_SIZE;
                self.mem.oam[off] = value;
            }
            _ => {}
        }
    }

    fn set_ppu_rendering(&mut self, rendering: bool) {
        Bus::set_ppu_rendering(self, rendering);
    }
}
