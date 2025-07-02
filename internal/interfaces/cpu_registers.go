package interfaces

type RegistersInterface interface {
	GetFlagC() bool
	GetFlagN() bool
	GetFlagV() bool
	GetFlagZ() bool
	GetCPSR() uint32
	SetCPSR(uint32)
	GetPC() uint32
	SetPC(uint32)
	GetMode() uint8
	GetReg(uint8) uint32
	GetSPSR() uint32
	IsFIQDisabled() bool
	IsIRQDisabled() bool
	IsThumb() bool
	SetFIQDisabled(bool)
	SetIRQDisabled(bool)
	SetFlagC(bool)
	SetFlagN(bool)
	SetFlagV(bool)
	SetFlagZ(bool)
	SetMode(uint8)
	SetReg(uint8, uint32)
	SetSPSR(uint32)
	SetThumbState(bool)
}
