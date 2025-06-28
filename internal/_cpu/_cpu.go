package cpu

import (
	"GoBA/internal/memory"
)

const (
	ARM_MODE   = 0
	THUMB_MODE = 1
)

// CPU represents the Game Boy Advance CPU.
type CPU struct {
	Registers [16]uint32 // 16 general-purpose registers (R0-R15)
	PC        uint32     // Program Counter
	Mode      uint8      // ARM/Thumb mode
	CPSR      CPSR
	Memory    *memory.Memory
}

type CPSR struct {
	N bool // Negative flag
	Z bool // Zero flag
	C bool // Carry flag
	V bool // Overflow flag
}

// NewCPU initializes a new CPU instance.
func NewCPU(mem *memory.Memory) *CPU {
	return &CPU{
		PC:     0x08000000, // Start execution at the beginning of the ROM
		Mode:   ARM_MODE,
		Memory: mem,
	}
}

// Starts the CPU execution loop.
func (c *CPU) Run() {
	for {
		instructions := c.fetch()
		c.execute(instructions)

		// Break condition for demonstration purposes (to avoid infinite loop)
		if c.PC >= 0x08000000+uint32(len(c.Memory.ROM)) {
			break
		}
	}
}

// Fetches the next instruction from memory.
func (c *CPU) fetch() uint32 {
	if c.Mode == THUMB_MODE {
		instruction := uint32(c.Memory.Read8(c.PC)) |
			(uint32(c.Memory.Read8(c.PC+1)) << 8)
		return instruction
	}
	instruction := uint32(c.Memory.Read8(c.PC)) |
		(uint32(c.Memory.Read8(c.PC+1)) << 8) |
		(uint32(c.Memory.Read8(c.PC+2)) << 16) |
		(uint32(c.Memory.Read8(c.PC+3)) << 24)
	return instruction
}

// Executes a given instruction.
func (c *CPU) execute(instruction uint32) {
	// Check the mode and handle instructions accordingly
	if c.Mode == THUMB_MODE {
		c.execute_Thumb(uint16(instruction))
		c.PC += 2 // Move to the next instruction
	} else {
		c.Execute_Arm(instruction)
		c.PC += 4 // Move to the next instruction
	}
}

func (c *CPU) setFlags(result uint32, carryOut bool, instruction ARMDataProcessingInstruction) {
	// Update Negative flag (N) - set if the result is negative (i.e., bit 31 is set)
	c.CPSR.N = (result & 0x80000000) != 0

	// Update Zero flag (Z) - set if the result is zero
	c.CPSR.Z = (result == 0)

	// Update Carry flag (C) - based on the carry out of the operation or shift
	c.CPSR.C = carryOut

	// Update Overflow flag (V) - only for arithmetic operations
	switch instruction.Opcode {
	case ADD, ADC, SUB, SBC, RSB, RSC, CMP, CMN:
		// For arithmetic operations, check for overflow conditions
		rn := c.Registers[instruction.Rn]
		rm := c.Registers[instruction.Rm]
		overflow := checkOverflow(rn, rm, result, instruction.Opcode)
		c.CPSR.V = overflow
	default:
		// For logical operations, Overflow flag isn't affected
		c.CPSR.V = false
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
