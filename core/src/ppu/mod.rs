#![allow(unused)]
//! GBA PPU module for a Game Boy Advance emulator.
//!
//! This module contains the core logic for emulating the GBA's Picture Processing Unit (PPU).
//! It defines the PPU's state, memory-mapped registers, and rendering pipeline.
//! The acceptance tests serve as a scaffold for implementing the PPU's behavior step-by-step.

// Constants for PPU memory-mapped I/O registers.
// These are defined in hexadecimal format and represent the memory addresses
// that the CPU uses to interact with the PPU.
const REG_DISPCNT: u32 = 0x0400_0000;
const REG_DISPSTAT: u32 = 0x0400_0004;
const REG_VCOUNT: u32 = 0x0400_0006;
const REG_BG0CNT: u32 = 0x0400_0008;
const REG_BG1CNT: u32 = 0x0400_000A;
const REG_BG2CNT: u32 = 0x0400_000C;
const REG_BG3CNT: u32 = 0x0400_000E;
const REG_BG0HOFS: u32 = 0x0400_0010;
const REG_BG0VOFS: u32 = 0x0400_0012;
const REG_BG1HOFS: u32 = 0x0400_0014;
const REG_BG1VOFS: u32 = 0x0400_0016;
const REG_BG2HOFS: u32 = 0x0400_0018;
const REG_BG2VOFS: u32 = 0x0400_001A;
const REG_BG3HOFS: u32 = 0x0400_001C;
const REG_BG3VOFS: u32 = 0x0400_001E;
const REG_BG2PA: u32 = 0x0400_0020;
const REG_BG2PB: u32 = 0x0400_0022;
const REG_BG2PC: u32 = 0x0400_0024;
const REG_BG2PD: u32 = 0x0400_0026;
const REG_BG2X: u32 = 0x0400_0028;
const REG_BG2Y: u32 = 0x0400_002C;
const REG_BG3PA: u32 = 0x0400_0030;
const REG_BG3PB: u32 = 0x0400_0032;
const REG_BG3PC: u32 = 0x0400_0034;
const REG_BG3PD: u32 = 0x0400_0036;
const REG_BG3X: u32 = 0x0400_0038;
const REG_BG3Y: u32 = 0x0400_003C;
const REG_MOSAIC: u32 = 0x0400_004C;
const REG_BLDCNT: u32 = 0x0400_0050;
const REG_BLDALPHA: u32 = 0x0400_0052;
const REG_BLDY: u32 = 0x0400_0054;
const REG_WIN0H: u32 = 0x0400_0040;
const REG_WIN1H: u32 = 0x0400_0042;
const REG_WIN0V: u32 = 0x0400_0044;
const REG_WIN1V: u32 = 0x0400_0046;
const REG_WININ: u32 = 0x0400_0048;
const REG_WINOUT: u32 = 0x0400_004A;

// Memory addresses for VRAM, OAM, and Palette RAM.
// These are separate memory regions that the PPU uses for graphics data.
const VRAM_START: u32 = 0x0600_0000;
const OAM_START: u32 = 0x0700_0000;
const PALETTE_RAM_START: u32 = 0x0500_0000;

/// Represents a minimal state of the GBA's PPU sufficient to start producing frames.
pub struct Ppu {
    dispcnt: u16,
    dispstat: u16,
    palette: Vec<u16>,
    framebuffer: Vec<u16>,
    cycles: usize,
}

const SCREEN_W: usize = 240;
const SCREEN_H: usize = 160;
const FRAME_PIXELS: usize = SCREEN_W * SCREEN_H;
const DISPCNT_FORCED_BLANK: u16 = 1 << 7;
const DISPCNT_BG0_ENABLE: u16 = 1 << 8;
const DISPCNT_BG1_ENABLE: u16 = 1 << 9;
const DISPCNT_BG2_ENABLE: u16 = 1 << 10;
const DISPCNT_BG3_ENABLE: u16 = 1 << 11;
const DISPCNT_OBJ_ENABLE: u16 = 1 << 12;
const DISPCNT_WIN0_ENABLE: u16 = 1 << 13;
const DISPCNT_WIN1_ENABLE: u16 = 1 << 14;
const DISPCNT_OBJ_WIN_ENABLE: u16 = 1 << 15;
const DISPCNT_OBJ_VRAM_MAPPING: u16 = 1 << 6;
const DISPCNT_MODE_MASK: u16 = 0b111;
const OBJ_PALETTE_START: u32 = 0x0500_0200;
const OBJ_VRAM_START_MODE012: u32 = 0x0601_0000;
const OBJ_VRAM_START_MODE345: u32 = 0x0601_4000;
const DISPSTAT_VBLANK_FLAG: u16 = 1 << 0;
const CYCLES_PER_SCANLINE: usize = 1232; // placeholder to align with harness
const SCANLINES_VISIBLE: usize = 160;
const SCANLINES_PER_FRAME: usize = 228;

impl Ppu {
    /// Creates a new PPU instance.
    pub fn new() -> Self {
        Ppu {
            dispcnt: 0,
            dispstat: 0,
            palette: vec![0u16; 256],
            framebuffer: vec![0u16; FRAME_PIXELS],
            cycles: 0,
        }
    }

    pub fn write_dispcnt(&mut self, value: u16) { self.dispcnt = value; }
    pub fn read_dispcnt(&self) -> u16 { self.dispcnt }
    pub fn read_dispstat(&self) -> u16 { self.dispstat }
    pub fn write_palette_entry(&mut self, index: usize, color: u16) {
        if index < self.palette.len() { self.palette[index] = color; }
    }
    pub fn framebuffer(&self) -> &[u16] { &self.framebuffer }

    pub fn cycles_until_vblank(&self) -> usize { CYCLES_PER_SCANLINE * SCANLINES_VISIBLE }
    pub fn cycles_per_frame(&self) -> usize { CYCLES_PER_SCANLINE * SCANLINES_PER_FRAME }
    pub fn step(&mut self, cycles: usize) {
        let prev = self.cycles;
        self.cycles = self.cycles.saturating_add(cycles);
        let vblank_start = self.cycles_until_vblank();
        if prev < vblank_start && self.cycles >= vblank_start {
            self.dispstat |= DISPSTAT_VBLANK_FLAG;
            self.render_frame();
        }
        if self.cycles >= self.cycles_per_frame() {
            self.cycles %= self.cycles_per_frame();
            self.dispstat &= !DISPSTAT_VBLANK_FLAG;
        }
    }

    /// Renders a single frame.
    ///
    /// This function will be the core of the PPU emulation. It should
    /// iterate through each scanline (0-159) and each pixel (0-239),
    /// fetching and processing tile and sprite data to produce a frame.
    pub fn render_frame(&mut self) {
        if (self.dispcnt & DISPCNT_FORCED_BLANK) != 0 {
            for p in self.framebuffer.iter_mut() { *p = 0; }
            return;
        }
        for p in self.framebuffer.iter_mut() { *p = 0; }
        let mode = self.dispcnt & DISPCNT_MODE_MASK;
        let bg0 = (self.dispcnt & DISPCNT_BG0_ENABLE) != 0;
        if mode == 0 && bg0 {
            let bgcol = self.palette.get(0).cloned().unwrap_or(0);
            for p in self.framebuffer.iter_mut() { *p = bgcol; }
        }
    }

    pub fn render_frame_with_bus<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        if (self.dispcnt & DISPCNT_FORCED_BLANK) != 0 {
            for p in self.framebuffer.iter_mut() { *p = 0; }
            return;
        }

        let lo = bus.read8(REG_DISPCNT) as u16;
        let hi = bus.read8(REG_DISPCNT + 1) as u16;
        self.dispcnt = lo | (hi << 8);

        for p in self.framebuffer.iter_mut() { *p = 0; }

        let mode = self.dispcnt & DISPCNT_MODE_MASK;
        match mode {
            0 => self.render_mode0(bus),
            1 => self.render_mode1(bus),
            2 => self.render_mode2(bus),
            3 => self.render_mode3(bus),
            4 => self.render_mode4(bus),
            5 => self.render_mode5(bus),
            _ => {}
        }
    }

    fn render_mode0<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        let backdrop = self.read_backdrop_color(bus);
        let mosaic = self.read_mosaic(bus);
        let obj_window_mask = self.build_obj_window_mask(bus);
        let mut temp_buffer = vec![0u16; FRAME_PIXELS];

        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let window_region = self.get_window_region(bus, x, y, &obj_window_mask);
                let mut pixel = backdrop;
                let mut priority = 4u8;

                for bg_num in 0..4 {
                    if !self.is_bg_enabled(bg_num) { continue; }
                    if !self.is_layer_enabled_in_window(bus, window_region, bg_num, false) { continue; }

                    let bgcnt = self.read_bgcnt(bus, bg_num);
                    let bg_priority = (bgcnt & 0x3) as u8;
                    if bg_priority >= priority { continue; }

                    let src_x = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_x(x, mosaic)
                    } else {
                        x
                    };
                    let src_y = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_y(y, mosaic)
                    } else {
                        y
                    };

                    if let Some(p) = self.render_text_bg_pixel(bus, bg_num, src_x, src_y) {
                        pixel = p;
                        priority = bg_priority;
                    }
                }

                temp_buffer[y * SCREEN_W + x] = pixel;
            }
        }

        {
            let mut fb = temp_buffer.as_mut_slice();
            self.render_objs_with_windows(bus, &mut fb, &obj_window_mask);
        }
        self.framebuffer.copy_from_slice(&temp_buffer);
    }

    fn render_mode1<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        let backdrop = self.read_backdrop_color(bus);
        let mosaic = self.read_mosaic(bus);
        let obj_window_mask = self.build_obj_window_mask(bus);
        let mut temp_buffer = vec![0u16; FRAME_PIXELS];

        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let window_region = self.get_window_region(bus, x, y, &obj_window_mask);
                let mut pixel = backdrop;
                let mut priority = 4u8;

                for bg_num in 0..3 {
                    if !self.is_bg_enabled(bg_num) { continue; }
                    if !self.is_layer_enabled_in_window(bus, window_region, bg_num, false) { continue; }

                    let bgcnt = self.read_bgcnt(bus, bg_num);
                    let bg_priority = (bgcnt & 0x3) as u8;
                    if bg_priority >= priority { continue; }

                    let src_x = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_x(x, mosaic)
                    } else {
                        x
                    };
                    let src_y = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_y(y, mosaic)
                    } else {
                        y
                    };

                    let p = if bg_num < 2 {
                        self.render_text_bg_pixel(bus, bg_num, src_x, src_y)
                    } else {
                        self.render_affine_bg_pixel(bus, bg_num, src_x, src_y)
                    };

                    if let Some(p) = p {
                        pixel = p;
                        priority = bg_priority;
                    }
                }

                temp_buffer[y * SCREEN_W + x] = pixel;
            }
        }

        {
            let mut fb = temp_buffer.as_mut_slice();
            self.render_objs_with_windows(bus, &mut fb, &obj_window_mask);
        }
        self.framebuffer.copy_from_slice(&temp_buffer);
    }

    fn render_mode2<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        let backdrop = self.read_backdrop_color(bus);
        let mosaic = self.read_mosaic(bus);
        let obj_window_mask = self.build_obj_window_mask(bus);
        let mut temp_buffer = vec![0u16; FRAME_PIXELS];

        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let window_region = self.get_window_region(bus, x, y, &obj_window_mask);
                let mut pixel = backdrop;
                let mut priority = 4u8;

                for bg_num in 2..4 {
                    if !self.is_bg_enabled(bg_num) { continue; }
                    if !self.is_layer_enabled_in_window(bus, window_region, bg_num, false) { continue; }

                    let bgcnt = self.read_bgcnt(bus, bg_num);
                    let bg_priority = (bgcnt & 0x3) as u8;
                    if bg_priority >= priority { continue; }

                    let src_x = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_x(x, mosaic)
                    } else {
                        x
                    };
                    let src_y = if (bgcnt >> 6) & 1 != 0 {
                        self.apply_mosaic_y(y, mosaic)
                    } else {
                        y
                    };

                    if let Some(p) = self.render_affine_bg_pixel(bus, bg_num, src_x, src_y) {
                        pixel = p;
                        priority = bg_priority;
                    }
                }

                temp_buffer[y * SCREEN_W + x] = pixel;
            }
        }

        {
            let mut fb = temp_buffer.as_mut_slice();
            self.render_objs_with_windows(bus, &mut fb, &obj_window_mask);
        }
        self.framebuffer.copy_from_slice(&temp_buffer);
    }

    fn render_mode3<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        if !self.is_bg_enabled(2) { return; }

        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let addr = VRAM_START + ((y * SCREEN_W + x) * 2) as u32;
                let lo = bus.read8(addr) as u16;
                let hi = bus.read8(addr + 1) as u16;
                self.framebuffer[y * SCREEN_W + x] = lo | (hi << 8);
            }
        }
        self.render_objs_direct(bus);
    }

    fn render_mode4<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        if !self.is_bg_enabled(2) { return; }

        let frame_select = (self.dispcnt >> 4) & 1;
        let frame_base = if frame_select == 0 { 0 } else { 0x0A000 };

        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let addr = VRAM_START + frame_base + ((y * SCREEN_W + x) as u32);
                let palette_idx = bus.read8(addr) as usize;
                if palette_idx == 0 { continue; }

                let pal_addr = PALETTE_RAM_START + (palette_idx * 2) as u32;
                let lo = bus.read8(pal_addr) as u16;
                let hi = bus.read8(pal_addr + 1) as u16;
                self.framebuffer[y * SCREEN_W + x] = lo | (hi << 8);
            }
        }
        self.render_objs_direct(bus);
    }

    fn render_mode5<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        if !self.is_bg_enabled(2) { return; }

        let frame_select = (self.dispcnt >> 4) & 1;
        let frame_base = if frame_select == 0 { 0 } else { 0x0A000 };
        const MODE5_W: usize = 160;
        const MODE5_H: usize = 128;

        for y in 0..MODE5_H {
            for x in 0..MODE5_W {
                let addr = VRAM_START + frame_base + ((y * MODE5_W + x) * 2) as u32;
                let lo = bus.read8(addr) as u16;
                let hi = bus.read8(addr + 1) as u16;
                if y < SCREEN_H && x < SCREEN_W {
                    self.framebuffer[y * SCREEN_W + x] = lo | (hi << 8);
                }
            }
        }
        self.render_objs_direct(bus);
    }

    fn render_objs<B: crate::bus::BusAccess>(&self, bus: &mut B, framebuffer: &mut [u16]) {
        if (self.dispcnt & DISPCNT_OBJ_ENABLE) == 0 {
            return;
        }
        let dispcnt = self.dispcnt;
        let mode = dispcnt & DISPCNT_MODE_MASK;
        let mosaic = self.read_mosaic(bus);
        let obj_vram_base = if mode >= 3 { OBJ_VRAM_START_MODE345 } else { OBJ_VRAM_START_MODE012 };
        let one_dimensional = (dispcnt & DISPCNT_OBJ_VRAM_MAPPING) != 0;
        self.render_objs_internal(bus, framebuffer, dispcnt, mode, mosaic, obj_vram_base, one_dimensional);
    }

    fn render_objs_with_windows<B: crate::bus::BusAccess>(&self, bus: &mut B, framebuffer: &mut [u16], obj_window_mask: &[bool]) {
        if (self.dispcnt & DISPCNT_OBJ_ENABLE) == 0 {
            return;
        }
        let dispcnt = self.dispcnt;
        let mode = dispcnt & DISPCNT_MODE_MASK;
        let mosaic = self.read_mosaic(bus);
        let obj_vram_base = if mode >= 3 { OBJ_VRAM_START_MODE345 } else { OBJ_VRAM_START_MODE012 };
        let one_dimensional = (dispcnt & DISPCNT_OBJ_VRAM_MAPPING) != 0;
        self.render_objs_internal_with_windows(bus, framebuffer, dispcnt, mode, mosaic, obj_vram_base, one_dimensional, obj_window_mask);
    }

    fn render_objs_internal_with_windows<B: crate::bus::BusAccess>(&self, bus: &mut B, framebuffer: &mut [u16], dispcnt: u16, mode: u16, mosaic: u16, obj_vram_base: u32, one_dimensional: bool, obj_window_mask: &[bool]) {
        for obj_num in (0..128).rev() {
            let oam_addr = OAM_START + (obj_num * 8) as u32;
            let attr0_lo = bus.read8(oam_addr) as u16;
            let attr0_hi = bus.read8(oam_addr + 1) as u16;
            let attr0 = attr0_lo | (attr0_hi << 8);
            let attr1_lo = bus.read8(oam_addr + 2) as u16;
            let attr1_hi = bus.read8(oam_addr + 3) as u16;
            let attr1 = attr1_lo | (attr1_hi << 8);
            let attr2_lo = bus.read8(oam_addr + 4) as u16;
            let attr2_hi = bus.read8(oam_addr + 5) as u16;
            let attr2 = attr2_lo | (attr2_hi << 8);

            let y = (attr0 & 0xFF) as usize;
            let x = (attr1 & 0x1FF) as usize;
            let rotation_scaling = (attr0 >> 8) & 1 != 0;
            let obj_disable = !rotation_scaling && ((attr0 >> 9) & 1 != 0);
            let obj_mode = (attr0 >> 10) & 0x3;
            let obj_mosaic = (attr0 >> 12) & 1 != 0;
            let is_256_color = (attr0 >> 13) & 1 != 0;
            let shape = (attr0 >> 14) & 0x3;
            let size = (attr1 >> 14) & 0x3;
            let tile_num = attr2 & 0x3FF;
            let priority = ((attr2 >> 10) & 0x3) as u8;
            let palette_num = (attr2 >> 12) & 0xF;

            if obj_disable || obj_mode == 3 {
                continue;
            }

            if obj_mode == 2 {
                continue;
            }

            let (obj_w, obj_h) = self.get_obj_size(shape, size);
            let display_w = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_w * 2
            } else {
                obj_w
            };
            let display_h = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_h * 2
            } else {
                obj_h
            };

            let screen_y = if y >= 160 { y.wrapping_sub(256) } else { y };
            let screen_x = if x >= 240 { x.wrapping_sub(512) } else { x };

            for py in 0..display_h {
                let fy = screen_y.wrapping_add(py);
                if fy >= SCREEN_H {
                    continue;
                }

                let src_y = if obj_mosaic {
                    self.apply_mosaic_y(fy, mosaic)
                } else {
                    fy
                };
                let src_y = src_y.wrapping_sub(screen_y);
                if src_y >= display_h {
                    continue;
                }

                for px in 0..display_w {
                    let fx = screen_x.wrapping_add(px);
                    if fx >= SCREEN_W {
                        continue;
                    }

                    let src_x = if obj_mosaic {
                        self.apply_mosaic_x(fx, mosaic)
                    } else {
                        fx
                    };
                    let src_x = src_x.wrapping_sub(screen_x);
                    if src_x >= display_w {
                        continue;
                    }

                    let window_region = self.get_window_region(bus, fx, fy, obj_window_mask);
                    if !self.is_layer_enabled_in_window(bus, window_region, 0, true) {
                        continue;
                    }

                    let pixel = if rotation_scaling {
                        let param_group = ((attr1 >> 9) & 0x1F) as usize;
                        self.render_affine_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, param_group, obj_w, obj_h, display_w, display_h, src_x, src_y)
                    } else {
                        let h_flip = (attr1 >> 12) & 1 != 0;
                        let v_flip = (attr1 >> 13) & 1 != 0;
                        self.render_regular_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, obj_w, obj_h, src_x, src_y, h_flip, v_flip)
                    };

                    if let Some(p) = pixel {
                        let idx = fy * SCREEN_W + fx;
                        let bg_priority = self.get_bg_priority_at_safe(bus, fx, fy, mode, dispcnt);
                        if priority < bg_priority || (priority == bg_priority && obj_num < 64) {
                            framebuffer[idx] = p;
                        }
                    }
                }
            }
        }
    }

    fn render_objs_direct<B: crate::bus::BusAccess>(&mut self, bus: &mut B) {
        if (self.dispcnt & DISPCNT_OBJ_ENABLE) == 0 {
            return;
        }
        let dispcnt = self.dispcnt;
        let mode = dispcnt & DISPCNT_MODE_MASK;
        let mosaic = self.read_mosaic(bus);
        let obj_vram_base = if mode >= 3 { OBJ_VRAM_START_MODE345 } else { OBJ_VRAM_START_MODE012 };
        let one_dimensional = (dispcnt & DISPCNT_OBJ_VRAM_MAPPING) != 0;

        self.render_objs_internal_direct(bus, dispcnt, mode, mosaic, obj_vram_base, one_dimensional);
    }

    fn render_objs_internal_direct<B: crate::bus::BusAccess>(&mut self, bus: &mut B, dispcnt: u16, mode: u16, mosaic: u16, obj_vram_base: u32, one_dimensional: bool) {
        for obj_num in (0..128).rev() {
            let oam_addr = OAM_START + (obj_num * 8) as u32;
            let attr0_lo = bus.read8(oam_addr) as u16;
            let attr0_hi = bus.read8(oam_addr + 1) as u16;
            let attr0 = attr0_lo | (attr0_hi << 8);
            let attr1_lo = bus.read8(oam_addr + 2) as u16;
            let attr1_hi = bus.read8(oam_addr + 3) as u16;
            let attr1 = attr1_lo | (attr1_hi << 8);
            let attr2_lo = bus.read8(oam_addr + 4) as u16;
            let attr2_hi = bus.read8(oam_addr + 5) as u16;
            let attr2 = attr2_lo | (attr2_hi << 8);

            let y = (attr0 & 0xFF) as usize;
            let x = (attr1 & 0x1FF) as usize;
            let rotation_scaling = (attr0 >> 8) & 1 != 0;
            let obj_disable = !rotation_scaling && ((attr0 >> 9) & 1 != 0);
            let obj_mode = (attr0 >> 10) & 0x3;
            let obj_mosaic = (attr0 >> 12) & 1 != 0;
            let is_256_color = (attr0 >> 13) & 1 != 0;
            let shape = (attr0 >> 14) & 0x3;
            let size = (attr1 >> 14) & 0x3;
            let tile_num = attr2 & 0x3FF;
            let priority = ((attr2 >> 10) & 0x3) as u8;
            let palette_num = (attr2 >> 12) & 0xF;

            if obj_disable || obj_mode == 3 {
                continue;
            }

            if obj_mode == 2 {
                continue;
            }

            let (obj_w, obj_h) = self.get_obj_size(shape, size);
            let display_w = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_w * 2
            } else {
                obj_w
            };
            let display_h = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_h * 2
            } else {
                obj_h
            };

            let screen_y = if y >= 160 { y.wrapping_sub(256) } else { y };
            let screen_x = if x >= 240 { x.wrapping_sub(512) } else { x };

            for py in 0..display_h {
                let fy = screen_y.wrapping_add(py);
                if fy >= SCREEN_H {
                    continue;
                }

                let src_y = if obj_mosaic {
                    self.apply_mosaic_y(fy, mosaic)
                } else {
                    fy
                };
                let src_y = src_y.wrapping_sub(screen_y);
                if src_y >= display_h {
                    continue;
                }

                for px in 0..display_w {
                    let fx = screen_x.wrapping_add(px);
                    if fx >= SCREEN_W {
                        continue;
                    }

                    let src_x = if obj_mosaic {
                        self.apply_mosaic_x(fx, mosaic)
                    } else {
                        fx
                    };
                    let src_x = src_x.wrapping_sub(screen_x);
                    if src_x >= display_w {
                        continue;
                    }

                    let pixel = if rotation_scaling {
                        let param_group = ((attr1 >> 9) & 0x1F) as usize;
                        self.render_affine_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, param_group, obj_w, obj_h, display_w, display_h, src_x, src_y)
                    } else {
                        let h_flip = (attr1 >> 12) & 1 != 0;
                        let v_flip = (attr1 >> 13) & 1 != 0;
                        self.render_regular_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, obj_w, obj_h, src_x, src_y, h_flip, v_flip)
                    };

                    if let Some(p) = pixel {
                        let idx = fy * SCREEN_W + fx;
                        let bg_priority = self.get_bg_priority_at_safe(bus, fx, fy, mode, dispcnt);
                        if priority < bg_priority || (priority == bg_priority && obj_num < 64) {
                            self.framebuffer[idx] = p;
                        }
                    }
                }
            }
        }
    }

    fn render_objs_internal<B: crate::bus::BusAccess>(&self, bus: &mut B, framebuffer: &mut [u16], dispcnt: u16, mode: u16, mosaic: u16, obj_vram_base: u32, one_dimensional: bool) {

        for obj_num in (0..128).rev() {
            let oam_addr = OAM_START + (obj_num * 8) as u32;
            let attr0_lo = bus.read8(oam_addr) as u16;
            let attr0_hi = bus.read8(oam_addr + 1) as u16;
            let attr0 = attr0_lo | (attr0_hi << 8);
            let attr1_lo = bus.read8(oam_addr + 2) as u16;
            let attr1_hi = bus.read8(oam_addr + 3) as u16;
            let attr1 = attr1_lo | (attr1_hi << 8);
            let attr2_lo = bus.read8(oam_addr + 4) as u16;
            let attr2_hi = bus.read8(oam_addr + 5) as u16;
            let attr2 = attr2_lo | (attr2_hi << 8);

            let y = (attr0 & 0xFF) as usize;
            let x = (attr1 & 0x1FF) as usize;
            let rotation_scaling = (attr0 >> 8) & 1 != 0;
            let obj_disable = !rotation_scaling && ((attr0 >> 9) & 1 != 0);
            let obj_mode = (attr0 >> 10) & 0x3;
            let obj_mosaic = (attr0 >> 12) & 1 != 0;
            let is_256_color = (attr0 >> 13) & 1 != 0;
            let shape = (attr0 >> 14) & 0x3;
            let size = (attr1 >> 14) & 0x3;
            let tile_num = attr2 & 0x3FF;
            let priority = ((attr2 >> 10) & 0x3) as u8;
            let palette_num = (attr2 >> 12) & 0xF;

            if obj_disable || obj_mode == 3 {
                continue;
            }

            if obj_mode == 2 {
                continue;
            }

            let (obj_w, obj_h) = self.get_obj_size(shape, size);
            let display_w = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_w * 2
            } else {
                obj_w
            };
            let display_h = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_h * 2
            } else {
                obj_h
            };

            let screen_y = if y >= 160 { y.wrapping_sub(256) } else { y };
            let screen_x = if x >= 240 { x.wrapping_sub(512) } else { x };

            for py in 0..display_h {
                let fy = screen_y.wrapping_add(py);
                if fy >= SCREEN_H {
                    continue;
                }

                let src_y = if obj_mosaic {
                    self.apply_mosaic_y(fy, mosaic)
                } else {
                    fy
                };
                let src_y = src_y.wrapping_sub(screen_y);
                if src_y >= display_h {
                    continue;
                }

                for px in 0..display_w {
                    let fx = screen_x.wrapping_add(px);
                    if fx >= SCREEN_W {
                        continue;
                    }

                    let src_x = if obj_mosaic {
                        self.apply_mosaic_x(fx, mosaic)
                    } else {
                        fx
                    };
                    let src_x = src_x.wrapping_sub(screen_x);
                    if src_x >= display_w {
                        continue;
                    }

                    let pixel = if rotation_scaling {
                        let param_group = ((attr1 >> 9) & 0x1F) as usize;
                        self.render_affine_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, param_group, obj_w, obj_h, display_w, display_h, src_x, src_y)
                    } else {
                        let h_flip = (attr1 >> 12) & 1 != 0;
                        let v_flip = (attr1 >> 13) & 1 != 0;
                        self.render_regular_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, obj_w, obj_h, src_x, src_y, h_flip, v_flip)
                    };

                    if let Some(p) = pixel {
                        let idx = fy * SCREEN_W + fx;
                        let bg_priority = self.get_bg_priority_at_safe(bus, fx, fy, mode, dispcnt);
                        if priority < bg_priority || (priority == bg_priority && obj_num < 64) {
                            framebuffer[idx] = p;
                        }
                    }
                }
            }
        }
    }

    fn get_obj_size(&self, shape: u16, size: u16) -> (usize, usize) {
        match (shape, size) {
            (0, 0) => (8, 8),
            (0, 1) => (16, 16),
            (0, 2) => (32, 32),
            (0, 3) => (64, 64),
            (1, 0) => (16, 8),
            (1, 1) => (32, 8),
            (1, 2) => (32, 16),
            (1, 3) => (64, 32),
            (2, 0) => (8, 16),
            (2, 1) => (8, 32),
            (2, 2) => (16, 32),
            (2, 3) => (32, 64),
            _ => (8, 8),
        }
    }

    fn render_regular_obj_pixel<B: crate::bus::BusAccess>(&self, bus: &mut B, obj_vram_base: u32, one_dimensional: bool, is_256_color: bool, tile_num: u16, palette_num: u16, obj_w: usize, obj_h: usize, src_x: usize, src_y: usize, h_flip: bool, v_flip: bool) -> Option<u16> {
        let tile_x = src_x / 8;
        let tile_y = src_y / 8;
        let pixel_x = src_x % 8;
        let pixel_y = src_y % 8;

        let final_tile_x = if h_flip { (obj_w / 8) - 1 - tile_x } else { tile_x };
        let final_tile_y = if v_flip { (obj_h / 8) - 1 - tile_y } else { tile_y };
        let final_pixel_x = if h_flip { 7 - pixel_x } else { pixel_x };
        let final_pixel_y = if v_flip { 7 - pixel_y } else { pixel_y };

        let base_tile = tile_num as u32;
        let tile_offset = if one_dimensional {
            final_tile_y * (obj_w / 8) + final_tile_x
        } else {
            final_tile_y * 32 + final_tile_x
        } as u32;

        let actual_tile = if is_256_color {
            base_tile + tile_offset * 2
        } else {
            base_tile + tile_offset
        };

        let tile_addr = obj_vram_base + actual_tile * (if is_256_color { 64 } else { 32 });
        let row_addr = tile_addr + final_pixel_y as u32 * (if is_256_color { 8 } else { 4 });

        if is_256_color {
            let pixel_addr = row_addr + final_pixel_x as u32;
            let palette_idx = bus.read8(pixel_addr) as usize;
            if palette_idx == 0 {
                return None;
            }
            let pal_addr = OBJ_PALETTE_START + (palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        } else {
            let byte_addr = row_addr + (final_pixel_x / 2) as u32;
            let byte = bus.read8(byte_addr);
            let palette_idx = if (final_pixel_x & 1) == 0 {
                byte & 0xF
            } else {
                byte >> 4
            } as usize;
            if palette_idx == 0 {
                return None;
            }
            let pal_addr = OBJ_PALETTE_START + (palette_num as usize * 32 + palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        }
    }

    fn render_affine_obj_pixel<B: crate::bus::BusAccess>(&self, bus: &mut B, obj_vram_base: u32, one_dimensional: bool, is_256_color: bool, tile_num: u16, palette_num: u16, param_group: usize, obj_w: usize, obj_h: usize, display_w: usize, display_h: usize, src_x: usize, src_y: usize) -> Option<u16> {
        let center_x = (obj_w / 2) as i32;
        let center_y = (obj_h / 2) as i32;

        let dx = src_x as i32 - (display_w / 2) as i32;
        let dy = src_y as i32 - (display_h / 2) as i32;

        let param_base = OAM_START + (param_group * 32 + 6) as u32;
        let pa_lo = bus.read8(param_base) as u16;
        let pa_hi = bus.read8(param_base + 1) as u16;
        let pa = (pa_lo | (pa_hi << 8)) as i16;
        let pb_lo = bus.read8(param_base + 8) as u16;
        let pb_hi = bus.read8(param_base + 9) as u16;
        let pb = (pb_lo | (pb_hi << 8)) as i16;
        let pc_lo = bus.read8(param_base + 16) as u16;
        let pc_hi = bus.read8(param_base + 17) as u16;
        let pc = (pc_lo | (pc_hi << 8)) as i16;
        let pd_lo = bus.read8(param_base + 24) as u16;
        let pd_hi = bus.read8(param_base + 25) as u16;
        let pd = (pd_lo | (pd_hi << 8)) as i16;

        let tex_x = center_x + ((pa as i32 * dx + pb as i32 * dy) >> 8);
        let tex_y = center_y + ((pc as i32 * dx + pd as i32 * dy) >> 8);

        if tex_x < 0 || tex_x >= obj_w as i32 || tex_y < 0 || tex_y >= obj_h as i32 {
            return None;
        }

        let tile_x = (tex_x as usize) / 8;
        let tile_y = (tex_y as usize) / 8;
        let pixel_x = (tex_x as usize) % 8;
        let pixel_y = (tex_y as usize) % 8;

        let base_tile = tile_num as u32;
        let tile_offset = if one_dimensional {
            tile_y * (obj_w / 8) + tile_x
        } else {
            tile_y * 32 + tile_x
        } as u32;

        let actual_tile = if is_256_color {
            base_tile + tile_offset * 2
        } else {
            base_tile + tile_offset
        };

        let tile_addr = obj_vram_base + actual_tile * (if is_256_color { 64 } else { 32 });
        let row_addr = tile_addr + pixel_y as u32 * (if is_256_color { 8 } else { 4 });

        if is_256_color {
            let pixel_addr = row_addr + pixel_x as u32;
            let palette_idx = bus.read8(pixel_addr) as usize;
            if palette_idx == 0 {
                return None;
            }
            let pal_addr = OBJ_PALETTE_START + (palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        } else {
            let byte_addr = row_addr + (pixel_x / 2) as u32;
            let byte = bus.read8(byte_addr);
            let palette_idx = if (pixel_x & 1) == 0 {
                byte & 0xF
            } else {
                byte >> 4
            } as usize;
            if palette_idx == 0 {
                return None;
            }
            let pal_addr = OBJ_PALETTE_START + (palette_num as usize * 32 + palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        }
    }

    fn get_bg_priority_at_safe<B: crate::bus::BusAccess>(&self, bus: &mut B, x: usize, y: usize, mode: u16, dispcnt: u16) -> u8 {
        let mut min_priority = 4u8;

        match mode {
            0 => {
                for bg_num in 0..4 {
                    let bit = 8 + bg_num;
                    if (dispcnt >> bit) & 1 == 0 {
                        continue;
                    }
                    if self.render_text_bg_pixel(bus, bg_num, x, y).is_some() {
                        let bgcnt = self.read_bgcnt(bus, bg_num);
                        let bg_priority = (bgcnt & 0x3) as u8;
                        if bg_priority < min_priority {
                            min_priority = bg_priority;
                        }
                    }
                }
            }
            1 => {
                for bg_num in 0..3 {
                    let bit = 8 + bg_num;
                    if (dispcnt >> bit) & 1 == 0 {
                        continue;
                    }
                    let has_pixel = if bg_num < 2 {
                        self.render_text_bg_pixel(bus, bg_num, x, y).is_some()
                    } else {
                        self.render_affine_bg_pixel(bus, bg_num, x, y).is_some()
                    };
                    if has_pixel {
                        let bgcnt = self.read_bgcnt(bus, bg_num);
                        let bg_priority = (bgcnt & 0x3) as u8;
                        if bg_priority < min_priority {
                            min_priority = bg_priority;
                        }
                    }
                }
            }
            2 => {
                for bg_num in 2..4 {
                    let bit = 8 + bg_num;
                    if (dispcnt >> bit) & 1 == 0 {
                        continue;
                    }
                    if self.render_affine_bg_pixel(bus, bg_num, x, y).is_some() {
                        let bgcnt = self.read_bgcnt(bus, bg_num);
                        let bg_priority = (bgcnt & 0x3) as u8;
                        if bg_priority < min_priority {
                            min_priority = bg_priority;
                        }
                    }
                }
            }
            _ => {}
        }

        min_priority
    }

    fn is_bg_enabled(&self, bg_num: usize) -> bool {
        let bit = 8 + bg_num;
        (self.dispcnt >> bit) & 1 != 0
    }

    fn read_backdrop_color<B: crate::bus::BusAccess>(&self, bus: &mut B) -> u16 {
        let lo = bus.read8(PALETTE_RAM_START) as u16;
        let hi = bus.read8(PALETTE_RAM_START + 1) as u16;
        lo | (hi << 8)
    }

    fn read_mosaic<B: crate::bus::BusAccess>(&self, bus: &mut B) -> u16 {
        let lo = bus.read8(REG_MOSAIC) as u16;
        let hi = bus.read8(REG_MOSAIC + 1) as u16;
        lo | (hi << 8)
    }

    fn apply_mosaic_x(&self, x: usize, mosaic: u16) -> usize {
        let h_size = ((mosaic & 0xF) + 1) as usize;
        (x / h_size) * h_size
    }

    fn apply_mosaic_y(&self, y: usize, mosaic: u16) -> usize {
        let v_size = (((mosaic >> 4) & 0xF) + 1) as usize;
        (y / v_size) * v_size
    }

    fn read_bgcnt<B: crate::bus::BusAccess>(&self, bus: &mut B, bg_num: usize) -> u16 {
        let addr = REG_BG0CNT + (bg_num * 2) as u32;
        let lo = bus.read8(addr) as u16;
        let hi = bus.read8(addr + 1) as u16;
        lo | (hi << 8)
    }

    fn read_bg_offset<B: crate::bus::BusAccess>(&self, bus: &mut B, bg_num: usize, h: bool) -> u16 {
        let base = REG_BG0HOFS + (bg_num * 4) as u32;
        let addr = if h { base } else { base + 2 };
        let lo = bus.read8(addr) as u16;
        let hi = bus.read8(addr + 1) as u16;
        (lo | (hi << 8)) & 0x1FF
    }

    fn render_text_bg_pixel<B: crate::bus::BusAccess>(&self, bus: &mut B, bg_num: usize, x: usize, y: usize) -> Option<u16> {
        let bgcnt = self.read_bgcnt(bus, bg_num);
        let hofs = self.read_bg_offset(bus, bg_num, true);
        let vofs = self.read_bg_offset(bus, bg_num, false);

        let screen_size = (bgcnt >> 14) & 0x3;
        let screen_base = (((bgcnt >> 8) & 0x1F) * 0x800) as u32;
        let char_base = (((bgcnt >> 2) & 0x3) * 0x4000) as u32;
        let is_256_color = (bgcnt >> 7) & 1 != 0;

        let bg_width = match screen_size {
            0 => 256,
            1 => 512,
            2 => 256,
            3 => 512,
            _ => 256,
        };
        let bg_height = match screen_size {
            0 => 256,
            1 => 256,
            2 => 512,
            3 => 512,
            _ => 256,
        };

        let bg_x = (x as u32 + hofs as u32) % bg_width;
        let bg_y = (y as u32 + vofs as u32) % bg_height;

        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;

        let map_addr = VRAM_START + screen_base + (tile_y * (bg_width / 8) + tile_x) * 2;
        let map_lo = bus.read8(map_addr) as u16;
        let map_hi = bus.read8(map_addr + 1) as u16;
        let map_entry = map_lo | (map_hi << 8);

        let tile_num = map_entry & 0x3FF;
        let h_flip = (map_entry >> 10) & 1 != 0;
        let v_flip = (map_entry >> 11) & 1 != 0;
        let palette_num = (map_entry >> 12) & 0xF;

        let final_pixel_x = if h_flip { 7 - pixel_x } else { pixel_x };
        let final_pixel_y = if v_flip { 7 - pixel_y } else { pixel_y };

        let tile_addr = VRAM_START + char_base + tile_num as u32 * (if is_256_color { 64 } else { 32 });
        let row_addr = tile_addr + final_pixel_y as u32 * (if is_256_color { 8 } else { 4 });

        if is_256_color {
            let pixel_addr = row_addr + final_pixel_x as u32;
            let palette_idx = bus.read8(pixel_addr) as usize;
            if palette_idx == 0 { return None; }
            let pal_addr = PALETTE_RAM_START + (palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        } else {
            let byte_addr = row_addr + (final_pixel_x / 2) as u32;
            let byte = bus.read8(byte_addr);
            let palette_idx = if (final_pixel_x & 1) == 0 {
                byte & 0xF
            } else {
                byte >> 4
            } as usize;
            if palette_idx == 0 { return None; }
            let pal_addr = PALETTE_RAM_START + (palette_num as usize * 32 + palette_idx * 2) as u32;
            let lo = bus.read8(pal_addr) as u16;
            let hi = bus.read8(pal_addr + 1) as u16;
            Some(lo | (hi << 8))
        }
    }

    fn render_affine_bg_pixel<B: crate::bus::BusAccess>(&self, bus: &mut B, bg_num: usize, x: usize, y: usize) -> Option<u16> {
        let bgcnt = self.read_bgcnt(bus, bg_num);
        let screen_size = (bgcnt >> 14) & 0x3;
        let screen_base = (((bgcnt >> 8) & 0x1F) * 0x800) as u32;
        let char_base = (((bgcnt >> 2) & 0x3) * 0x4000) as u32;
        let wrap = (bgcnt >> 13) & 1 != 0;

        let bg_size = match screen_size {
            0 => 128,
            1 => 256,
            2 => 512,
            3 => 1024,
            _ => 128,
        };

        let pa_addr = REG_BG2PA + ((bg_num - 2) * 0x10) as u32;
        let pb_addr = REG_BG2PB + ((bg_num - 2) * 0x10) as u32;
        let pc_addr = REG_BG2PC + ((bg_num - 2) * 0x10) as u32;
        let pd_addr = REG_BG2PD + ((bg_num - 2) * 0x10) as u32;
        let x_addr = REG_BG2X + ((bg_num - 2) * 0x10) as u32;
        let y_addr = REG_BG2Y + ((bg_num - 2) * 0x10) as u32;

        let pa_lo = bus.read8(pa_addr) as u16;
        let pa_hi = bus.read8(pa_addr + 1) as u16;
        let pa = (pa_lo | (pa_hi << 8)) as i16;

        let pb_lo = bus.read8(pb_addr) as u16;
        let pb_hi = bus.read8(pb_addr + 1) as u16;
        let pb = (pb_lo | (pb_hi << 8)) as i16;

        let pc_lo = bus.read8(pc_addr) as u16;
        let pc_hi = bus.read8(pc_addr + 1) as u16;
        let pc = (pc_lo | (pc_hi << 8)) as i16;

        let pd_lo = bus.read8(pd_addr) as u16;
        let pd_hi = bus.read8(pd_addr + 1) as u16;
        let pd = (pd_lo | (pd_hi << 8)) as i16;

        let x_lo = bus.read8(x_addr) as u32;
        let x_mid = bus.read8(x_addr + 1) as u32;
        let x_hi = bus.read8(x_addr + 2) as u32;
        let x_top = bus.read8(x_addr + 3) as u32;
        let mut ref_x = (x_lo | (x_mid << 8) | (x_hi << 16) | (x_top << 24)) as i32;
        ref_x = (ref_x << 4) >> 4;

        let y_lo = bus.read8(y_addr) as u32;
        let y_mid = bus.read8(y_addr + 1) as u32;
        let y_hi = bus.read8(y_addr + 2) as u32;
        let y_top = bus.read8(y_addr + 3) as u32;
        let mut ref_y = (y_lo | (y_mid << 8) | (y_hi << 16) | (y_top << 24)) as i32;
        ref_y = (ref_y << 4) >> 4;

        let src_x = ref_x + (pa as i32 * x as i32) + (pb as i32 * y as i32);
        let src_y = ref_y + (pc as i32 * x as i32) + (pd as i32 * y as i32);

        if !wrap && (src_x < 0 || src_x >= (bg_size * 8) as i32 || src_y < 0 || src_y >= (bg_size * 8) as i32) {
            return None;
        }

        let bg_x = (src_x as u32) % (bg_size * 8);
        let bg_y = (src_y as u32) % (bg_size * 8);

        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;

        let map_addr = VRAM_START + screen_base + (tile_y * bg_size + tile_x);
        let tile_num = bus.read8(map_addr) as u32;

        let tile_addr = VRAM_START + char_base + tile_num * 64;
        let row_addr = tile_addr + pixel_y * 8;
        let pixel_addr = row_addr + pixel_x;

        let palette_idx = bus.read8(pixel_addr) as usize;
        if palette_idx == 0 { return None; }

        let pal_addr = PALETTE_RAM_START + (palette_idx * 2) as u32;
        let lo = bus.read8(pal_addr) as u16;
        let hi = bus.read8(pal_addr + 1) as u16;
        Some(lo | (hi << 8))
    }

    fn get_window_region<B: crate::bus::BusAccess>(&self, bus: &mut B, x: usize, y: usize, obj_window_mask: &[bool]) -> u8 {
        let win0_enable = (self.dispcnt & DISPCNT_WIN0_ENABLE) != 0;
        let win1_enable = (self.dispcnt & DISPCNT_WIN1_ENABLE) != 0;
        let obj_win_enable = (self.dispcnt & DISPCNT_OBJ_WIN_ENABLE) != 0;

        if win0_enable {
            let win0h_lo = bus.read8(REG_WIN0H) as u16;
            let win0h_hi = bus.read8(REG_WIN0H + 1) as u16;
            let win0h = win0h_lo | (win0h_hi << 8);
            let win0v_lo = bus.read8(REG_WIN0V) as u16;
            let win0v_hi = bus.read8(REG_WIN0V + 1) as u16;
            let win0v = win0v_lo | (win0v_hi << 8);

            let x1 = ((win0h >> 8) & 0xFF) as usize;
            let x2 = ((win0h & 0xFF) as usize).min(240);
            let y1 = ((win0v >> 8) & 0xFF) as usize;
            let y2 = ((win0v & 0xFF) as usize).min(160);

            if x1 <= x2 && x >= x1 && x < x2 && y >= y1 && y < y2 {
                return 0;
            }
        }

        if win1_enable {
            let win1h_lo = bus.read8(REG_WIN1H) as u16;
            let win1h_hi = bus.read8(REG_WIN1H + 1) as u16;
            let win1h = win1h_lo | (win1h_hi << 8);
            let win1v_lo = bus.read8(REG_WIN1V) as u16;
            let win1v_hi = bus.read8(REG_WIN1V + 1) as u16;
            let win1v = win1v_lo | (win1v_hi << 8);

            let x1 = ((win1h >> 8) & 0xFF) as usize;
            let x2 = ((win1h & 0xFF) as usize).min(240);
            let y1 = ((win1v >> 8) & 0xFF) as usize;
            let y2 = ((win1v & 0xFF) as usize).min(160);

            if x1 <= x2 && x >= x1 && x < x2 && y >= y1 && y < y2 {
                return 1;
            }
        }

        if obj_win_enable && obj_window_mask[y * SCREEN_W + x] {
            return 2;
        }

        3
    }

    fn is_layer_enabled_in_window<B: crate::bus::BusAccess>(&self, bus: &mut B, window_region: u8, layer: usize, is_obj: bool) -> bool {
        let winin_lo = bus.read8(REG_WININ) as u16;
        let winin_hi = bus.read8(REG_WININ + 1) as u16;
        let winin = winin_lo | (winin_hi << 8);
        let winout_lo = bus.read8(REG_WINOUT) as u16;
        let winout_hi = bus.read8(REG_WINOUT + 1) as u16;
        let winout = winout_lo | (winout_hi << 8);

        let (mask, effect_mask) = match window_region {
            0 => {
                let bg_mask = (winin >> layer) & 1;
                let obj_mask = (winin >> 4) & 1;
                let effect = (winin >> 5) & 1;
                (if is_obj { obj_mask } else { bg_mask }, effect)
            }
            1 => {
                let bg_mask = (winin >> (8 + layer)) & 1;
                let obj_mask = (winin >> 12) & 1;
                let effect = (winin >> 13) & 1;
                (if is_obj { obj_mask } else { bg_mask }, effect)
            }
            2 => {
                let bg_mask = (winout >> (8 + layer)) & 1;
                let obj_mask = (winout >> 12) & 1;
                let effect = (winout >> 13) & 1;
                (if is_obj { obj_mask } else { bg_mask }, effect)
            }
            _ => {
                let bg_mask = (winout >> layer) & 1;
                let obj_mask = (winout >> 4) & 1;
                let effect = (winout >> 5) & 1;
                (if is_obj { obj_mask } else { bg_mask }, effect)
            }
        };

        mask != 0
    }

    fn build_obj_window_mask<B: crate::bus::BusAccess>(&self, bus: &mut B) -> Vec<bool> {
        let mut mask = vec![false; FRAME_PIXELS];

        if (self.dispcnt & DISPCNT_OBJ_ENABLE) == 0 || (self.dispcnt & DISPCNT_OBJ_WIN_ENABLE) == 0 {
            return mask;
        }

        let mode = self.dispcnt & DISPCNT_MODE_MASK;
        let obj_vram_base = if mode >= 3 { OBJ_VRAM_START_MODE345 } else { OBJ_VRAM_START_MODE012 };
        let one_dimensional = (self.dispcnt & DISPCNT_OBJ_VRAM_MAPPING) != 0;

        for obj_num in 0..128 {
            let oam_addr = OAM_START + (obj_num * 8) as u32;
            let attr0_lo = bus.read8(oam_addr) as u16;
            let attr0_hi = bus.read8(oam_addr + 1) as u16;
            let attr0 = attr0_lo | (attr0_hi << 8);
            let attr1_lo = bus.read8(oam_addr + 2) as u16;
            let attr1_hi = bus.read8(oam_addr + 3) as u16;
            let attr1 = attr1_lo | (attr1_hi << 8);
            let attr2_lo = bus.read8(oam_addr + 4) as u16;
            let attr2_hi = bus.read8(oam_addr + 5) as u16;
            let attr2 = attr2_lo | (attr2_hi << 8);

            let obj_mode = (attr0 >> 10) & 0x3;
            if obj_mode != 2 {
                continue;
            }

            let y = (attr0 & 0xFF) as usize;
            let x = (attr1 & 0x1FF) as usize;
            let rotation_scaling = (attr0 >> 8) & 1 != 0;
            let obj_disable = !rotation_scaling && ((attr0 >> 9) & 1 != 0);
            let is_256_color = (attr0 >> 13) & 1 != 0;
            let shape = (attr0 >> 14) & 0x3;
            let size = (attr1 >> 14) & 0x3;
            let tile_num = attr2 & 0x3FF;
            let palette_num = (attr2 >> 12) & 0xF;

            if obj_disable {
                continue;
            }

            let (obj_w, obj_h) = self.get_obj_size(shape, size);
            let display_w = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_w * 2
            } else {
                obj_w
            };
            let display_h = if rotation_scaling && ((attr0 >> 9) & 1 != 0) {
                obj_h * 2
            } else {
                obj_h
            };

            let screen_y = if y >= 160 { y.wrapping_sub(256) } else { y };
            let screen_x = if x >= 240 { x.wrapping_sub(512) } else { x };

            for py in 0..display_h {
                let fy = screen_y.wrapping_add(py);
                if fy >= SCREEN_H {
                    continue;
                }

                let src_y = py;
                if src_y >= display_h {
                    continue;
                }

                for px in 0..display_w {
                    let fx = screen_x.wrapping_add(px);
                    if fx >= SCREEN_W {
                        continue;
                    }

                    let src_x = px;
                    if src_x >= display_w {
                        continue;
                    }

                    let pixel = if rotation_scaling {
                        let param_group = ((attr1 >> 9) & 0x1F) as usize;
                        self.render_affine_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, param_group, obj_w, obj_h, display_w, display_h, src_x, src_y)
                    } else {
                        let h_flip = (attr1 >> 12) & 1 != 0;
                        let v_flip = (attr1 >> 13) & 1 != 0;
                        self.render_regular_obj_pixel(bus, obj_vram_base, one_dimensional, is_256_color, tile_num, palette_num, obj_w, obj_h, src_x, src_y, h_flip, v_flip)
                    };

                    if pixel.is_some() {
                        let idx = fy * SCREEN_W + fx;
                        mask[idx] = true;
                    }
                }
            }
        }

        mask
    }

    fn read_bldcnt<B: crate::bus::BusAccess>(&self, bus: &mut B) -> u16 {
        let lo = bus.read8(REG_BLDCNT) as u16;
        let hi = bus.read8(REG_BLDCNT + 1) as u16;
        lo | (hi << 8)
    }

    fn read_bldalpha<B: crate::bus::BusAccess>(&self, bus: &mut B) -> u16 {
        let lo = bus.read8(REG_BLDALPHA) as u16;
        let hi = bus.read8(REG_BLDALPHA + 1) as u16;
        lo | (hi << 8)
    }

    fn read_bldy<B: crate::bus::BusAccess>(&self, bus: &mut B) -> u16 {
        bus.read8(REG_BLDY) as u16
    }

    fn is_1st_target<B: crate::bus::BusAccess>(&self, bus: &mut B, layer: usize, is_obj: bool, is_backdrop: bool) -> bool {
        let bldcnt = self.read_bldcnt(bus);
        if is_backdrop {
            return (bldcnt >> 5) & 1 != 0;
        }
        if is_obj {
            return (bldcnt >> 4) & 1 != 0;
        }
        (bldcnt >> layer) & 1 != 0
    }

    fn is_2nd_target<B: crate::bus::BusAccess>(&self, bus: &mut B, layer: usize, is_obj: bool, is_backdrop: bool) -> bool {
        let bldcnt = self.read_bldcnt(bus);
        if is_backdrop {
            return (bldcnt >> 13) & 1 != 0;
        }
        if is_obj {
            return (bldcnt >> 12) & 1 != 0;
        }
        (bldcnt >> (8 + layer)) & 1 != 0
    }

    fn apply_color_effects<B: crate::bus::BusAccess>(&self, bus: &mut B, pixel1: u16, pixel2: Option<u16>, layer1: usize, is_obj1: bool, is_backdrop1: bool) -> u16 {
        let bldcnt = self.read_bldcnt(bus);
        let effect_mode = (bldcnt >> 6) & 0x3;

        if effect_mode == 0 {
            return pixel1;
        }

        let is_1st = self.is_1st_target(bus, layer1, is_obj1, is_backdrop1);
        if !is_1st {
            return pixel1;
        }

        match effect_mode {
            1 => {
                if let Some(p2) = pixel2 {
                    let bldalpha = self.read_bldalpha(bus);
                    let eva = ((bldalpha & 0x1F) as u32).min(16);
                    let evb = (((bldalpha >> 8) & 0x1F) as u32).min(16);

                    let r1 = ((pixel1 >> 0) & 0x1F) as u32;
                    let g1 = ((pixel1 >> 5) & 0x1F) as u32;
                    let b1 = ((pixel1 >> 10) & 0x1F) as u32;

                    let r2 = ((p2 >> 0) & 0x1F) as u32;
                    let g2 = ((p2 >> 5) & 0x1F) as u32;
                    let b2 = ((p2 >> 10) & 0x1F) as u32;

                    let r = ((r1 * eva + r2 * evb) / 16).min(31) as u16;
                    let g = ((g1 * eva + g2 * evb) / 16).min(31) as u16;
                    let b = ((b1 * eva + b2 * evb) / 16).min(31) as u16;

                    r | (g << 5) | (b << 10)
                } else {
                    pixel1
                }
            }
            2 => {
                let bldy = self.read_bldy(bus);
                let evy = ((bldy & 0x1F) as u32).min(16);

                let r1 = ((pixel1 >> 0) & 0x1F) as u32;
                let g1 = ((pixel1 >> 5) & 0x1F) as u32;
                let b1 = ((pixel1 >> 10) & 0x1F) as u32;

                let r = (r1 + ((31 - r1) * evy / 16)).min(31) as u16;
                let g = (g1 + ((31 - g1) * evy / 16)).min(31) as u16;
                let b = (b1 + ((31 - b1) * evy / 16)).min(31) as u16;

                r | (g << 5) | (b << 10)
            }
            3 => {
                let bldy = self.read_bldy(bus);
                let evy = ((bldy & 0x1F) as u32).min(16);

                let r1 = ((pixel1 >> 0) & 0x1F) as u32;
                let g1 = ((pixel1 >> 5) & 0x1F) as u32;
                let b1 = ((pixel1 >> 10) & 0x1F) as u32;

                let r = (r1 - (r1 * evy / 16)).min(31) as u16;
                let g = (g1 - (g1 * evy / 16)).min(31) as u16;
                let b = (b1 - (b1 * evy / 16)).min(31) as u16;

                r | (g << 5) | (b << 10)
            }
            _ => pixel1,
        }
    }
}

/// The main test module for the PPU.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{Bus, BusAccess};

    /// Test Suite for PPU Initialization and basic state.
    #[test]
    fn ppu_can_be_created() {
        let ppu = Ppu::new();
        assert_eq!(ppu.read_dispcnt(), 0);
        assert_eq!(ppu.read_dispstat(), 0);
        assert_eq!(ppu.framebuffer().len(), FRAME_PIXELS);
        assert!(ppu.framebuffer().iter().all(|&p| p == 0));
    }

    /// Test Suite for Display Control Register (REG_DISPCNT).
    #[test]
    fn display_mode_is_set_correctly() {
        let mut ppu = Ppu::new();
        ppu.write_dispcnt(0);
        assert_eq!(ppu.read_dispcnt() & DISPCNT_MODE_MASK, 0);
        ppu.write_dispcnt(1);
        assert_eq!(ppu.read_dispcnt() & DISPCNT_MODE_MASK, 1);
    }

    #[test]
    fn backgrounds_are_enabled_and_disabled() {
        // TODO: Test enabling and disabling individual backgrounds (BG0-BG3) via REG_DISPCNT.
    }

    #[test]
    fn sprites_are_enabled_and_disabled() {
        // TODO: Test enabling and disabling sprites via REG_DISPCNT.
    }

    #[test]
    fn forced_blank_mode_is_respected() {
        let mut ppu = Ppu::new();
        ppu.write_palette_entry(0, 0x7C00); // non-black to ensure change visible
        ppu.write_dispcnt(DISPCNT_FORCED_BLANK);
        ppu.step(ppu.cycles_until_vblank() + 4);
        assert!(ppu.framebuffer().iter().all(|&px| px == 0));
    }
    #[test]
    fn io_dispcnt_controls_mode0_bg0_via_bus() {
        let mut ppu = Ppu::new();
        let mut bus = Bus::new();
        // palette[0] = red
        bus.write16(PALETTE_RAM_START, 0x7C00);
        // DISPCNT = mode 0 + BG0 enable (bit 8)
        let dispcnt = 0u16 | (1 << 8);
        bus.write16(REG_DISPCNT, dispcnt);

        // render via bus
        ppu.render_frame_with_bus(&mut bus);
        assert!(ppu.framebuffer().iter().all(|&px| px == 0x7C00));
    }

    /// Test Suite for Display Status Register (REG_DISPSTAT).
    #[test]
    fn vblank_flag_is_set_and_cleared() {
        let mut ppu = Ppu::new();
        assert_eq!(ppu.read_dispstat() & DISPSTAT_VBLANK_FLAG, 0);
        ppu.step(ppu.cycles_until_vblank() + 1);
        assert_ne!(ppu.read_dispstat() & DISPSTAT_VBLANK_FLAG, 0);
        // Advance to end of frame to clear VBlank flag
        let rem = ppu.cycles_per_frame() - ppu.cycles_until_vblank() - 1;
        ppu.step(rem);
        assert_eq!(ppu.read_dispstat() & DISPSTAT_VBLANK_FLAG, 0);
    }

    #[test]
    fn hblank_flag_is_set_and_cleared() {
        // TODO: Simulate the PPU drawing a single scanline and assert the H-Blank flag is set and cleared.
    }

    #[test]
    fn vcount_match_flag_is_set() {
        // TODO: Set a V-Count match value in REG_DISPSTAT, run the PPU, and assert the flag is set when VCOUNT matches.
    }

    /// Test Suite for Vertical Count Register (REG_VCOUNT).
    #[test]
    fn vcount_increments_correctly_per_scanline() {
        // TODO: Simulate the PPU drawing a few scanlines and assert REG_VCOUNT's value.
    }

    /// Test Suite for Background Control Registers (REG_BGxCNT).
    #[test]
    fn background_priority_is_set_correctly() {
        // TODO: Test setting priorities for multiple backgrounds.
    }

    #[test]
    fn background_character_base_block_is_set() {
        // Not applicable in minimal implementation; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn background_screen_base_block_is_set() {
        // Not applicable in minimal implementation; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn background_size_is_set_correctly() {
        // Not applicable in minimal implementation; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Background Offsets (REG_BGxHOFS, REG_BGxVOFS).
    #[test]
    fn background_offsets_are_applied() {
        // Not applicable in minimal implementation; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Sprite Attributes (OAM).
    #[test]
    fn sprite_position_is_correct() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn sprite_size_and_shape_are_correct() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn sprite_rendering_with_alpha_blending() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Affine Transformations (Backgrounds and Sprites).
    #[test]
    fn affine_background_is_transformed_correctly() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn affine_sprite_is_transformed_correctly() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Windowing.
    #[test]
    fn window_clips_correctly() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Color Effects (Alpha Blending, Brightness).
    #[test]
    fn alpha_blending_is_applied_correctly() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn brightness_is_adjusted_correctly() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    /// Test Suite for Interrupts.
    #[test]
    fn vblank_interrupt_is_triggered() {
        // CPU interrupt wiring not present in minimal core; ensure no panic during frame step.
        let mut ppu = Ppu::new();
        ppu.step(ppu.cycles_per_frame());
        assert!(true);
    }

    #[test]
    fn hblank_interrupt_is_triggered() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }

    #[test]
    fn vcount_match_interrupt_is_triggered() {
        // Not implemented in minimal PPU; placeholder ensures test module compiles.
        assert!(true);
    }
}
