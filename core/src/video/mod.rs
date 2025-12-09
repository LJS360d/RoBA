#[derive(Default)]
pub struct Video;

impl Video {
    pub fn new() -> Self { Self }
}

pub const GBA_SCREEN_W: usize = 240;
pub const GBA_SCREEN_H: usize = 160;

pub fn bgr555_to_rgba8888(bgr555: u16) -> [u8; 4] {
    let r5 = (bgr555 & 0x1F) as u8;
    let g5 = ((bgr555 >> 5) & 0x1F) as u8;
    let b5 = ((bgr555 >> 10) & 0x1F) as u8;
    let r = (r5 << 3) | (r5 >> 2);
    let g = (g5 << 3) | (g5 >> 2);
    let b = (b5 << 3) | (b5 >> 2);
    [r, g, b, 0xFF]
}

pub fn framebuffer_rgb555_to_rgba(dst: &mut [u8], src_bgr555: &[u16]) {
    assert_eq!(dst.len(), src_bgr555.len() * 4);
    for (i, &px) in src_bgr555.iter().enumerate() {
        let rgba = bgr555_to_rgba8888(px);
        let o = i * 4;
        dst[o..o + 4].copy_from_slice(&rgba);
    }
}
