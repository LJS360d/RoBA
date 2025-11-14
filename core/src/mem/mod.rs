pub struct Mem {
    pub vram: Vec<u8>,      // 96KB
    pub palette: Vec<u8>,   // 512 bytes (256 * u16)
    pub oam: Vec<u8>,       // 1KB
}

impl Mem {
    pub fn new() -> Self {
        Self {
            vram: vec![0u8; 96 * 1024],
            palette: vec![0u8; 512],
            oam: vec![0u8; 1024],
        }
    }
}
