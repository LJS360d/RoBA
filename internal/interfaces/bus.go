package interfaces

import "GoBA/internal/io"

type BusInterface interface {
	GetIORegsPtr() *io.IORegs
	Read8(uint32) uint8
	Write8(uint32, uint8)
	Read16(uint32) uint16
	Write16(uint32, uint16)
	Read32(uint32) uint32
	Write32(uint32, uint32)
	Tick(int)
}
