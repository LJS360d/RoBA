pub struct Io {
    pub dispcnt: u16,
    pub dispstat: u16,
    pub vcount: u16,
    pub bg0cnt: u16,
    pub bg1cnt: u16,
    pub bg2cnt: u16,
    pub bg3cnt: u16,
    pub bg0hofs: u16,
    pub bg0vofs: u16,
    pub bg1hofs: u16,
    pub bg1vofs: u16,
    pub bg2hofs: u16,
    pub bg2vofs: u16,
    pub bg3hofs: u16,
    pub bg3vofs: u16,
    pub bg2pa: i16,
    pub bg2pb: i16,
    pub bg2pc: i16,
    pub bg2pd: i16,
    pub bg2x: i32,
    pub bg2y: i32,
    pub bg3pa: i16,
    pub bg3pb: i16,
    pub bg3pc: i16,
    pub bg3pd: i16,
    pub bg3x: i32,
    pub bg3y: i32,
    pub mosaic: u16,

    pub keyinput: u16,
    pub keycnt: u16,

    pub ie: u16,
    pub if_: u16,
    pub ime: u16,

    pub postflg: u8,
    pub haltcnt: u8,
    pub halted: bool,
}

impl Default for Io {
    fn default() -> Self {
        Self {
            dispcnt: 0,
            dispstat: 0,
            vcount: 0,
            bg0cnt: 0,
            bg1cnt: 0,
            bg2cnt: 0,
            bg3cnt: 0,
            bg0hofs: 0,
            bg0vofs: 0,
            bg1hofs: 0,
            bg1vofs: 0,
            bg2hofs: 0,
            bg2vofs: 0,
            bg3hofs: 0,
            bg3vofs: 0,
            bg2pa: 0x0100,
            bg2pb: 0,
            bg2pc: 0,
            bg2pd: 0x0100,
            bg2x: 0,
            bg2y: 0,
            bg3pa: 0x0100,
            bg3pb: 0,
            bg3pc: 0,
            bg3pd: 0x0100,
            bg3x: 0,
            bg3y: 0,
            mosaic: 0,

            keyinput: 0x03FF,
            keycnt: 0,

            ie: 0,
            if_: 0,
            ime: 0,

            postflg: 0,
            haltcnt: 0,
            halted: false,
        }
    }
}

impl Io {
    pub fn new() -> Self { Self::default() }

    pub fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x0400_0000 => (self.dispcnt & 0xFF) as u8,
            0x0400_0001 => (self.dispcnt >> 8) as u8,
            0x0400_0004 => (self.dispstat & 0xFF) as u8,
            0x0400_0005 => (self.dispstat >> 8) as u8,
            0x0400_0006 => (self.vcount & 0xFF) as u8,
            0x0400_0007 => (self.vcount >> 8) as u8,
            0x0400_0008 => (self.bg0cnt & 0xFF) as u8,
            0x0400_0009 => (self.bg0cnt >> 8) as u8,
            0x0400_000A => (self.bg1cnt & 0xFF) as u8,
            0x0400_000B => (self.bg1cnt >> 8) as u8,
            0x0400_000C => (self.bg2cnt & 0xFF) as u8,
            0x0400_000D => (self.bg2cnt >> 8) as u8,
            0x0400_000E => (self.bg3cnt & 0xFF) as u8,
            0x0400_000F => (self.bg3cnt >> 8) as u8,
            0x0400_0010 => (self.bg0hofs & 0xFF) as u8,
            0x0400_0011 => (self.bg0hofs >> 8) as u8,
            0x0400_0012 => (self.bg0vofs & 0xFF) as u8,
            0x0400_0013 => (self.bg0vofs >> 8) as u8,
            0x0400_0014 => (self.bg1hofs & 0xFF) as u8,
            0x0400_0015 => (self.bg1hofs >> 8) as u8,
            0x0400_0016 => (self.bg1vofs & 0xFF) as u8,
            0x0400_0017 => (self.bg1vofs >> 8) as u8,
            0x0400_0018 => (self.bg2hofs & 0xFF) as u8,
            0x0400_0019 => (self.bg2hofs >> 8) as u8,
            0x0400_001A => (self.bg2vofs & 0xFF) as u8,
            0x0400_001B => (self.bg2vofs >> 8) as u8,
            0x0400_001C => (self.bg3hofs & 0xFF) as u8,
            0x0400_001D => (self.bg3hofs >> 8) as u8,
            0x0400_001E => (self.bg3vofs & 0xFF) as u8,
            0x0400_001F => (self.bg3vofs >> 8) as u8,
            0x0400_0020 => (self.bg2pa as u16 & 0xFF) as u8,
            0x0400_0021 => ((self.bg2pa as u16) >> 8) as u8,
            0x0400_0022 => (self.bg2pb as u16 & 0xFF) as u8,
            0x0400_0023 => ((self.bg2pb as u16) >> 8) as u8,
            0x0400_0024 => (self.bg2pc as u16 & 0xFF) as u8,
            0x0400_0025 => ((self.bg2pc as u16) >> 8) as u8,
            0x0400_0026 => (self.bg2pd as u16 & 0xFF) as u8,
            0x0400_0027 => ((self.bg2pd as u16) >> 8) as u8,
            0x0400_0028 => (self.bg2x as u32 & 0xFF) as u8,
            0x0400_0029 => ((self.bg2x as u32 >> 8) & 0xFF) as u8,
            0x0400_002A => ((self.bg2x as u32 >> 16) & 0xFF) as u8,
            0x0400_002B => ((self.bg2x as u32 >> 24) & 0xFF) as u8,
            0x0400_002C => (self.bg2y as u32 & 0xFF) as u8,
            0x0400_002D => ((self.bg2y as u32 >> 8) & 0xFF) as u8,
            0x0400_002E => ((self.bg2y as u32 >> 16) & 0xFF) as u8,
            0x0400_002F => ((self.bg2y as u32 >> 24) & 0xFF) as u8,
            0x0400_0030 => (self.bg3pa as u16 & 0xFF) as u8,
            0x0400_0031 => ((self.bg3pa as u16) >> 8) as u8,
            0x0400_0032 => (self.bg3pb as u16 & 0xFF) as u8,
            0x0400_0033 => ((self.bg3pb as u16) >> 8) as u8,
            0x0400_0034 => (self.bg3pc as u16 & 0xFF) as u8,
            0x0400_0035 => ((self.bg3pc as u16) >> 8) as u8,
            0x0400_0036 => (self.bg3pd as u16 & 0xFF) as u8,
            0x0400_0037 => ((self.bg3pd as u16) >> 8) as u8,
            0x0400_0038 => (self.bg3x as u32 & 0xFF) as u8,
            0x0400_0039 => ((self.bg3x as u32 >> 8) & 0xFF) as u8,
            0x0400_003A => ((self.bg3x as u32 >> 16) & 0xFF) as u8,
            0x0400_003B => ((self.bg3x as u32 >> 24) & 0xFF) as u8,
            0x0400_003C => (self.bg3y as u32 & 0xFF) as u8,
            0x0400_003D => ((self.bg3y as u32 >> 8) & 0xFF) as u8,
            0x0400_003E => ((self.bg3y as u32 >> 16) & 0xFF) as u8,
            0x0400_003F => ((self.bg3y as u32 >> 24) & 0xFF) as u8,
            0x0400_004C => (self.mosaic & 0xFF) as u8,
            0x0400_004D => (self.mosaic >> 8) as u8,

            0x0400_0130 => (self.keyinput & 0xFF) as u8,
            0x0400_0131 => (self.keyinput >> 8) as u8,
            0x0400_0132 => (self.keycnt & 0xFF) as u8,
            0x0400_0133 => (self.keycnt >> 8) as u8,

            0x0400_0200 => (self.ie & 0xFF) as u8,
            0x0400_0201 => (self.ie >> 8) as u8,
            0x0400_0202 => (self.if_ & 0xFF) as u8,
            0x0400_0203 => (self.if_ >> 8) as u8,
            0x0400_0208 => (self.ime & 0xFF) as u8,
            0x0400_0209 => (self.ime >> 8) as u8,

            0x0400_0300 => self.postflg,
            0x0400_0301 => 0,

            _ => 0,
        }
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x0400_0000 => self.dispcnt = (self.dispcnt & 0xFF00) | value as u16,
            0x0400_0001 => self.dispcnt = (self.dispcnt & 0x00FF) | ((value as u16) << 8),
            0x0400_0004 => self.dispstat = (self.dispstat & 0xFF00) | value as u16,
            0x0400_0005 => self.dispstat = (self.dispstat & 0x00FF) | ((value as u16) << 8),
            0x0400_0006 => {}
            0x0400_0007 => {}
            0x0400_0008 => self.bg0cnt = (self.bg0cnt & 0xFF00) | value as u16,
            0x0400_0009 => self.bg0cnt = (self.bg0cnt & 0x00FF) | ((value as u16) << 8),
            0x0400_000A => self.bg1cnt = (self.bg1cnt & 0xFF00) | value as u16,
            0x0400_000B => self.bg1cnt = (self.bg1cnt & 0x00FF) | ((value as u16) << 8),
            0x0400_000C => self.bg2cnt = (self.bg2cnt & 0xFF00) | value as u16,
            0x0400_000D => self.bg2cnt = (self.bg2cnt & 0x00FF) | ((value as u16) << 8),
            0x0400_000E => self.bg3cnt = (self.bg3cnt & 0xFF00) | value as u16,
            0x0400_000F => self.bg3cnt = (self.bg3cnt & 0x00FF) | ((value as u16) << 8),
            0x0400_0010 => self.bg0hofs = (self.bg0hofs & 0xFF00) | value as u16,
            0x0400_0011 => self.bg0hofs = (self.bg0hofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_0012 => self.bg0vofs = (self.bg0vofs & 0xFF00) | value as u16,
            0x0400_0013 => self.bg0vofs = (self.bg0vofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_0014 => self.bg1hofs = (self.bg1hofs & 0xFF00) | value as u16,
            0x0400_0015 => self.bg1hofs = (self.bg1hofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_0016 => self.bg1vofs = (self.bg1vofs & 0xFF00) | value as u16,
            0x0400_0017 => self.bg1vofs = (self.bg1vofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_0018 => self.bg2hofs = (self.bg2hofs & 0xFF00) | value as u16,
            0x0400_0019 => self.bg2hofs = (self.bg2hofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_001A => self.bg2vofs = (self.bg2vofs & 0xFF00) | value as u16,
            0x0400_001B => self.bg2vofs = (self.bg2vofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_001C => self.bg3hofs = (self.bg3hofs & 0xFF00) | value as u16,
            0x0400_001D => self.bg3hofs = (self.bg3hofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_001E => self.bg3vofs = (self.bg3vofs & 0xFF00) | value as u16,
            0x0400_001F => self.bg3vofs = (self.bg3vofs & 0x00FF) | (((value as u16) & 1) << 8),
            0x0400_0020 => self.bg2pa = (self.bg2pa as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0021 => self.bg2pa = ((self.bg2pa as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0022 => self.bg2pb = (self.bg2pb as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0023 => self.bg2pb = ((self.bg2pb as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0024 => self.bg2pc = (self.bg2pc as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0025 => self.bg2pc = ((self.bg2pc as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0026 => self.bg2pd = (self.bg2pd as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0027 => self.bg2pd = ((self.bg2pd as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0028 => {
                let old = self.bg2x as u32;
                self.bg2x = ((old & !0xFF) | value as u32) as i32;
            }
            0x0400_0029 => {
                let old = self.bg2x as u32;
                self.bg2x = ((old & !0xFF00) | ((value as u32) << 8)) as i32;
            }
            0x0400_002A => {
                let old = self.bg2x as u32;
                self.bg2x = ((old & !0xFF0000) | ((value as u32) << 16)) as i32;
            }
            0x0400_002B => {
                let old = self.bg2x as u32;
                self.bg2x = ((old & !0xFF000000) | ((value as u32) << 24)) as i32;
                self.bg2x = (self.bg2x << 4) >> 4;
            }
            0x0400_002C => {
                let old = self.bg2y as u32;
                self.bg2y = ((old & !0xFF) | value as u32) as i32;
            }
            0x0400_002D => {
                let old = self.bg2y as u32;
                self.bg2y = ((old & !0xFF00) | ((value as u32) << 8)) as i32;
            }
            0x0400_002E => {
                let old = self.bg2y as u32;
                self.bg2y = ((old & !0xFF0000) | ((value as u32) << 16)) as i32;
            }
            0x0400_002F => {
                let old = self.bg2y as u32;
                self.bg2y = ((old & !0xFF000000) | ((value as u32) << 24)) as i32;
                self.bg2y = (self.bg2y << 4) >> 4;
            }
            0x0400_0030 => self.bg3pa = (self.bg3pa as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0031 => self.bg3pa = ((self.bg3pa as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0032 => self.bg3pb = (self.bg3pb as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0033 => self.bg3pb = ((self.bg3pb as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0034 => self.bg3pc = (self.bg3pc as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0035 => self.bg3pc = ((self.bg3pc as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0036 => self.bg3pd = (self.bg3pd as u16 & 0xFF00 | value as u16) as i16,
            0x0400_0037 => self.bg3pd = ((self.bg3pd as u16 & 0x00FF) | ((value as u16) << 8)) as i16,
            0x0400_0038 => {
                let old = self.bg3x as u32;
                self.bg3x = ((old & !0xFF) | value as u32) as i32;
            }
            0x0400_0039 => {
                let old = self.bg3x as u32;
                self.bg3x = ((old & !0xFF00) | ((value as u32) << 8)) as i32;
            }
            0x0400_003A => {
                let old = self.bg3x as u32;
                self.bg3x = ((old & !0xFF0000) | ((value as u32) << 16)) as i32;
            }
            0x0400_003B => {
                let old = self.bg3x as u32;
                self.bg3x = ((old & !0xFF000000) | ((value as u32) << 24)) as i32;
                self.bg3x = (self.bg3x << 4) >> 4;
            }
            0x0400_003C => {
                let old = self.bg3y as u32;
                self.bg3y = ((old & !0xFF) | value as u32) as i32;
            }
            0x0400_003D => {
                let old = self.bg3y as u32;
                self.bg3y = ((old & !0xFF00) | ((value as u32) << 8)) as i32;
            }
            0x0400_003E => {
                let old = self.bg3y as u32;
                self.bg3y = ((old & !0xFF0000) | ((value as u32) << 16)) as i32;
            }
            0x0400_003F => {
                let old = self.bg3y as u32;
                self.bg3y = ((old & !0xFF000000) | ((value as u32) << 24)) as i32;
                self.bg3y = (self.bg3y << 4) >> 4;
            }
            0x0400_004C => self.mosaic = (self.mosaic & 0xFF00) | value as u16,
            0x0400_004D => self.mosaic = (self.mosaic & 0x00FF) | ((value as u16) << 8),

            0x0400_0130 => {}
            0x0400_0131 => {}
            0x0400_0132 => self.keycnt = (self.keycnt & 0xFF00) | value as u16,
            0x0400_0133 => self.keycnt = (self.keycnt & 0x00FF) | ((value as u16) << 8),

            0x0400_0200 => self.ie = (self.ie & 0xFF00) | value as u16,
            0x0400_0201 => self.ie = (self.ie & 0x00FF) | ((value as u16) << 8),
            0x0400_0202 => self.if_ &= !(value as u16),
            0x0400_0203 => self.if_ &= !((value as u16) << 8),
            0x0400_0208 => self.ime = value as u16 & 1,
            0x0400_0209 => {}

            0x0400_0300 => self.postflg = value & 1,
            0x0400_0301 => {
                self.haltcnt = value;
                if (value & 0x80) == 0 {
                    self.halted = true;
                }
            }

            _ => {}
        }
    }

    pub fn request_interrupt(&mut self, irq: u16) {
        self.if_ |= irq;
        if (self.ie & irq) != 0 {
            self.halted = false;
        }
    }

    pub fn pending_interrupts(&self) -> bool {
        (self.ime & 1) != 0 && (self.ie & self.if_) != 0
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }
}
