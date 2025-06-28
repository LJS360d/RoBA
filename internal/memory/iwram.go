package memory

import "GoBA/internal/interfaces"

type IWRAM struct {
	interfaces.MemoryDevice
	data []byte
}

func NewIWRAM() *IWRAM {
	return &IWRAM{
		data: make([]byte, IWRAM_SIZE),
	}
}

func (i *IWRAM) Read8(addr uint32) uint8 {
	return i.data[addr]
}

func (i *IWRAM) Write8(addr uint32, value uint8) {
	i.data[addr] = value
}

func (i *IWRAM) Contains(addr uint32) bool {
	return addr >= IWRAM_START && addr <= IWRAM_END
}
