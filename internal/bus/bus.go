package bus

import (
	"log"

	"GoBA/internal/apu"
	"GoBA/internal/cartridge"
	"GoBA/internal/dma"
	"GoBA/internal/interfaces"
	"GoBA/internal/io"
	"GoBA/internal/joypad"
	"GoBA/internal/memory"
	"GoBA/internal/ppu"
	"GoBA/internal/timer"
	"GoBA/util/dbg"
)

// GBA Memory Map Constants
const (
	BIOSAddrStart = 0x00000000
	BIOSAddrEnd   = 0x00003FFF
	BIOSSize      = BIOSAddrEnd - BIOSAddrStart + 1 // 16KB

	EWRAMAddrStart = 0x02000000
	EWRAMAddrEnd   = 0x0203FFFF
	EWRAMSize      = EWRAMAddrEnd - EWRAMAddrStart + 1 // 256KB
	EWRAMMirrorEnd = 0x02FFFFFF                        // Mirrored up to 0x02FFFFFF

	IWRAMAddrStart = 0x03000000
	IWRAMAddrEnd   = 0x03007FFF
	IWRAMSize      = IWRAMAddrEnd - IWRAMAddrStart + 1 // 32KB
	IWRAMMirrorEnd = 0x03FFFFFF                        // Mirrored up to 0x03FFFFFF

	IOAddrStart = 0x04000000
	IOAddrEnd   = 0x040003FF // Main I/O registers block
	IOSize      = IOAddrEnd - IOAddrStart + 1
	IOMirrorEnd = 0x04FFFFFF // Mirrored up to 0x04FFFFFF

	PALRAMAddrStart = 0x05000000
	PALRAMAddrEnd   = 0x050003FF
	PALRAMSize      = PALRAMAddrEnd - PALRAMAddrStart + 1 // 1KB
	PALRAMMirrorEnd = 0x05FFFFFF                          // Mirrored

	VRAMAddrStart = 0x06000000
	VRAMAddrEnd   = 0x06017FFF
	VRAMSize      = VRAMAddrEnd - VRAMAddrStart + 1 // 96KB
	VRAMMirrorEnd = 0x06FFFFFF                      // Mirrored (partially, up to 0x0601FFFF for some mirrors)

	OAMAddrStart = 0x07000000
	OAMAddrEnd   = 0x070003FF
	OAMSize      = OAMAddrEnd - OAMAddrStart + 1 // 1KB
	OAMMirrorEnd = 0x07FFFFFF                    // Mirrored

	GamePakAddrStartWS0 = 0x08000000
	GamePakAddrEndWS0   = 0x09FFFFFF
	GamePakAddrStartWS1 = 0x0A000000
	GamePakAddrEndWS1   = 0x0BFFFFFF
	GamePakAddrStartWS2 = 0x0C000000
	GamePakAddrEndWS2   = 0x0DFFFFFF

	GamePakSRAMAddrStart = 0x0E000000
	GamePakSRAMAddrEnd   = 0x0E00FFFF // Max 64KB, often smaller
	GamePakSRAMSize      = GamePakSRAMAddrEnd - GamePakSRAMAddrStart + 1
)

// Bus connects the CPU to various memory-mapped components.
type Bus struct {
	interfaces.BusInterface
	BIOS  *memory.BIOS
	EWRAM *memory.EWRAM // On-board Work RAM
	IWRAM *memory.IWRAM // On-chip Work RAM

	// many registers have side effects.
	// main I/O block (0x04000000 - 0x040003FF)
	IORegs *io.IORegs

	PPU       *ppu.PPU             // Handles PALRAM, VRAM, OAM and PPU I/O regs
	Cartridge *cartridge.Cartridge // Handles Game Pak ROM and SRAM

	// TODO: Add Serial, etc. (?)
	DMAController *dma.Controller
	Timers        *timer.Controller
	APU           *apu.APU
	Keypad        *joypad.Joypad

	// Cycle counting - to be implemented
	CycleCount uint64
}

// NewBus creates a new Bus instance.
// Components like PPU and Cartridge should be initialized and passed in.
func NewBus(bios *memory.BIOS, ewram *memory.EWRAM, iwram *memory.IWRAM, ppu *ppu.PPU, cart *cartridge.Cartridge, ioRegs *io.IORegs) *Bus {
	if bios == nil || ewram == nil || iwram == nil || ppu == nil || cart == nil {
		log.Fatalf("Bus: Cannot initialize with nil components")
	}
	return &Bus{
		BIOS:      bios,
		EWRAM:     ewram,
		IWRAM:     iwram,
		PPU:       ppu,
		Cartridge: cart,
		IORegs:    ioRegs,
	}
}

// Read8 reads a byte from the memory map.
func (b *Bus) Read8(addr uint32) uint8 {
	// Apply address masking for mirrors if necessary before switch case
	// For example, EWRAM is 256KB but mirrored up to 0x02FFFFFF.
	// addr &= 0x0203FFFF for EWRAM if addr is in its mirrored range.

	switch {
	// BIOS (0x00000000 - 0x00003FFF)
	case addr >= BIOSAddrStart && addr <= BIOSAddrEnd:
		// BIOS is only accessible if PC is within BIOS region or if System Control Reg (0x4000800) bit 0 is set.
		// For now, let's assume it's accessible. This logic will be refined.
		// Also, BIOS is read-only.
		return b.BIOS.Read8(addr - BIOSAddrStart)

	// EWRAM (0x02000000 - 0x02FFFFFF, actual 0x02000000 - 0x0203FFFF)
	case addr >= EWRAMAddrStart && addr <= EWRAMMirrorEnd:
		return b.EWRAM.Read8((addr - EWRAMAddrStart) % EWRAMSize)

	// IWRAM (0x03000000 - 0x03FFFFFF, actual 0x03000000 - 0x03007FFF)
	case addr >= IWRAMAddrStart && addr <= IWRAMMirrorEnd:
		return b.IWRAM.Read8((addr - IWRAMAddrStart) % IWRAMSize)

	// I/O Registers (0x04000000 - 0x04FFFFFF, actual 0x04000000 - 0x040003FF)
	case addr >= IOAddrStart && addr <= IOMirrorEnd:
		maskedAddr := (addr - IOAddrStart) % IOSize
		// Many I/O registers are handled by PPU, Timers, DMA, etc.
		// This switch needs to delegate to those components.
		// For now, a simplified direct read from a placeholder array.
		// TODO: Delegate to specific I/O handlers (PPU, DMA, Timers, etc.)
		if b.PPU.IsPPUIORegister(maskedAddr) {
			return b.PPU.ReadIORegister8(maskedAddr)
		}
		// Add other I/O component checks here (DMA, Timers, Sound, Keypad, Serial)
		// Example: if dma.IsDMAIORegister(maskedAddr) { return b.DMAController.Read(maskedAddr) }

		// Fallback for unhandled I/O registers (should log or return open bus value)
		dbg.Printf("Bus: Unhandled 8-bit read from I/O addr %08X (masked %04X)\n", addr, maskedAddr)
		if maskedAddr < b.IORegs.Size() {
			return b.IORegs.GetReg(maskedAddr)
		}
		return 0xFF // Open bus value

	// Palette RAM (0x05000000 - 0x05FFFFFF, actual 0x05000000 - 0x050003FF)
	case addr >= PALRAMAddrStart && addr <= PALRAMMirrorEnd:
		return b.PPU.ReadPaletteRAM8((addr - PALRAMAddrStart) % PALRAMSize)

	// VRAM (0x06000000 - 0x06FFFFFF, actual 0x06000000 - 0x06017FFF)
	// VRAM mirroring is a bit complex (e.g. 06010000-0601FFFF mirrors 06000000-0600FFFF in Bitmap mode for Page 1)
	case addr >= VRAMAddrStart && addr <= VRAMMirrorEnd:
		// Basic mirroring for now, PPU will handle complex cases.
		return b.PPU.ReadVRAM8((addr - VRAMAddrStart) % VRAMSize) // Simplified, PPU should handle exact mapping

	// OAM (0x07000000 - 0x07FFFFFF, actual 0x07000000 - 0x070003FF)
	case addr >= OAMAddrStart && addr <= OAMMirrorEnd:
		return b.PPU.ReadOAM8((addr - OAMAddrStart) % OAMSize)

	// Game Pak ROM (0x08000000 - 0x0DFFFFFF)
	case (addr >= GamePakAddrStartWS0 && addr <= GamePakAddrEndWS0) ||
		(addr >= GamePakAddrStartWS1 && addr <= GamePakAddrEndWS1) ||
		(addr >= GamePakAddrStartWS2 && addr <= GamePakAddrEndWS2):
		// Wait states are handled by cycle accounting, not directly by address mapping here.
		// The cartridge handles the actual ROM data.
		return b.Cartridge.ReadROM8(addr) // Cartridge needs to handle the full 08000000-0DFFFFFF range

	// Game Pak SRAM (0x0E000000 - 0x0E00FFFF, mirrored up to 0x0FFFFFFF by some sources, but often just this range)
	case addr >= GamePakSRAMAddrStart && addr <= GamePakSRAMAddrEnd: // Simplified range for now
		return b.Cartridge.ReadSRAM8(addr - GamePakSRAMAddrStart)

	default:
		// Open bus read - GBA returns prefetch buffer or specific values.
		// For now, return 0xFF and log
		// dbg.Printf("Bus: Unhandled 8-bit read from address %08X\n", addr)
		return 0xFF // Or specific open bus behavior if known
	}
}

// Write8 writes a byte to the specified memory address.
func (b *Bus) Write8(addr uint32, value uint8) {
	switch {
	// BIOS (Read-Only)
	case /* addr >= 0x00000000 &&  */ addr <= 0x00003FFF:
		// Attempted write to BIOS. GBA BIOS is Read-Only. Ignore or log an error.
		dbg.Printf("WARN: Attempted write to Read-Only BIOS at %08X\n", addr)
		return
	// EWRAM (External Work RAM)
	case addr >= 0x02000000 && addr <= 0x0203FFFF:
		// Remap address to EWRAM's local offset (0x02000000 is base)
		b.EWRAM.Write8(addr-0x02000000, value)
	// IWRAM (Internal Work RAM)
	case addr >= 0x03000000 && addr <= 0x03007FFF:
		// Remap address to IWRAM's local offset (0x03000000 is base)
		b.IWRAM.Write8(addr-0x03000000, value)
	// I/O Registers
	case addr >= 0x04000000 && addr <= 0x040003FE:
		// Remap address to I/O registers' local offset (0x04000000 is base)
		b.IORegs.SetReg(addr-0x04000000, value)
	// PPU VRAM (Video RAM)
	case addr >= 0x06000000 && addr <= 0x06017FFF:
		// Remap address to VRAM's local offset (0x06000000 is base)
		b.PPU.WriteVRAM8(addr-0x06000000, value) // Assuming PPU has a WriteVRAM8
	// PPU OAM (Object Attribute Memory)
	case addr >= 0x07000000 && addr <= 0x070003FF:
		// Remap address to OAM's local offset (0x07000000 is base)
		b.PPU.WriteOAM8(addr-0x07000000, value) // Assuming PPU has a WriteOAM8
	// Game Pak ROM/Flash (WS0, WS1, WS2) - Read-Only
	case addr >= 0x08000000 && addr <= 0x0DFFFFFF:
		// Attempted write to ROM/Flash. This region is Read-Only. Ignore or log
		dbg.Printf("WARN: Attempted write to Read-Only ROM at %08X\n", addr)
		return
	// Game Pak SRAM (Save RAM) - Writable
	case addr >= 0x0E000000 && addr <= 0x0E00FFFF:
		// This is the Save RAM region. It is writable.
		// Remap address to Cartridge's SRAM local offset (0x0E000000 is base)
		b.Cartridge.WriteSRAM8(addr-0x0E000000, value) // Assuming your Cartridge has a WriteSRAM8
	default:
		// Unhandled or open bus address
		dbg.Printf("Bus: Unhandled 8-bit write to address %08X\n", addr)
	}
}

// Read16 reads a 16-bit value (little-endian).
func (b *Bus) Read16(addr uint32) uint16 {
	// Ensure address is halfword aligned for many regions, though ARM7TDMI can handle unaligned.
	// GBA hardware might have specific alignment penalties or behaviors.
	// For simplicity, we assume CPU handles alignment for now, bus provides data.
	// TODO: Add cycle penalties for unaligned access if necessary.
	// TODO: Consider bus access timing (wait states).

	// Read two bytes and combine them in little-endian order.
	lo := uint16(b.Read8(addr))
	hi := uint16(b.Read8(addr + 1))
	return (hi << 8) | lo
}

// Write16 writes a 16-bit value (little-endian).
func (b *Bus) Write16(addr uint32, value uint16) {
	// TODO: Add cycle penalties for unaligned access if necessary.
	// TODO: Consider bus access timing (wait states).

	lo := uint8(value & 0xFF)
	hi := uint8((value >> 8) & 0xFF)
	b.Write8(addr, lo)
	b.Write8(addr+1, hi)
}

// Read32 reads a 32-bit value (little-endian).
func (b *Bus) Read32(addr uint32) uint32 {
	// TODO: Add cycle penalties for unaligned access if necessary.
	// TODO: Consider bus access timing (wait states).

	b0 := uint32(b.Read8(addr))
	b1 := uint32(b.Read8(addr + 1))
	b2 := uint32(b.Read8(addr + 2))
	b3 := uint32(b.Read8(addr + 3))
	return (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
}

// Write32 writes a 32-bit value (little-endian).
func (b *Bus) Write32(addr uint32, value uint32) {
	// TODO: Add cycle penalties for unaligned access if necessary.
	// TODO: Consider bus access timing (wait states).

	b0 := uint8(value & 0xFF)
	b1 := uint8((value >> 8) & 0xFF)
	b2 := uint8((value >> 16) & 0xFF)
	b3 := uint8((value >> 24) & 0xFF)
	b.Write8(addr, b0)
	b.Write8(addr+1, b1)
	b.Write8(addr+2, b2)
	b.Write8(addr+3, b3)
}

// Tick advances the bus state by a number of cycles.
// This will be used for synchronizing components.
func (b *Bus) Tick(cycles int) {
	b.CycleCount += uint64(cycles)
	b.PPU.Tick(cycles)
	b.Timers.Tick(cycles)
	b.DMAController.Tick(cycles)
	b.APU.Tick(cycles)
}
