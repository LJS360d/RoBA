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
	Memory    *memory.Memory
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
		fmt.Printf("Data Processing Instruction, Opcode: %d, Immediate: %t\n", inst.Opcode, inst.I)

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
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rm] + c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM ADD R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

// Execute SUB instruction
func (c *CPU) ExecSub_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rn] - c.Registers[rs]
	c.Registers[rm] = result // Store result in Rm
	fmt.Printf("ARM SUB R%d, R%d, R%d: Result = %d\n", rm, rn, rs, result)
}

// Execute MUL instruction
func (c *CPU) ExecMul_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rm] * c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM MUL R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

// Execute AND instruction
func (c *CPU) ExecAnd_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rm] & c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM AND R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

// Execute ORR instruction
func (c *CPU) ExecOrr_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rm] | c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM ORR R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}

// Execute EOR instruction
func (c *CPU) ExecEor_Arm(instruction uint32) {
	rn := (instruction >> 16) & 0x0F // Bits 16-19 for Rn
	rm := (instruction >> 0) & 0x0F  // Bits 0-3 for Rm
	rs := (instruction >> 8) & 0x0F  // Bits 8-11 for Rs

	result := c.Registers[rm] ^ c.Registers[rs]
	c.Registers[rn] = result // Store result in Rn
	fmt.Printf("ARM EOR R%d, R%d, R%d: Result = %d\n", rn, rm, rs, result)
}
