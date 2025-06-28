package ppu

import (
	"GoBA/internal/interfaces"
	"image"
	"image/color"
)

const (
	ScreenWidth  = 240
	ScreenHeight = 160
)

type PPU struct {
	Bus        interfaces.BusInterface
	Frame      *image.RGBA
	VCount     uint16 // Vertical counter
	dispcnt    uint32 // Display control register
	frameReady bool
}

func NewPPU() *PPU {
	return &PPU{
		Bus:     nil,
		Frame:   image.NewRGBA(image.Rect(0, 0, ScreenWidth, ScreenHeight)),
		VCount:  0,
		dispcnt: 0,
	}
}

func (p *PPU) SetBus(bus interfaces.BusInterface) {
	p.Bus = bus
}

func (p *PPU) IsPPUIORegister(addr uint32) bool {
	return addr <= 0x005F
}

func (p *PPU) ReadIORegister8(addr uint32) uint8 {
	switch addr {
	case 0x0000: // DISPCNT LSB
		return uint8(p.dispcnt & 0xFF)
	case 0x0001: // DISPCNT MSB
		return uint8((p.dispcnt >> 8) & 0xFF)
	case 0x0006: // VCOUNT LSB
		return uint8(p.VCount & 0xFF)
	case 0x0007: // VCOUNT MSB
		return uint8(p.VCount >> 8)
	}
	return 0
}

func (p *PPU) WriteIORegister8(addr uint32, value uint8) {
	switch addr {
	case 0x0000: // DISPCNT LSB
		p.dispcnt = (p.dispcnt & 0xFF00) | uint32(value)
	case 0x0001: // DISPCNT MSB
		p.dispcnt = (p.dispcnt & 0x00FF) | (uint32(value) << 8)
	}
}

func (p *PPU) ReadPaletteRAM8(addr uint32) uint8 {
	// Palette RAM is 1KB (512 colors)
	if addr < 0x400 {
		return p.Bus.GetIORegsPtr().GetReg(0x05000000&0x3FF + addr)
	}
	return 0
}

func (p *PPU) WritePaletteRAM8(addr uint32, value uint8) {
	if addr < 0x400 {
		p.Bus.GetIORegsPtr().SetReg(0x05000000&0x3FF+addr, value)
	}
}

func (p *PPU) ReadVRAM8(addr uint32) uint8 {
	if addr < 0x18000 { // VRAM is 96KB
		return p.Bus.GetIORegsPtr().GetReg(0x06000000&0x1FFFF + addr)
	}
	return 0
}

func (p *PPU) WriteVRAM8(addr uint32, value uint8) {
	if addr < 0x18000 {
		p.Bus.GetIORegsPtr().SetReg(0x06000000&0x1FFFF+addr, value)
	}
}

func (p *PPU) ReadOAM8(addr uint32) uint8 {
	// TODO
	return 0
}

func (p *PPU) WriteOAM8(addr uint32, value uint8) {
	// TODO
}

func (p *PPU) RenderScanline() {
	mode := p.dispcnt & 0x7
	switch mode {
	case 3: // 16-bit color bitmap mode
		p.renderMode3()
	default:
		// Clear screen to black for unsupported modes
		for x := 0; x < ScreenWidth; x++ {
			p.Frame.SetRGBA(x, int(p.VCount), color.RGBA{0, 0, 0, 255})
		}
	}
}

func (p *PPU) renderMode3() {
	// VRAM base address for Mode 3
	vramBase := uint32(0x06000000)

	// Each pixel is 2 bytes (16-bit BGR555)
	for x := 0; x < ScreenWidth; x++ {
		pixelAddr := vramBase + uint32(p.VCount*ScreenWidth*2) + uint32(x*2)
		color16 := uint16(p.Bus.Read8(pixelAddr)) | (uint16(p.Bus.Read8(pixelAddr+1)) << 8)

		// Convert BGR555 to RGBA8888
		r := uint8((color16 & 0x1F) * 8)         // 5 bits red
		g := uint8(((color16 >> 5) & 0x1F) * 8)  // 5 bits green
		b := uint8(((color16 >> 10) & 0x1F) * 8) // 5 bits blue

		p.Frame.SetRGBA(x, int(p.VCount), color.RGBA{r, g, b, 255})
	}
}

func (p *PPU) Tick(cycles int) {
	// Simplified timing - 1 scanline per 1232 cycles
	// In reality, this should be tied to CPU cycles
	p.VCount = (p.VCount + uint16(cycles/1232)) % 228

	// Render scanline when we're in visible area
	if p.VCount < 160 {
		p.RenderScanline()
	}

	// Frame is complete when we reach VBlank
	if p.VCount == 160 {
		p.frameReady = true
	}
}

func (p *PPU) IsFrameReady() bool {
	return p.frameReady
}

func (p *PPU) ResetFrameReady() {
	p.frameReady = false
}
