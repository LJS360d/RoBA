use crate::mem::Mem;
use crate::io::Io;

pub trait BusAccess {
    fn read32(&mut self, addr: u32) -> u32;
    fn read16(&mut self, addr: u32) -> u16;
    fn read8(&mut self, addr: u32) -> u8;
    fn write32(&mut self, addr: u32, value: u32);
    fn write16(&mut self, addr: u32, value: u16);
    fn write8(&mut self, addr: u32, value: u8);
}

pub struct Bus {
    pub mem: Mem,
    pub io: Io,
}

impl Bus {
    pub fn new() -> Self { Self { mem: Mem::new(), io: Io::new() } }
}
const IO_BASE: u32 = 0x0400_0000;

const PALETTE_BASE: u32 = 0x0500_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const OAM_BASE: u32 = 0x0700_0000;

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
                let off = (a - PALETTE_BASE) as usize;
                self.mem.palette[off]
            }
            a if a >= VRAM_BASE && a < VRAM_BASE + 0x18000 => {
                let off = (a - VRAM_BASE) as usize;
                self.mem.vram[off]
            }
            a if a >= OAM_BASE && a < OAM_BASE + 0x400 => {
                let off = (a - OAM_BASE) as usize;
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
                let off = (a - PALETTE_BASE) as usize;
                self.mem.palette[off] = value;
            }
            a if a >= VRAM_BASE && a < VRAM_BASE + 0x18000 => {
                let off = (a - VRAM_BASE) as usize;
                self.mem.vram[off] = value;
            }
            a if a >= OAM_BASE && a < OAM_BASE + 0x400 => {
                let off = (a - OAM_BASE) as usize;
                self.mem.oam[off] = value;
            }
            _ => {}
        }
    }
}
