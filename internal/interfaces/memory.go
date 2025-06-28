package interfaces

// MemoryDevice represents a component connected to the bus that handles
// specific memory regions.
type MemoryDevice interface {
	Read8(addr uint32) byte
	ReadHalfWord(addr uint32) uint16
	ReadWord(addr uint32) uint32
	Write8(addr uint32, value byte)
	WriteHalfWord(addr uint32, value uint16)
	WriteWord(addr uint32, value uint32)
	Contains(addr uint32) bool // Indicates if this device handles the given address
}
