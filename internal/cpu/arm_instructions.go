package cpu

// Base structure for all ARM instructions
type ARMInstruction struct {
	Cond ARMCondition // Condition field (4 bits) bits 31-28
}

// Data Processing Instruction Structure
type ARMDataProcessingInstruction struct {
	// bits 31-28
	ARMInstruction
	// bits 27-26, 2 bits that are always 0b00

	// bit 25, Immediate 2nd Operand Flag (0=Register, 1=Immediate)
	I bool
	// bits 24-21, Operation Code (4 bits)
	Opcode ARMDataProcessingOperation
	// bit 20, Set Condition Codes (1 bit)
	S bool
	// bits 19-16, First operand register (4 bits)
	Rn uint8
	// bits 15-12, Destination register (4 bits)
	Rd uint8
	// bits 6-5, Shift type (2 bits), (0=LSL, 1=LSR, 2=ASR, 3=ROR)
	ShiftType ARMShiftType
	// bit 4, Shift by Register Flag (0=Immediate, 1=Register)
	R bool
	// when I == 0 & R == 0 -> Register as 2nd Operand takes bits (11-7)
	//
	// when I == 1 -> Immediate as 2nd Operand takes bits (11-8)
	Is uint8
	// when I == 0 & R == 1 -> Shift by Register
	//
	// Rs instead of shift Is,  bit 7 is always 0
	//
	// bits 11-8, Source register (4 bits) only set if R = 1
	Rs uint8
	/// bits 11-8, ROR-Shift applied to nn (0-30, in steps of 2)
	// when I == 1 -> Immediate as 2nd Operand takes bits (7-0), Immediate value (Operand Unsigned 8bit Immediate)
	Nn uint8
	// bits 3-0, Second operand register, unset when I == 1, bits taken by Nn
	Rm uint8
	// when I == 1 -> Is + Nn takes all of --> bit 7 + ShiftType + R + Rm (so 11-0)
}

// Load/Store Instruction Structure
type ARMLoadStoreInstruction struct {
	ARMInstruction
	P      bool   // P flag: 0 for load/store, 1 for load/store with pre/post increment
	U      bool   // U flag: 0 for negative offset, 1 for positive offset
	B      bool   // B flag: Byte or Word transfer
	W      bool   // W flag: Writeback to the base register
	L      bool   // L flag: 0 for store, 1 for load
	Rn     uint8  // Base register (4 bits)
	Rd     uint8  // Destination register (4 bits)
	Offset uint32 // Offset (12 bits)
}

// Branch Instruction Structure
type ARMBranchInstruction struct {
	ARMInstruction
	Link       bool   // Link flag: 0 for B, 1 for BL
	TargetAddr uint32 // Target address (26 bits)
}

// Control Instruction Structure
type ARMControlInstruction struct {
	ARMInstruction
	Opcode uint32 // Opcode for control instructions
}

// ARMMultiplyInstruction represents a Multiply or Multiply-Accumulate instruction.
type ARMMultiplyInstruction struct {
	ARMInstruction
	A  bool  // Accumulate bit (1=MLA, 0=MUL)
	S  bool  // Set Condition Codes
	Rd uint8 // Destination register
	Rn uint8 // Accumulate register (for MLA) or always 0 (for MUL)
	Rs uint8 // Source register 1
	Rm uint8 // Source register 2
}

// ARMSWIInstruction represents a Software Interrupt instruction.
type ARMSWIInstruction struct {
	ARMInstruction
	Immediate uint32 // 24-bit immediate value
}

// ARMBlockDataTransferInstruction represents LDM (Load Multiple) or STM (Store Multiple) instructions.
type ARMBlockDataTransferInstruction struct {
	ARMInstruction
	P            bool   // Pre/Post-indexing (1=Pre, 0=Post)
	U            bool   // Up/Down (1=Add offset, 0=Subtract offset)
	S            bool   // S-bit (User/System mode or PC write-back)
	W            bool   // Write-back (1=Write base register back, 0=No write-back)
	L            bool   // Load/Store (1=Load, 0=Store)
	Rn           uint8  // Base register
	RegisterList uint16 // 16-bit register list mask
}

type ARMCondition uint32

const (
	EQ ARMCondition = 0x0 // Equal                (Z=1)
	NE ARMCondition = 0x1 // Not Equal            (Z=0)
	CS ARMCondition = 0x2 // Carry Set            (C=1)
	CC ARMCondition = 0x3 // Carry Clear          (C=0)
	MI ARMCondition = 0x4 // Minus, Negative      (N=1)
	PL ARMCondition = 0x5 // Plus, Positive or Zero (N=0)
	VS ARMCondition = 0x6 // Overflow Set         (V=1)
	VC ARMCondition = 0x7 // Overflow Clear       (V=0)
	HI ARMCondition = 0x8 // Unsigned Higher      (C=1 and Z=0)
	LS ARMCondition = 0x9 // Unsigned Lower or Same (C=0 or Z=1)
	GE ARMCondition = 0xA // Signed Greater Than or Equal (N=V)
	LT ARMCondition = 0xB // Signed Less Than     (N!=V)
	GT ARMCondition = 0xC // Signed Greater Than  (Z=0 and N=V)
	LE ARMCondition = 0xD // Signed Less Than or Equal (Z=1 or N!=V)
	AL ARMCondition = 0xE // Always               (No condition)
	NV ARMCondition = 0xF // Never                (Undefined, effectively NOP)
)

type ARMDataProcessingOperation uint32

// ARM Data Processing OpCodes
// Enum for ARM Data Processing Opcodes
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

// ARM Shift Type
type ARMShiftType uint32

// Enum for ARM Shift Types
const (
	LSL ARMShiftType = 0x0 // Logical Shift Left
	LSR ARMShiftType = 0x1 // Logical Shift Right
	ASR ARMShiftType = 0x2 // Arithmetic Shift Right
	ROR ARMShiftType = 0x3 // Rotate Right
)
