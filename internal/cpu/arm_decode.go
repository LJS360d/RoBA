package cpu

import (
	"fmt"
)

// ARMInstructionType defines the broad category of an ARM instruction.
type ARMInstructionType string

const (
	ARMITDataProcessing    ARMInstructionType = "Data Processing"
	ARMITLoadStore         ARMInstructionType = "Load/Store Single Data Transfer"
	ARMITBranch            ARMInstructionType = "Branch"
	ARMITSWI               ARMInstructionType = "Software Interrupt"
	ARMITBlockDataTransfer ARMInstructionType = "Block Data Transfer"
	ARMITMultiply          ARMInstructionType = "Multiply"
	ARMITTransferMRS       ARMInstructionType = "PSR Transfer (MRS)"
	ARMITTransferMSR       ARMInstructionType = "PSR Transfer (MSR)"
	ARMITUndefined         ARMInstructionType = "Undefined" // For instructions not yet implemented or unknown
)

// ARMCondition defines the condition codes for conditional execution.
type ARMCondition uint8

const (
	EQ ARMCondition = 0x0 // Equal (Z=1)
	NE ARMCondition = 0x1 // Not Equal (Z=0)
	CS ARMCondition = 0x2 // Carry Set (C=1)
	CC ARMCondition = 0x3 // Carry Clear (C=0)
	MI ARMCondition = 0x4 // Minus, Negative (N=1)
	PL ARMCondition = 0x5 // Plus, Positive or Zero (N=0)
	VS ARMCondition = 0x6 // Overflow Set (V=1)
	VC ARMCondition = 0x7 // Overflow Clear (V=0)
	HI ARMCondition = 0x8 // Unsigned Higher (C=1 and Z=0)
	LS ARMCondition = 0x9 // Unsigned Lower or Same (C=0 or Z=1)
	GE ARMCondition = 0xA // Signed Greater Than or Equal (N=V)
	LT ARMCondition = 0xB // Signed Less Than (N!=V)
	GT ARMCondition = 0xC // Signed Greater Than (Z=0 and N=V)
	LE ARMCondition = 0xD // Signed Less Than or Equal (Z=1 or N!=V)
	AL ARMCondition = 0xE // Always (No condition)
	NV ARMCondition = 0xF // Never (Undefined, effectively NOP)
)

// ARMDataProcessingOperation defines the specific operation for Data Processing instructions.
type ARMDataProcessingOperation uint8

const (
	AND ARMDataProcessingOperation = 0x0 // AND logical AND
	EOR ARMDataProcessingOperation = 0x1 // EOR logical exclusive OR
	SUB ARMDataProcessingOperation = 0x2 // SUB subtract
	RSB ARMDataProcessingOperation = 0x3 // RSB reverse subtract
	ADD ARMDataProcessingOperation = 0x4 // ADD add
	ADC ARMDataProcessingOperation = 0x5 // ADC add with carry
	SBC ARMDataProcessingOperation = 0x6 // SBC subtract with carry
	RSC ARMDataProcessingOperation = 0x7 // RSC reverse subtract with carry
	TST ARMDataProcessingOperation = 0x8 // TST test bits
	TEQ ARMDataProcessingOperation = 0x9 // TEQ test equality
	CMP ARMDataProcessingOperation = 0xA // CMP compare
	CMN ARMDataProcessingOperation = 0xB // CMN compare negative
	ORR ARMDataProcessingOperation = 0xC // ORR logical OR
	MOV ARMDataProcessingOperation = 0xD // MOV move
	BIC ARMDataProcessingOperation = 0xE // BIC bit clear
	MVN ARMDataProcessingOperation = 0xF // MVN move not
)

// ARMShiftType defines the type of shift operation.
type ARMShiftType uint8

const (
	LSL ARMShiftType = 0x0 // Logical Shift Left
	LSR ARMShiftType = 0x1 // Logical Shift Right
	ASR ARMShiftType = 0x2 // Arithmetic Shift Right
	ROR ARMShiftType = 0x3 // Rotate Right
)

// ARMInstruction represents a decoded ARM instruction.
type ARMInstruction struct {
	Type ARMInstructionType // e.g., "Data Processing", "Load/Store", "Branch"

	// Common fields across instruction types
	Cond ARMCondition // Condition field (bits 31-28)
	S    bool         // Set Condition Codes flag (bit 20) for Data Processing, Multiply

	// Data Processing specific fields
	OpcodeDP  ARMDataProcessingOperation // Opcode for Data Processing (bits 24-21)
	Rn        uint8                      // First operand register (bits 19-16)
	Rd        uint8                      // Destination register (bits 15-12)
	Rm        uint8                      // Second operand register (bits 3-0) for register-based operand2
	Immediate uint32                     // Immediate value (for immediate-based operand2)
	I         bool                       // Immediate operand flag (bit 25)
	ShiftType ARMShiftType               // Shift type (bits 6-5)
	ShiftImm  uint8                      // Shift immediate (bits 11-7) for immediate shift
	Rs        uint8                      // Shift register (bits 11-8) for register shift
	RotateImm uint8                      // Rotate amount for immediate operand (bits 11-8 of instruction)

	// Load/Store Single Data Transfer specific fields
	L      bool   // Load/Store flag (bit 20: true for Load, false for Store)
	P      bool   // Pre/Post-indexed addressing (bit 24)
	U      bool   // Up/Down (add/subtract offset) (bit 23)
	B      bool   // Byte/Word (bit 22: true for Byte, false for Word)
	W      bool   // Write-back flag (bit 21)
	Offset uint32 // Offset for load/store (bits 11-0 or 7-0 and 11-8 for immediate)

	// Branch specific fields
	OffsetBranch int32 // Signed 24-bit offset for branches (bits 23-0)
	Link         bool  // Link flag (bit 24: true for BL, false for B)
	Exchange     bool  // Exchange flag (bit 4: true for BX/BLX)

	// SWI specific fields
	SWIComment uint32 // 24-bit comment field for SWI (bits 23-0)

	// Block Data Transfer specific fields
	RegisterList uint16 // 16-bit register list (bits 15-0)

	// Multiply specific fields
	A    bool  // Accumulate bit (bit 21: 1=MLA, 0=MUL)
	RdHi uint8 // High destination register (bits 19-16 for long multiply)
	RdLo uint8 // Low destination register (bits 15-12 for long multiply)
}

// DecodeInstruction_Arm decodes a 32-bit ARM instruction into an ARMInstruction struct.
// It parses the instruction based on the ARM instruction format and sets the relevant fields.
// The function provides detailed comments for each instruction type and bit field.
func DecodeInstruction_Arm(instruction uint32) (ARMInstruction, error) {
	decoded := ARMInstruction{
		Cond: ARMCondition((instruction >> 28) & 0xF), // Condition field (bits 31-28)
	}

	// Extract primary opcode bits to determine instruction type
	// Bits 27-20 are crucial for identifying the instruction format.
	// Reference: GBATEK "ARM Binary Opcode Format"
	opcode27_24 := (instruction >> 24) & 0xF

	switch {
	// Data Processing and PSR Transfer (MRS/MSR)
	// Format: Cond | 00 | I | Opcode | S | Rn | Rd | Operand2
	case (opcode27_24&0xC == 0x0): // Bits 27-26 are '00'
		decoded.Type = ARMITDataProcessing
		decoded.I = ((instruction >> 25) & 0x1) == 1                             // Immediate operand flag (bit 25)
		decoded.OpcodeDP = ARMDataProcessingOperation((instruction >> 21) & 0xF) // Opcode (bits 24-21)
		decoded.S = ((instruction >> 20) & 0x1) == 1                             // Set Condition Codes flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF)                            // First operand register (bits 19-16)
		decoded.Rd = uint8((instruction >> 12) & 0xF)                            // Destination register (bits 15-12)

		if decoded.I { // Immediate as 2nd Operand
			decoded.RotateImm = uint8((instruction >> 8) & 0xF)                                              // Rotate amount (bits 11-8)
			imm8 := instruction & 0xFF                                                                       // 8-bit immediate value (bits 7-0)
			decoded.Immediate = (imm8 >> (decoded.RotateImm * 2)) | (imm8 << (32 - (decoded.RotateImm * 2))) // ROR operation
		} else { // Register as 2nd Operand
			decoded.Rm = uint8(instruction & 0xF)                      // Second operand register (bits 3-0)
			decoded.ShiftType = ARMShiftType((instruction >> 5) & 0x3) // Shift type (bits 6-5)
			if ((instruction >> 4) & 0x1) == 0 {                       // Immediate shift
				decoded.ShiftImm = uint8((instruction >> 7) & 0x1F) // Shift immediate (bits 11-7)
			} else { // Register shift
				decoded.Rs = uint8((instruction >> 8) & 0xF) // Shift register (bits 11-8)
				// Bit 7 must be 0 for register shift, bit 4 must be 1.
				if ((instruction >> 7) & 0x1) != 0 {
					return ARMInstruction{}, fmt.Errorf("invalid instruction: bit 7 must be 0 for register shift")
				}
			}
		}

		// Handle MRS (Move from PSR to Register) and MSR (Move to PSR from Register/Immediate)
		// These share the 00b at bits 27-26, but have specific patterns in bits 24-20 and 19-16.
		// MRS: Cond | 00 | 0 | 10 | 0 | Psr | 1111 | Rd | 0000 0000 0000
		// MSR (Reg): Cond | 00 | 0 | 10 | 0 | Field | 0000 | Rm | 0000 0000 0000
		// MSR (Imm): Cond | 00 | 1 | 10 | 0 | Field | 0000 | Immediate
		if ((instruction >> 23) & 0x3) == 0x2 { // Bits 24-23 are '10'
			if ((instruction >> 21) & 0x1) == 0 { // Bit 21 is '0' for MRS
				if ((instruction>>16)&0xF) == 0xF && ((instruction>>4)&0xFF) == 0 { // Check for MRS specific bits
					decoded.Type = ARMITTransferMRS
					// Rd is already parsed
					// Psr (CPSR/SPSR) is bit 22
					// No other fields are relevant here
				} else {
					return ARMInstruction{}, fmt.Errorf("invalid instruction: MRS format mismatch")
				}
			} else { // Bit 21 is '1' for MSR
				decoded.Type = ARMITTransferMSR
				// Field mask (bits 19-16) and operand (Rm or Immediate)
				// No other fields are relevant here for a generic decoder
			}
		}

	// Multiply (MUL, MLA, UMULL, UMLAL, SMULL, SMLAL)
	// Format: Cond | 0000 | A | S | Rd | Rn | Rs | 1001 | Rm
	case (opcode27_24&0xE == 0x0) && ((instruction>>4)&0xF == 0x9): // Bits 27-24 are '0000', bits 7-4 are '1001'
		decoded.Type = ARMITMultiply
		decoded.A = ((instruction >> 21) & 0x1) == 1  // Accumulate bit (bit 21)
		decoded.S = ((instruction >> 20) & 0x1) == 1  // Set Condition Codes flag (bit 20)
		decoded.Rd = uint8((instruction >> 16) & 0xF) // Destination register (bits 19-16)
		decoded.Rn = uint8((instruction >> 12) & 0xF) // Accumulate register (bits 15-12), or RdLo for long multiply
		decoded.Rs = uint8((instruction >> 8) & 0xF)  // Operand Register (bits 11-8)
		decoded.Rm = uint8(instruction & 0xF)         // Second operand register (bits 3-0)

		// Differentiate between MUL/MLA and long multiplies based on bits 24-21
		mulOpcode := (instruction >> 21) & 0xF
		if mulOpcode == 0x0 || mulOpcode == 0x1 { // MUL or MLA
			// Rd, Rn, Rs, Rm are already parsed
		} else if mulOpcode >= 0x4 && mulOpcode <= 0x7 { // UMULL, UMLAL, SMULL, SMLAL
			decoded.RdHi = decoded.Rd // RdHi is bits 19-16
			decoded.RdLo = decoded.Rn // RdLo is bits 15-12
			decoded.Rn = 0            // Rn is not used in these instructions
		} else {
			return ARMInstruction{}, fmt.Errorf("invalid multiply instruction opcode: %x", mulOpcode)
		}

	// Load/Store Single Data Transfer
	// Format: Cond | 01 | P | U | B | W | L | Rn | Rd | Offset/Shifted_Rm
	case (opcode27_24&0xC == 0x4): // Bits 27-26 are '01'
		decoded.Type = ARMITLoadStore
		decoded.P = ((instruction >> 24) & 0x1) == 1  // Pre/Post-indexed addressing (bit 24)
		decoded.U = ((instruction >> 23) & 0x1) == 1  // Up/Down (add/subtract offset) (bit 23)
		decoded.B = ((instruction >> 22) & 0x1) == 1  // Byte/Word (bit 22)
		decoded.W = ((instruction >> 21) & 0x1) == 1  // Write-back flag (bit 21)
		decoded.L = ((instruction >> 20) & 0x1) == 1  // Load/Store flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF) // Base register (bits 19-16)
		decoded.Rd = uint8((instruction >> 12) & 0xF) // Source/Destination register (bits 15-12)

		if ((instruction >> 25) & 0x1) == 0 { // Immediate offset
			decoded.Offset = instruction & 0xFFF // 12-bit immediate offset (bits 11-0)
		} else { // Register offset with optional shift
			decoded.Rm = uint8(instruction & 0xF)                      // Register offset (bits 3-0)
			decoded.ShiftType = ARMShiftType((instruction >> 5) & 0x3) // Shift type (bits 6-5)
			decoded.ShiftImm = uint8((instruction >> 7) & 0x1F)        // Shift immediate (bits 11-7)
			// Bit 4 must be 0 for this format
			if ((instruction >> 4) & 0x1) != 0 {
				return ARMInstruction{}, fmt.Errorf("invalid instruction: bit 4 must be 0 for register offset with shift")
			}
		}

	// Block Data Transfer (LDM, STM)
	// Format: Cond | 100 | P | U | S | W | L | Rn | Register_List
	case (opcode27_24&0xE == 0x8): // Bits 27-25 are '100'
		decoded.Type = ARMITBlockDataTransfer
		decoded.P = ((instruction >> 24) & 0x1) == 1        // Pre/Post-indexed addressing (bit 24)
		decoded.U = ((instruction >> 23) & 0x1) == 1        // Up/Down (add/subtract offset) (bit 23)
		decoded.S = ((instruction >> 22) & 0x1) == 1        // S-bit (PSR and R15 update) (bit 22)
		decoded.W = ((instruction >> 21) & 0x1) == 1        // Write-back flag (bit 21)
		decoded.L = ((instruction >> 20) & 0x1) == 1        // Load/Store flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF)       // Base register (bits 19-16)
		decoded.RegisterList = uint16(instruction & 0xFFFF) // 16-bit register list (bits 15-0)

	// Branch and Branch with Link (B, BL)
	// Format: Cond | 101 | L | Offset
	case (opcode27_24&0xE == 0xA): // Bits 27-25 are '101'
		decoded.Type = ARMITBranch
		decoded.Link = ((instruction >> 24) & 0x1) == 1 // Link flag (bit 24)
		// 24-bit signed offset (bits 23-0)
		offset := int32(instruction & 0xFFFFFF)
		// Sign-extend the 24-bit offset to 32-bit
		if offset&(1<<23) != 0 {
			offset |= ^0xFFFFFF
		}
		decoded.OffsetBranch = offset

	// Software Interrupt (SWI)
	// Format: Cond | 1111 | SWI_Comment
	case (opcode27_24&0xF == 0xF): // Bits 27-24 are '1111'
		decoded.Type = ARMITSWI
		decoded.SWIComment = instruction & 0xFFFFFF // 24-bit comment field (bits 23-0)

	// Branch and Exchange (BX, BLX)
	// Format: Cond | 0001 0010 | 1111 1111 | 1111 | 0001 | Rm (for BX)
	// Format: Cond | 0001 0010 | 1111 1111 | 1111 | 0011 | Rm (for BLX register)
	// Note: BLX immediate has a different format (Cond | 1111 0 | Offset | 1 | Offset_low)
	case (instruction&0x0FFFFF00 == 0x012FFF00) && ((instruction>>4)&0x1) == 1: // Check for BX/BLX register format
		decoded.Type = ARMITBranch
		decoded.Rm = uint8(instruction & 0xF) // Register containing target address (bits 3-0)
		decoded.Exchange = true               // Indicates BX or BLX
		// Bit 5 indicates BLX (1) or BX (0)
		if ((instruction >> 5) & 0x1) == 1 {
			decoded.Link = true // It's a BLX instruction
		}

	default:
		// If none of the above common patterns match, it's an undefined or unimplemented instruction.
		// You can expand this section to cover more instruction types as needed.
		decoded.Type = ARMITUndefined
		return decoded, fmt.Errorf("unsupported or undefined ARM instruction: 0x%08X", instruction)
	}

	return decoded, nil
}

// Helper function to represent the decoded instruction in a readable format.
func (inst ARMInstruction) String() string {
	s := fmt.Sprintf("Type: %s, Cond: %X", inst.Type, inst.Cond)

	switch inst.Type {
	case ARMITDataProcessing:
		s += fmt.Sprintf(", Opcode: %X, S: %t, Rn: R%d, Rd: R%d", inst.OpcodeDP, inst.S, inst.Rn, inst.Rd)
		if inst.I {
			s += fmt.Sprintf(", Immediate: 0x%X", inst.Immediate)
		} else {
			s += fmt.Sprintf(", Rm: R%d, ShiftType: %X, ShiftImm: %d, Rs: R%d", inst.Rm, inst.ShiftType, inst.ShiftImm, inst.Rs)
		}
	case ARMITLoadStore:
		s += fmt.Sprintf(", L: %t, P: %t, U: %t, B: %t, W: %t, Rn: R%d, Rd: R%d", inst.L, inst.P, inst.U, inst.B, inst.W, inst.Rn, inst.Rd)
		if ((inst.Offset >> 12) & 0x1) == 0 { // Immediate offset (12-bit)
			s += fmt.Sprintf(", Offset: 0x%X", inst.Offset)
		} else { // Register offset with shift
			s += fmt.Sprintf(", Rm: R%d, ShiftType: %X, ShiftImm: %d", inst.Rm, inst.ShiftType, inst.ShiftImm)
		}
	case ARMITBranch:
		s += fmt.Sprintf(", Link: %t, Offset: 0x%X", inst.Link, inst.OffsetBranch)
		if inst.Exchange {
			s += fmt.Sprintf(", Exchange: %t, Rm: R%d", inst.Exchange, inst.Rm)
		}
	case ARMITSWI:
		s += fmt.Sprintf(", SWIComment: 0x%X", inst.SWIComment)
	case ARMITBlockDataTransfer:
		s += fmt.Sprintf(", P: %t, U: %t, S: %t, W: %t, L: %t, Rn: R%d, RegisterList: 0x%X", inst.P, inst.U, inst.S, inst.W, inst.L, inst.Rn, inst.RegisterList)
	case ARMITMultiply:
		s += fmt.Sprintf(", A: %t, S: %t, Rd: R%d, Rn: R%d, Rs: R%d, Rm: R%d", inst.A, inst.S, inst.Rd, inst.Rn, inst.Rs, inst.Rm)
		if inst.RdHi != 0 || inst.RdLo != 0 { // For long multiply instructions
			s += fmt.Sprintf(", RdHi: R%d, RdLo: R%d", inst.RdHi, inst.RdLo)
		}
	case ARMITUndefined:
		s += ", (Undefined Instruction)"
	}
	return s
}
