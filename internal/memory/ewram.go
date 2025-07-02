package memory

import "GoBA/internal/interfaces"

type EWRAM struct {
	interfaces.MemoryDevice
	data []byte
}

func NewEWRAM() interfaces.MemoryDevice {
	return &EWRAM{
		data: make([]byte, EWRAM_SIZE),
	}
}

func (e *EWRAM) Read8(addr uint32) uint8 {
	return e.data[addr]
}

func (e *EWRAM) Write8(addr uint32, value uint8) {
	e.data[addr] = value
}
