package cpu

// ARM Instructions
// Each instruction is defined with its opcode and a brief description.

// Data Processing Instructions
const (
	// 0xE0000000 - 0xEFFFFFFF: Conditional execution
	ADD = 0xE0800000 // Add two registers: Rn + Rm -> Rd
	SUB = 0xE0400000 // Subtract one register from another: Rn - Rm -> Rd
	MUL = 0xE0000090 // Multiply two registers: Rn * Rm -> Rd
	DIV = 0xE0000091 // Divide one register by another (software): Rn / Rm -> Rd (not directly supported)
	ADC = 0xE0A00000 // Add with carry: Rn + Rm + Carry -> Rd
	SBC = 0xE0B00000 // Subtract with carry: Rn - Rm - Not Carry -> Rd
	ORR = 0xE1800000 // Bitwise OR: Rn | Rm -> Rd
	AND = 0xE0000000 // Bitwise AND: Rn & Rm -> Rd
	EOR = 0xE0200000 // Bitwise Exclusive OR: Rn ^ Rm -> Rd
	BIC = 0xE0C00000 // Bit clear (AND NOT): Rn & ~Rm -> Rd

	// Load and Store Instructions
	LDR = 0xE5900000 // Load a register from memory: Rd = [Rn + offset]
	STR = 0xE5800000 // Store a register to memory: [Rn + offset] = Rd
	LDM = 0xE8D00000 // Load multiple registers: load registers from memory
	STM = 0xE8C00000 // Store multiple registers: store registers to memory

	// Branch Instructions
	B  = 0xEA000000 // Branch to a label: jump to address
	BL = 0xEB000000 // Branch with link (call a function): jump and save return address
	BX = 0xE12FFF10 // Branch to an address in a register: jump to address in a register

	// Control Flow Instructions
	CLR = 0xE320F000 // Clear register: set register to zero
	NOP = 0xE320F000 // No operation: do nothing
)

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
