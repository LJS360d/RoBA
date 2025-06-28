package memory

import (
	"GoBA/embedded"
	"GoBA/internal/interfaces"
	_ "embed"
	"fmt"
)

// BIOS represents the GBA's internal Boot ROM.
type BIOS struct {
	interfaces.MemoryDevice
	data []byte // The loaded BIOS ROM data
}

// NewBIOS loads the GBA BIOS ROM from the specified file path.
func NewBIOS() *BIOS {
	return &BIOS{
		data: embedded.BIOSData,
	}
}

// ReadByte reads a single byte from the BIOS at the given absolute address.
// It handles the BIOS memory region (0x00000000 - 0x00003FFF).
func (b *BIOS) Read8(addr uint32) byte {
	if addr >= BIOS_START && addr <= BIOS_END {
		return b.data[addr-BIOS_START]
	}
	// This should ideally not happen if the Bus correctly routes addresses.
	panic(fmt.Sprintf("BIOS: Attempted to read byte from out-of-bounds address: 0x%X", addr))
}

// ReadHalfWord reads a 16-bit half-word from the BIOS.
func (b *BIOS) ReadHalfWord(addr uint32) uint16 {
	if addr >= BIOS_START && addr <= BIOS_END-1 { // -1 to ensure room for 2 bytes
		// Ensure aligned access for half-words in case of strictness later,
		// although GBA usually allows unaligned. For ROM, it's fine.
		low := uint16(b.data[addr-BIOS_START])
		high := uint16(b.data[addr-BIOS_START+1])
		return low | (high << 8)
	}
	panic(fmt.Sprintf("BIOS: Attempted to read half-word from out-of-bounds or unaligned address: 0x%X", addr))
}

// ReadWord reads a 32-bit word from the BIOS.
func (b *BIOS) ReadWord(addr uint32) uint32 {
	if addr >= BIOS_START && addr <= BIOS_END-3 { // -3 to ensure room for 4 bytes
		b0 := uint32(b.data[addr-BIOS_START])
		b1 := uint32(b.data[addr-BIOS_START+1])
		b2 := uint32(b.data[addr-BIOS_START+2])
		b3 := uint32(b.data[addr-BIOS_START+3])
		return b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
	}
	panic(fmt.Sprintf("BIOS: Attempted to read word from out-of-bounds or unaligned address: 0x%X", addr))
}

// WriteByte attempts to write a byte to the BIOS.
// BIOS is read-only, so this operation panics.
func (b *BIOS) Write8(addr uint32, value byte) {
	panic(fmt.Sprintf("BIOS: Attempted to write 0x%X to read-only BIOS at address 0x%X", value, addr))
}

// WriteHalfWord attempts to write a half-word to the BIOS.
// BIOS is read-only, so this operation panics.
func (b *BIOS) WriteHalfWord(addr uint32, value uint16) {
	panic(fmt.Sprintf("BIOS: Attempted to write 0x%X to read-only BIOS at address 0x%X", value, addr))
}

// WriteWord attempts to write a word to the BIOS.
// BIOS is read-only, so this operation panics.
func (b *BIOS) WriteWord(addr uint32, value uint32) {
	panic(fmt.Sprintf("BIOS: Attempted to write 0x%X to read-only BIOS at address 0x%X", value, addr))
}
