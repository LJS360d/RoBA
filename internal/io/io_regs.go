package io

type IORegs struct {
	regs [0x400]byte
}

func NewIORegs() *IORegs {
	return &IORegs{}
}

func (i *IORegs) GetReg(addr uint32) uint8 {
	return i.regs[addr]
}

func (i *IORegs) SetReg(addr uint32, value uint8) {
	i.regs[addr] = value
}

func (i *IORegs) Size() uint32 {
	return uint32(len(i.regs))
}
