package cartridge

const (
	SRAM_START = 0x0E000000
	SRAM_END   = 0x0E007FFF
	SRAM_SIZE  = SRAM_END - SRAM_START + 1 // 1KB
)

type Cartridge struct {
	ROM  []byte
	SRAM []byte
}

func NewCartridge(romData []byte) *Cartridge {
	c := &Cartridge{
		ROM:  romData,
		SRAM: make([]byte, SRAM_SIZE),
	}
	return c
}

func (c *Cartridge) ReadROM8(addr uint32) uint8 {
	return c.ROM[addr]
}
func (c *Cartridge) ReadSRAM8(addr uint32) uint8 {
	return c.SRAM[addr]
}
func (c *Cartridge) WriteROM8(addr uint32, value uint8) {
	c.ROM[addr] = value
}
func (c *Cartridge) WriteSRAM8(addr uint32, value uint8) {
	c.SRAM[addr] = value
}
