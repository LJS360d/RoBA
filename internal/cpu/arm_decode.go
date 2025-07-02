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
	ARMITBranchExchange    ARMInstructionType = "Branch and Exchange"
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

	// For robust decoding, examine specific bit patterns with masks.
	// The order of checks is crucial for overlapping patterns.
	// More specific patterns should be checked before more general ones.

	// Bits 27-20 are very important for primary instruction classification.
	// Refer to the GBATEK "ARM Binary Opcode Format"
	// Example: (instruction & 0x0F000000) >> 24 extracts bits 27-24 (top 4 bits of opcode)

	switch {
	// --- Type 1: Software Interrupt (SWI) ---
	// Cond | 1111 | SWI_Comment
	// Mask: 0xFF000000 (bits 31-24 are 1111xxxx)
	case (instruction>>24)&0xFF == 0xF: // Check if bits 27-24 are 1111 (0xF)
		decoded.Type = ARMITSWI
		decoded.SWIComment = instruction & 0xFFFFFF // 24-bit comment field (bits 23-0)

	// --- Type 2: Branch and Exchange (BX, BLX - Register variant) ---
	// Cond | 0001 0010 | 1111 1111 | 1111 | 0001 | Rm (BX)
	// Cond | 0001 0010 | 1111 1111 | 1111 | 0011 | Rm (BLX)
	// Unique pattern: bits 27-4 are `00010010111111111111` and bit 4 is `1`
	// Mask: 0x0FFFFFD0. Compare with 0x012FFF10 for BX/BLX.
	case (instruction&0x0FFFFFF0 == 0x012FFF10) || (instruction&0x0FFFFFF0 == 0x012FFF30): // Also check BLX reg form
		decoded.Type = ARMITBranchExchange // New type for clarity
		decoded.Rm = uint8(instruction & 0xF)
		decoded.Exchange = true
		if ((instruction >> 5) & 0x1) == 1 { // Bit 5 is 'L' for BLX
			decoded.Link = true
		}

	// --- Type 3: Multiply and Multiply Long ---
	// Cond | 0000 | A | S | Rd | Rn | Rs | 1001 | Rm
	// Cond | 0000 | 1 | A | S | RdHi | RdLo | Rs | 1001 | Rm (Long Multiply)
	// Unique pattern: bits 27-24 are '0000' AND bits 7-4 are '1001'
	case ((instruction>>24)&0xF == 0x0) && ((instruction>>4)&0xF == 0x9):
		decoded.Type = ARMITMultiply
		decoded.A = ((instruction >> 21) & 0x1) == 1 // Accumulate bit (bit 21)
		decoded.S = ((instruction >> 20) & 0x1) == 1 // Set Condition Codes flag (bit 20)

		// Check for Long Multiply (bits 23-22: 01, type 4-7, implies bits 27-24=0000)
		if ((instruction >> 22) & 0x3) == 0x1 { // If bits 23-22 are '01'
			// This covers UMULL, UMLAL, SMULL, SMLAL (opcodes 0x4-0x7)
			decoded.RdHi = uint8((instruction >> 16) & 0xF) // Bits 19-16
			decoded.RdLo = uint8((instruction >> 12) & 0xF) // Bits 15-12
			decoded.Rs = uint8((instruction >> 8) & 0xF)    // Bits 11-8
			decoded.Rm = uint8(instruction & 0xF)           // Bits 3-0
			decoded.Rn = 0                                  // Rn field is not used as a source register for these
		} else { // Standard Multiply (MUL, MLA)
			decoded.Rd = uint8((instruction >> 16) & 0xF) // Destination register (bits 19-16)
			decoded.Rn = uint8((instruction >> 12) & 0xF) // Accumulate register (bits 15-12 for MLA, not used for MUL)
			decoded.Rs = uint8((instruction >> 8) & 0xF)  // Operand Register (bits 11-8)
			decoded.Rm = uint8(instruction & 0xF)         // Second operand register (bits 3-0)
		}

	// --- Type 4: PSR Transfer (MRS/MSR) ---
	// These share the '00' prefix (bits 27-26) with Data Processing,
	// but have specific patterns in bits 24-21 and 15-0.
	// MRS: Cond | 00101 | S (0) | Rn (1111) | Rd | 0000_0000_0000
	// MSR (Reg): Cond | 00100 | S (0) | Field (19-16) | 0000 | 0000_0000_Rm
	// MSR (Imm): Cond | 00110 | S (0) | Field (19-16) | Imm12 (Rotate | Immediate)
	// A common mask to identify them is (instruction & 0x0FB0F000)
	case (instruction&0x0FB0F000 == 0x01000000) || (instruction&0x0FE00000 == 0x03200000): // More precise checks for MSR Imm and MSR Reg/MRS
		// Check for MRS (Move from PSR to Register)
		// Pattern: Cond | 0010100 | 1111 | Rd | 000000000000
		if (instruction&0x0FF000F0 == 0x01000000) && ((instruction>>21)&0x7) == 0x5 { // Check for 00101_00 in bits 27-20 and 0000 in bits 11-8
			decoded.Type = ARMITTransferMRS
			decoded.Rd = uint8((instruction >> 12) & 0xF)
			// No other relevant fields to parse for MRS
		} else if ((instruction>>21)&0x7 == 0x4) || ((instruction>>21)&0x7 == 0x6) { // MSR (Move to PSR)
			// MSR Register: Cond | 0010000 | Field | 0000 | 0000_Rm
			// MSR Immediate: Cond | 0011000 | Field | Rotate | Imm8
			decoded.Type = ARMITTransferMSR
			decoded.I = ((instruction >> 25) & 0x1) == 1 // Immediate or Register source
			// The field mask bits (19-16) are implied for MSR
			if decoded.I {
				decoded.RotateImm = uint8((instruction >> 8) & 0xF)
				imm8 := instruction & 0xFF
				decoded.Immediate = (imm8 >> (decoded.RotateImm * 2)) | (imm8 << (32 - (decoded.RotateImm * 2)))
			} else {
				decoded.Rm = uint8(instruction & 0xF)
			}
		} else {
			return ARMInstruction{}, fmt.Errorf("unhandled PSR Transfer variant: 0x%08X", instruction)
		}

	// --- Type 5: Data Processing (General) ---
	// Cond | 00 | I | Opcode | S | Rn | Rd | Operand2
	// This should come after more specific instructions that also start with '00' in bits 27-26.
	case (instruction>>26)&0x3 == 0x0: // Bits 27-26 are '00'
		decoded.Type = ARMITDataProcessing
		decoded.I = ((instruction >> 25) & 0x1) == 1                             // Immediate operand flag (bit 25)
		decoded.OpcodeDP = ARMDataProcessingOperation((instruction >> 21) & 0xF) // Opcode (bits 24-21)
		decoded.S = ((instruction >> 20) & 0x1) == 1                             // Set Condition Codes flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF)                            // First operand register (bits 19-16)
		decoded.Rd = uint8((instruction >> 12) & 0xF)                            // Destination register (bits 15-12)

		if decoded.I { // Immediate as 2nd Operand
			decoded.RotateImm = uint8((instruction >> 8) & 0xF) // Rotate amount (bits 11-8)
			imm8 := instruction & 0xFF                          // 8-bit immediate value (bits 7-0)
			// Compute the rotated immediate value
			decoded.Immediate = (imm8 >> (decoded.RotateImm * 2)) | (imm8 << (32 - (decoded.RotateImm * 2)))
		} else { // Register as 2nd Operand
			decoded.Rm = uint8(instruction & 0xF)                      // Second operand register (bits 3-0)
			decoded.ShiftType = ARMShiftType((instruction >> 5) & 0x3) // Shift type (bits 6-5)
			if ((instruction >> 4) & 0x1) == 0 {                       // Immediate shift
				decoded.ShiftImm = uint8((instruction >> 7) & 0x1F) // Shift immediate (bits 11-7)
			} else { // Register shift
				decoded.Rs = uint8((instruction >> 8) & 0xF) // Shift register (bits 11-8)
				if ((instruction >> 7) & 0x1) != 0 {
					return ARMInstruction{}, fmt.Errorf("invalid instruction: bit 7 must be 0 for register shift")
				}
			}
		}

	// --- Type 6: Load/Store Single Data Transfer ---
	// Cond | 01 | P | U | B | W | L | Rn | Rd | Offset/Shifted_Rm
	case (instruction>>26)&0x3 == 0x1: // Bits 27-26 are '01'
		decoded.Type = ARMITLoadStore
		decoded.P = ((instruction >> 24) & 0x1) == 1  // Pre/Post-indexed addressing (bit 24)
		decoded.U = ((instruction >> 23) & 0x1) == 1  // Up/Down (add/subtract offset) (bit 23)
		decoded.B = ((instruction >> 22) & 0x1) == 1  // Byte/Word (bit 22)
		decoded.W = ((instruction >> 21) & 0x1) == 1  // Write-back flag (bit 21)
		decoded.L = ((instruction >> 20) & 0x1) == 1  // Load/Store flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF) // Base register (bits 19-16)
		decoded.Rd = uint8((instruction >> 12) & 0xF) // Source/Destination register (bits 15-12)

		if ((instruction >> 25) & 0x1) == 0 { // Immediate offset (bit 25 is 0)
			decoded.Offset = instruction & 0xFFF // 12-bit immediate offset (bits 11-0)
		} else { // Register offset with optional shift (bit 25 is 1)
			decoded.Rm = uint8(instruction & 0xF)                      // Register offset (bits 3-0)
			decoded.ShiftType = ARMShiftType((instruction >> 5) & 0x3) // Shift type (bits 6-5)
			decoded.ShiftImm = uint8((instruction >> 7) & 0x1F)        // Shift immediate (bits 11-7)
			if ((instruction >> 4) & 0x1) != 0 {                       // Bit 4 must be 0 for this format (register as offset, not register with shift as offset)
				return ARMInstruction{}, fmt.Errorf("invalid instruction: bit 4 must be 0 for register offset with shift in LDR/STR")
			}
		}

	// --- Type 7: Block Data Transfer (LDM, STM) ---
	// Cond | 100 | P | U | S | W | L | Rn | Register_List
	case (instruction>>25)&0x7 == 0x4: // Bits 27-25 are '100'
		decoded.Type = ARMITBlockDataTransfer
		decoded.P = ((instruction >> 24) & 0x1) == 1        // Pre/Post-indexed addressing (bit 24)
		decoded.U = ((instruction >> 23) & 0x1) == 1        // Up/Down (add/subtract offset) (bit 23)
		decoded.S = ((instruction >> 22) & 0x1) == 1        // S-bit (PSR and R15 update) (bit 22)
		decoded.W = ((instruction >> 21) & 0x1) == 1        // Write-back flag (bit 21)
		decoded.L = ((instruction >> 20) & 0x1) == 1        // Load/Store flag (bit 20)
		decoded.Rn = uint8((instruction >> 16) & 0xF)       // Base register (bits 19-16)
		decoded.RegisterList = uint16(instruction & 0xFFFF) // 16-bit register list (bits 15-0)

	// --- Type 8: Branch and Branch with Link (B, BL) ---
	// Cond | 101 | L | Offset
	case (instruction>>25)&0x7 == 0x5: // Bits 27-25 are '101'
		decoded.Type = ARMITBranch
		decoded.Link = ((instruction >> 24) & 0x1) == 1 // Link flag (bit 24)
		// 24-bit signed offset (bits 23-0)
		offset := int32(instruction & 0xFFFFFF)
		// Sign-extend the 24-bit offset to 32-bit
		if offset&(1<<23) != 0 {
			offset |= ^0xFFFFFF
		}
		decoded.OffsetBranch = offset

	// --- Add other instruction types here as needed, following the priority of unique masks. ---
	// The Undefined instruction pattern 011...1... (bits 27-24 are 0x7 or 0xE depending on the exact pattern)
	// Cond | 011 | ... | 1 | ... | (Undefined)
	// Cond | 110 | P | U | N | W | L | Rn | CRd | CP# | OffsetCoProc  (Coprocessor Data Transfer)
	// Cond | 1110 | ... (Coprocessor Data Operation)
	// Cond | 1110 | ... (Coprocessor Register Transfer)

	default:
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
