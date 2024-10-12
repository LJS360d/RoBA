package cpu

// Base structure for all ARM instructions
type ARMInstruction struct {
	Cond ArmCondition // Condition field (4 bits) bits 31-28
}

// Data Processing Instruction Structure
type ARMDataProcessingInstruction struct {
	// bits 31-28
	ARMInstruction
	// bits 27-26, 2 bits that are always 0b00

	// bit 25, Immediate 2nd Operand Flag (0=Register, 1=Immediate)
	I bool
	// bits 24-21, Operation Code (4 bits)
	Opcode uint32
	// bit 20, Set Condition Codes (1 bit)
	S bool
	// bits 19-16, First operand register (4 bits)
	Rn uint32
	// bits 15-12, Destination register (4 bits)
	Rd uint32
	// bits 6-5, Shift type (2 bits), (0=LSL, 1=LSR, 2=ASR, 3=ROR)
	ShiftType uint32
	// bit 4, Shift by Register Flag (0=Immediate, 1=Register)
	R bool
	// when I == 0 & R == 0 -> Register as 2nd Operand takes bits (11-7)
	//
	// when I == 1 -> Immediate as 2nd Operand takes bits (11-8)
	Is uint32
	// when I == 0 & R == 1 -> Shift by Register
	//
	// Rs instead of shift Is,  bit 7 is always 0
	//
	// bits 11-8, Source register (4 bits) only set if R = 1
	Rs uint32
	/// bits 11-8, ROR-Shift applied to nn (0-30, in steps of 2)
	// when I == 1 -> Immediate as 2nd Operand takes bits (7-0), Immediate value (Operand Unsigned 8bit Immediate)
	Nn uint32
	// bits 3-0, Second operand register, unset when I == 1, bits taken by Nn
	Rm uint32
	// when I == 1 -> Is + Nn takes all of --> bit 7 + ShiftType + R + Rm (so 11-0)
}

// Load/Store Instruction Structure
type ARMLoadStoreInstruction struct {
	ARMInstruction
	P      uint32 // P flag: 0 for load/store, 1 for load/store with pre/post increment
	U      uint32 // U flag: 0 for negative offset, 1 for positive offset
	B      uint32 // B flag: Byte or Word transfer
	W      uint32 // W flag: Writeback to the base register
	L      uint32 // L flag: 0 for store, 1 for load
	Rn     uint32 // Base register (4 bits)
	Rd     uint32 // Destination register (4 bits)
	Offset uint32 // Offset (12 bits)
}

// Branch Instruction Structure
type ARMBranchInstruction struct {
	ARMInstruction
	Link       uint32 // Link flag: 0 for B, 1 for BL
	TargetAddr uint32 // Target address (26 bits)
}

// Control Instruction Structure
type ARMControlInstruction struct {
	ARMInstruction
	Opcode uint32 // Opcode for control instructions
}

// ParseInstruction parses a 32-bit instruction and returns the appropriate struct.
func ParseInstruction_Arm(instruction uint32) interface{} {
	// Extract the condition (bits 28-31)
	cond := ArmCondition((instruction >> 28) & 0x0F)

	// Check the opcode type (bits 26-27)
	switch (instruction >> 26) & 0x03 { // 2 bits
	case 0: // Data Processing
		// Extract fields specific to the Data Processing Instruction
		I := (instruction >> 25) & 0x01        // Immediate 2nd Operand Flag (1 bit)
		S := (instruction >> 20) & 0x01        // Set Condition Codes (1 bit)
		Rn := (instruction >> 16) & 0x0F       // First operand register (bits 19-16)
		Rd := (instruction >> 12) & 0x0F       // Destination register (bits 15-12)
		ShiftType := (instruction >> 5) & 0x03 // Shift type (bits 6-5)
		R := (instruction >> 4) & 0x01         // Shift by Register Flag (1 bit)
		Rm := instruction & 0x0F               // Second operand register (bits 3-0)

		var Is uint32
		var Rs uint32
		var Nn uint32

		if I == 0 && R == 0 {
			// When I == 0 & R == 0 -> Register as 2nd Operand takes bits (11-7)
			Is = (instruction >> 7) & 0x1F // 5 bits from bits 11-7
		} else if I == 1 {
			// When I == 1 -> Immediate as 2nd Operand takes bits (11-8)
			Is = (instruction >> 8) & 0x0F // Immediate value (bits 11-8)
			Nn = instruction & 0xFF        // Operand Unsigned 8-bit Immediate (bits 7-0)
		} else if I == 0 && R == 1 {
			// When I == 0 & R == 1 -> Shift by Register
			Rs = (instruction >> 8) & 0x0F // Source register (bits 11-8)
		}

		return ARMDataProcessingInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			I:              I != 0, // Convert to bool
			Opcode:         (instruction >> 21) & 0x0F,
			S:              S != 0, // Convert to bool
			Rn:             Rn,
			Rd:             Rd,
			ShiftType:      ShiftType,
			R:              R != 0, // Convert to bool
			Is:             Is,
			Rs:             Rs,
			Nn:             Nn,
			Rm:             Rm,
		}
	case 1: // Branch (B, BL)
		return ARMBranchInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			Link:           (instruction >> 24) & 0x01,      // Link bit
			TargetAddr:     (instruction & 0x00FFFFFF) << 2, // Addressing
		}
	case 2: // Load/Store
		return ARMLoadStoreInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			P:              (instruction >> 24) & 0x01,
			U:              (instruction >> 23) & 0x01,
			B:              (instruction >> 22) & 0x01,
			W:              (instruction >> 21) & 0x01,
			L:              (instruction >> 20) & 0x01,
			Rn:             (instruction >> 16) & 0x0F,
			Rd:             (instruction >> 12) & 0x0F,
			Offset:         instruction & 0x0FFF, // 12-bit immediate offset
		}
	default:
		return ARMControlInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			Opcode:         instruction & 0x0FFFFFFF, // General control opcode
		}
	}
}

type ArmCondition uint32

const (
	Always ArmCondition = iota
	Equal
	NotEqual
	CarrySet
	CarryClear
	Negative
	PositiveOrZero
	Overflow
	NoOverflow
	UnsignedHigherOrSame
	UnsignedLower
	GreaterThan
	LessThanOrEqualTo
	SignedHigher
	SignedLowerOrSame
	Always2
)
