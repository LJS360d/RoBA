pub trait BusAccess {
	fn read32(&mut self, addr: u32) -> u32;
	fn read16(&mut self, addr: u32) -> u16;
	fn read8(&mut self, addr: u32) -> u8;
	fn write32(&mut self, addr: u32, value: u32);
	fn write16(&mut self, addr: u32, value: u16);
	fn write8(&mut self, addr: u32, value: u8);
}

pub struct Bus;
impl Bus { pub fn new() -> Self { Self } }
