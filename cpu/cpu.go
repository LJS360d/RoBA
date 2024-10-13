package cpu

import (
	"GoBA/memory"
	"fmt"
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
		instructions := c.Fetch()
		c.Execute(instructions)

		// Break condition for demonstration purposes (to avoid infinite loop)
		if c.PC >= 0x08000000+uint32(len(c.Memory.ROM)) {
			break
		}
	}
}

// Fetches the next instruction from memory.
func (c *CPU) Fetch() uint32 {
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
func (c *CPU) Execute(instruction uint32) {
	// Check the mode and handle instructions accordingly
	if c.Mode == THUMB_MODE {
		c.Execute_Thumb(instruction)
		c.PC += 2 // Move to the next instruction
	} else {
		c.Execute_Arm(instruction)
		c.PC += 4 // Move to the next instruction
	}
}

func applyShift(value uint32, shiftType ARMShiftType, shiftAmount uint32) uint32 {
	switch shiftType {
	case LSL: // LSL
		return value << shiftAmount
	case LSR: // LSR
		if shiftAmount == 0 {
			return 0 // LSR #32 means the result is 0
		}
		return value >> shiftAmount
	case ASR: // ASR
		if shiftAmount == 0 {
			// ASR #32 replicates sign bit
			if value&0x80000000 != 0 {
				return 0xFFFFFFFF
			}
			return 0
		}
		return uint32(int32(value) >> shiftAmount) // Signed shift
	case ROR: // ROR
		shiftAmount %= 32
		return (value >> shiftAmount) | (value << (32 - shiftAmount))
	}
	return value // Default case, should not be hit
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

func (c *CPU) Execute_Thumb(instruction uint32) {
	switch instruction & 0x0FFF { // Masking to check opcode bits
	case THUMB_ADD:
		c.ExecAdd_Thumb(instruction)
	// Add more Thumb instructions here
	default:
		fmt.Printf("Unknown Thumb instruction: 0x%04X\n", instruction)
	}
}

// Execute ARM instruction based on opcode.
func (c *CPU) Execute_Arm(instruction uint32) {
	decoded := ParseInstruction_Arm(instruction)
	switch inst := decoded.(type) {
	case ARMDataProcessingInstruction:
		// Handle DataProcessingInstruction
		switch inst.Opcode {
		case AND:
			c.ExecAnd_Arm(inst)
		case EOR:
			c.ExecEor_Arm(inst)
		case SUB:
		case RSB:
		case ADD:
		case ADC:
		case SBC:
		case RSC:
		case TST:
		case TEQ:
		case CMP:
		case CMN:
		case ORR:
			c.ExecOrr_Arm(inst)
		case MOV:
		case BIC:
		case MVN:

		}

	case ARMLoadStoreInstruction:
		// Handle LoadStoreInstruction

	case ARMBranchInstruction:
		// Handle BranchInstruction

	case ARMControlInstruction:
		// Handle ControlInstruction

	default:
		// Handle unknown instruction
		fmt.Println("Unknown Instruction type")
	}
}

// #############################
// Thumb Instruction Implementations
// #############################

// Executes Thumb ADD instruction.
func (c *CPU) ExecAdd_Thumb(instruction uint32) {
	rn := (instruction >> 3) & 0x07 // Bits 3-5 for Rn
	rm := instruction & 0x07        // Bits 0-2 for Rm

	result := c.Registers[rm] + c.Registers[rn]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("Thumb ADD R%d, R%d: Result = %d\n", rn, rm, result)
}

// #############################
// ARM Instruction Implementations
// #############################

// Execute ADD instruction
func (c *CPU) ExecAdd_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F
	rm := (instruction >> 0) & 0x0F
	rs := (instruction >> 8) & 0x0F

	result := c.Registers[rm] + c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM ADD R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

// Execute SUB instruction
func (c *CPU) ExecSub_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F
	rm := (instruction >> 0) & 0x0F
	rs := (instruction >> 8) & 0x0F

	result := c.Registers[rn] - c.Registers[rs]
	c.Registers[rm] = result // Store result in Rm
	fmt.Printf("ARM SUB R%d, R%d, R%d: Result = %d\n", rm, rn, rs, result)
}

// Execute MUL instruction
func (c *CPU) ExecMul_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F
	rm := (instruction >> 0) & 0x0F
	rs := (instruction >> 8) & 0x0F

	result := c.Registers[rm] * c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM MUL R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

func (c *CPU) ExecAnd_Arm(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the AND operation between Rn and operand2
	result := c.Registers[rn] & op2

	// Store result in the destination register (Rd)
	c.Registers[instruction.Rd] = result

	// Set flags if the instruction specifies (S = true)
	if instruction.S && instruction.Rd != 15 {
		c.setFlags(result, carryOut, instruction)
	}

	fmt.Printf("ARM AND R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// Execute ORR instruction
func (c *CPU) ExecOrr_Arm(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the ORR operation between Rn and operand2
	result := c.Registers[rn] | op2

	// Store result in the destination register (Rd)
	c.Registers[instruction.Rd] = result

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
	fmt.Printf("ARM ORR R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// Execute EOR instruction
func (c *CPU) ExecEor_Arm(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the EOR operation between Rn and operand2
	result := c.Registers[rn] ^ op2

	// Store result in the destination register (Rd)
	c.Registers[instruction.Rd] = result

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

	fmt.Printf("ARM EOR R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

func (c *CPU) calcOp2(instruction ARMDataProcessingInstruction) (uint32, bool) {
	if instruction.I {
		// Immediate operand case: instruction uses a rotated immediate value
		// Apply ROR to the immediate value (instruction.Nn) by instruction.Is * 2
		rotatedImmediate := applyShift(instruction.Nn, ROR, instruction.Is*2)
		carryOut := (instruction.Is != 0) && (instruction.Nn&0x80000000 != 0) // Carry from the ROR
		return rotatedImmediate, carryOut
	} else {
		// Register operand case: Rm can be shifted by Is or Rs
		rm := c.Registers[instruction.Rm]
		if instruction.ShiftType < 4 {
			return applyShift(rm, instruction.ShiftType, instruction.Is), (rm & (1 << (instruction.Is - 1))) != 0
		}
	}

	return 0, false // Default case, shouldn't be hit
}
