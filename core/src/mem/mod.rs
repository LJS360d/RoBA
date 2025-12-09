pub const BIOS_SIZE: usize = 16 * 1024;
pub const EWRAM_SIZE: usize = 256 * 1024;
pub const IWRAM_SIZE: usize = 32 * 1024;
pub const VRAM_SIZE: usize = 96 * 1024;
pub const PALETTE_SIZE: usize = 512;
pub const OAM_SIZE: usize = 1024;
pub const ROM_MAX_SIZE: usize = 32 * 1024 * 1024;

pub struct Mem {
    pub bios: Vec<u8>,
    pub ewram: Vec<u8>,
    pub iwram: Vec<u8>,
    pub vram: Vec<u8>,
    pub palette: Vec<u8>,
    pub oam: Vec<u8>,
    pub rom: Vec<u8>,
    pub sram: Vec<u8>,
}

impl Mem {
    pub fn new() -> Self {
        Self {
            bios: vec![0u8; BIOS_SIZE],
            ewram: vec![0u8; EWRAM_SIZE],
            iwram: vec![0u8; IWRAM_SIZE],
            vram: vec![0u8; VRAM_SIZE],
            palette: vec![0u8; PALETTE_SIZE],
            oam: vec![0u8; OAM_SIZE],
            rom: Vec::new(),
            sram: vec![0u8; 64 * 1024],
        }
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        let len = data.len().min(BIOS_SIZE);
        self.bios[..len].copy_from_slice(&data[..len]);
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.rom = data.to_vec();
    }
}
