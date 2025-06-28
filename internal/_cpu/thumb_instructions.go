package cpu

// ARM Instructions
// Each instruction is defined with its opcode and a brief description.

// Thumb Instructions
// Each Thumb instruction is defined with its opcode and a brief description.
const (
	// Data Processing Instructions
	THUMB_ADD = 0x0C00 // Add two registers: Rn + Rm -> Rd (Thumb format)
	THUMB_SUB = 0x0C01 // Subtract one register from another: Rn - Rm -> Rd (Thumb format)
	THUMB_MOV = 0x1C00 // Move immediate value to register: Rd = imm
	THUMB_LDR = 0x6800 // Load a register from memory: Rd = [Rn + offset]
	THUMB_STR = 0x6000 // Store a register to memory: [Rn + offset] = Rd

	// Branch Instructions
	THUMB_B  = 0xD000 // Branch to a label: jump to address (Thumb format)
	THUMB_BL = 0xF000 // Branch with link: jump and save return address (Thumb format)

	// Control Instructions
	THUMB_NOP = 0xBF00 // No operation: do nothing (Thumb format)
	// TODO: Map remaining Thumb instructions here as needed.
)
