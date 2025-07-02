package interfaces

// CPUInterface represents the ARM7TDMI CPU component
type CPUInterface interface {
	Registers() RegistersInterface
	Bus() BusInterface
	Reset()
	Step()
	Execute(instruction uint32) error
}
