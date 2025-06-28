package cpu

import (
	"GoBA/internal/interfaces"
	"fmt"
)

type CPU struct {
	Registers *Registers
	Bus       interfaces.BusInterface
	Cycles    uint64
	// Pipeline is for internal CPU state, often used for prefetching.
	// In a real ARM7TDMI, it's a 3-stage pipeline (Fetch, Decode, Execute).
	// For simplicity, we might just track the next two instructions.
	// However, your FlushPipeline suggests a 2-stage pipeline (current and next).
	// Let's assume a simple 2-stage for now based on your `FlushPipeline` usage.
	Pipeline [2]uint32
}

func NewCPU(bus interfaces.BusInterface) *CPU {
	return &CPU{
		Registers: NewRegisters(),
		Bus:       bus,
	}
}

func (c *CPU) Reset() {
	c.Registers = NewRegisters()
	c.Registers.PC = 0x00000000 // BIOS entry point
	c.Registers.SetMode(SVCMode)
	c.Registers.SetIRQDisabled(true)
	c.Registers.SetFIQDisabled(true)
}

func (c *CPU) Step() {
	// Handle interrupts first
	// if c.checkInterrupts() {
	//   return
	// }

	if c.Registers.IsThumb() {
		instr := uint32(c.Bus.Read16(c.Registers.PC))
		c.Registers.PC += 2 // Thumb: 2-byte prefetch
		c.executeThumb(instr)
	} else { // ARM
		instr := c.Bus.Read32(c.Registers.PC)
		c.Registers.PC += 4 // ARM: 4-byte prefetch
		c.execute_Arm(instr)
	}

	c.Cycles += 1 // Simplified cycle counting
}

func (c *CPU) setFlags(result uint32, carryOut bool, instruction ARMDataProcessingInstruction) {
	// Update Negative flag (N) - set if the result is negative (i.e., bit 31 is set)
	c.Registers.SetFlagN(result&0x80000000 != 0)

	// Update Zero flag (Z) - set if the result is zero
	c.Registers.SetFlagZ(result == 0)

	// Update Carry flag (C) - based on the carry out of the operation or shift
	c.Registers.SetFlagC(carryOut)

	// Update Overflow flag (V) - only for arithmetic operations
	switch instruction.Opcode {
	case ADD, ADC, SUB, SBC, RSB, RSC, CMP, CMN:
		// For arithmetic operations, check for overflow conditions
		rn := c.Registers.GetReg(instruction.Rn)
		rm := c.Registers.GetReg(instruction.Rm)
		overflow := checkOverflow(rn, rm, result, instruction.Opcode)
		c.Registers.SetFlagV(overflow)
	default:
		// For logical operations, Overflow flag isn't affected
		c.Registers.SetFlagV(false)
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
	c.Pipeline[0] = c.Bus.Read32(c.Registers.PC)
	c.Registers.PC += 4
	c.Pipeline[1] = c.Bus.Read32(c.Registers.PC)
	c.Registers.PC += 4
	// After flush, PC points to the instruction after the second fetched one.
}

func (c *CPU) executeThumb(instruction uint32) {
	fmt.Println("Execute Thumb Instruction")
}
