package memory

import (
	"fmt"
)

const (
	BIOS_START  = 0x00000000
	BIOS_END    = 0x00003FFF
	BIOS_SIZE   = BIOS_END - BIOS_START + 1 // 16KB
	EWRAM_START = 0x02000000
	EWRAM_END   = 0x0203FFFF
	EWRAM_SIZE  = EWRAM_END - EWRAM_START + 1 // 256KB
	IWRAM_START = 0x03000000
	IWRAM_END   = 0x03007FFF
	IWRAM_SIZE  = IWRAM_END - IWRAM_START + 1 // 32KB
	VRAM_START  = 0x06000000
	VRAM_END    = 0x06017FFF
	VRAM_SIZE   = VRAM_END - VRAM_START + 1 // 96KB
	OAM_START   = 0x07000000
	OAM_END     = 0x070003FF
	OAM_SIZE    = OAM_END - OAM_START + 1 // 1KB
	ROM_START   = 0x08000000
	ROM_END     = 0x09FFFFFF
	ROM_SIZE    = ROM_END - ROM_START + 1 // 32MB
)

// Memory represents the GBA memory system.
type Memory struct {
	BIOS  []byte
	EWRAM []byte
	IWRAM []byte
	VRAM  []byte
	OAM   []byte
	ROM   []byte
}

// NewMemory initializes the memory map for the GBA.
func NewMemory(romData []byte) *Memory {
	m := &Memory{
		BIOS:  make([]byte, BIOS_SIZE),
		EWRAM: make([]byte, EWRAM_SIZE),
		IWRAM: make([]byte, IWRAM_SIZE),
		VRAM:  make([]byte, VRAM_SIZE),
		OAM:   make([]byte, OAM_SIZE),
		ROM:   romData, // ROM is loaded from the file.
	}
	return m
}

// Read8 reads an 8-bit value from the given memory address.
func (m *Memory) Read8(addr uint32) byte {
	switch {
	case addr >= BIOS_START && addr <= BIOS_END:
		return m.BIOS[addr-BIOS_START]
	case addr >= EWRAM_START && addr <= EWRAM_END:
		return m.EWRAM[addr-EWRAM_START]
	case addr >= IWRAM_START && addr <= IWRAM_END:
		return m.IWRAM[addr-IWRAM_START]
	case addr >= VRAM_START && addr <= VRAM_END:
		return m.VRAM[addr-VRAM_START]
	case addr >= OAM_START && addr <= OAM_END:
		return m.OAM[addr-OAM_START]
	case addr >= ROM_START && addr <= ROM_END:
		return m.ROM[addr-ROM_START]
	default:
		panic("read from out of bounds memory address 0x" + fmt.Sprintf("%08X", addr))
	}
}

// Write8 writes an 8-bit value to the given memory address.
func (m *Memory) Write8(addr uint32, value byte) {
	switch {
	case addr >= EWRAM_START && addr <= EWRAM_END:
		m.EWRAM[addr-EWRAM_START] = value
	case addr >= IWRAM_START && addr <= IWRAM_END:
		m.IWRAM[addr-IWRAM_START] = value
	case addr >= VRAM_START && addr <= VRAM_END:
		m.VRAM[addr-VRAM_START] = value
	case addr >= OAM_START && addr <= OAM_END:
		m.OAM[addr-OAM_START] = value
	default:
		panic(fmt.Sprintf("attempted write [0x%08x] to out of bounds memory address 0x%08x", value, addr))
	}
}
