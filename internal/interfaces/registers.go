package interfaces

type RegistersInterface interface {
	GetFlagC() bool
	GetFlagN() bool
	GetFlagV() bool
	GetFlagZ() bool
	GetMode() uint8
	GetReg(uint8) uint32
	GetSPSR() uint32
	ISFIQDisabled() bool
	ISIRQDisabled() bool
	IsThumb() bool
	SetFIQDisabled() bool
	SetFlagC(bool)
	SetFlagN(bool)
	SetFlagV(bool)
	SetFlagZ(bool)
	SetIRQDisabled(bool)
	SetMode(uint8)
	SetReg(uint8, uint32)
	SetSPSR(uint32)
	SetThumbState(bool)
}
