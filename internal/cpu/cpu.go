package cpu

import (
	"GoBA/internal/interfaces"
	"GoBA/internal/memory"
)

type CPU struct {
	interfaces.CPUInterface
	registers interfaces.RegistersInterface
	bus       interfaces.BusInterface
	cycles    uint64
	// pipeline is for internal CPU state, often used for prefetching.
	// In a real ARM7TDMI, it's a 3-stage pipeline (Fetch, Decode, Execute).
	// For simplicity, we might just track the next two instructions.
	// However, your FlushPipeline suggests a 2-stage pipeline (current and next).
	// Let's assume a simple 2-stage for now based on your `FlushPipeline` usage.
	pipeline [2]uint32
}

func NewCPU(bus interfaces.BusInterface) interfaces.CPUInterface {
	return &CPU{
		registers: NewRegisters(),
		bus:       bus,
	}
}

func (c *CPU) Registers() interfaces.RegistersInterface {
	return c.registers
}

func (c *CPU) Bus() interfaces.BusInterface {
	return c.bus
}

func (c *CPU) Reset() {
	c.registers = NewRegisters()
	c.registers.SetPC(memory.BIOS_START) // BIOS entry point
	c.registers.SetMode(SVCMode)
	c.registers.SetIRQDisabled(true)
	c.registers.SetFIQDisabled(true)
}

func (c *CPU) Step() {
	// Handle interrupts first
	// if c.checkInterrupts() {
	//   return
	// }
	PC := c.registers.GetPC()
	if c.registers.IsThumb() {
		instr := uint32(c.bus.Read16(PC))
		c.registers.SetPC(PC + 2) // Thumb: 2-byte prefetch
		c.executeThumb(instr)
	} else { // ARM
		instr := c.bus.Read32(PC)
		c.registers.SetPC(PC + 4) // ARM: 4-byte prefetch
		c.execute_Arm(instr)
	}

	c.cycles += 1 // Simplified cycle counting
}

func (c *CPU) setFlags(result uint32, carryOut bool, instruction ARMInstruction) {
	// Update Negative flag (N) - set if the result is negative (i.e., bit 31 is set)
	c.registers.SetFlagN(result&0x80000000 != 0)

	// Update Zero flag (Z) - set if the result is zero
	c.registers.SetFlagZ(result == 0)

	// Update Carry flag (C) - based on the carry out of the operation or shift
	c.registers.SetFlagC(carryOut)

	// Update Overflow flag (V) - only for arithmetic operations
	switch instruction.OpcodeDP {
	case ADD, ADC, SUB, SBC, RSB, RSC, CMP, CMN:
		// For arithmetic operations, check for overflow conditions
		rn := c.registers.GetReg(instruction.Rn)
		rm := c.registers.GetReg(instruction.Rm)
		overflow := checkOverflow(rn, rm, result, instruction.OpcodeDP)
		c.registers.SetFlagV(overflow)
	default:
		// For logical operations, Overflow flag isn't affected
		c.registers.SetFlagV(false)
	}
}

func checkOverflow(rn uint32, rm uint32, result uint32, opcode ARMDataProcessingOperation) bool {
	switch opcode {
	case ADD, ADC, CMN:
		// Overflow occurs when the sign bit of the result differs from both operands
		return ((rn ^ result) & (rm ^ result) & 0x80000000) != 0
	case SUB, SBC, CMP, RSB, RSC:
		// Overflow occurs when the sign bit of the operands differ
		return ((rn ^ rm) & (rn ^ result) & 0x80000000) != 0
	default:
		return false
	}
}

// FlushPipeline resets the instruction pipeline.
// In a 3-stage pipeline (Fetch, Decode, Execute), PC points to Fetch.
// After Fetch, PC is incremented. So when Execute runs, PC is (current_instruction_address + 8).
// Your pipeline seems to be 2-stage (current and next).
// This function would typically refill the pipeline after a branch or exception.
func (c *CPU) FlushPipeline() {
	// When flushing, PC is already pointing to the next instruction to fetch.
	// So, we fetch the instruction at PC, then increment PC, then fetch again.
	// This simulates refilling the pipeline.
	PC := c.registers.GetPC()
	c.pipeline[0] = c.bus.Read32(PC)
	c.registers.SetPC(PC + 4)
	c.pipeline[1] = c.bus.Read32(PC)
	c.registers.SetPC(PC + 4)
	// After flush, PC points to the instruction after the second fetched one.
}

func (c *CPU) executeThumb(instruction uint32) {
	panic("unimplemented")
}
