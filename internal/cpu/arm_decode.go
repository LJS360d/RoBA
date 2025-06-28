package cpu

import "fmt"

// ParseInstruction parses a 32-bit instruction and returns the appropriate struct.
func DecodeInstruction_Arm(instruction uint32) interface{} {
	// Extract the condition (bits 28-31)
	cond := ARMCondition((instruction >> 28) & 0x0F)

	// Check the opcode type (bits 26-27)
	switch (instruction >> 26) & 0x03 { // 2 bits
	case 0: // 00: Data Processing or Multiply
		// ARM Multiply (Bit 24=0, Bit 23=0, Bit 7=0, Bit 4=1, Bits 20-23 and 5-6 determine precise type)
		// Pattern for Multiply: Cond_0000_A_S_Rd_Rn_Rs_1001_Rm
		if ((instruction>>24)&0xF) == 0x0 && // Bits 24-27 are 0000 (part of multiply pattern)
			((instruction>>4)&0xF) == 0x9 { // Bit 7 and 4 are specific (0 and 1) for multiply
			return ARMMultiplyInstruction{
				ARMInstruction: ARMInstruction{Cond: cond},
				A:              ((instruction >> 21) & 0x01) != 0, // A bit (Accumulate)
				S:              ((instruction >> 20) & 0x01) != 0, // S bit (Set Condition Codes)
				Rd:             uint8((instruction >> 16) & 0x0F), // Rd
				Rn:             uint8((instruction >> 12) & 0x0F), // Rn
				Rs:             uint8((instruction >> 8) & 0x0F),  // Rs
				Rm:             uint8(instruction & 0x0F),         // Rm
			}
		}

		// Otherwise, it's a Data Processing instruction
		I := ((instruction >> 25) & 0x01) != 0        // Immediate 2nd Operand Flag (1 bit)
		S := ((instruction >> 20) & 0x01) != 0        // Set Condition Codes (1 bit)
		Rn := uint8((instruction >> 16) & 0x0F)       // First operand register (bits 19-16)
		Rd := uint8((instruction >> 12) & 0x0F)       // Destination register (bits 15-12)
		ShiftType := uint8((instruction >> 5) & 0x03) // Shift type (bits 6-5)
		R := ((instruction >> 4) & 0x01) != 0         // Shift by Register Flag (1 bit)
		Rm := uint8(instruction & 0x0F)               // Second operand register (bits 3-0)

		var Is uint8
		var Rs uint8
		var Nn uint8

		if !I && !R {
			// When I == 0 & R == 0 -> Register as 2nd Operand takes bits (11-7)
			Is = uint8((instruction >> 7) & 0x1F) // 5 bits from bits 11-7
		} else if I {
			// When I == 1 -> Immediate as 2nd Operand takes bits (11-8)
			Is = uint8((instruction >> 8) & 0x0F) // Immediate value (bits 11-8)
			Nn = uint8(instruction & 0xFF)        // Operand Unsigned 8-bit Immediate (bits 7-0)
		} else if !I && R {
			// When I == 0 & R == 1 -> Shift by Register
			Rs = uint8((instruction >> 8) & 0x0F) // Source register (bits 11-8)
		}

		return ARMDataProcessingInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			I:              I,
			Opcode:         ARMDataProcessingOperation((instruction >> 21) & 0x0F),
			S:              S,
			Rn:             Rn,
			Rd:             Rd,
			ShiftType:      ARMShiftType(ShiftType),
			R:              R,
			Is:             Is,
			Rs:             Rs,
			Nn:             Nn,
			Rm:             Rm,
		}
	case 1: // 01: Load/Store (Single Data Transfer)
		return ARMLoadStoreInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			P:              ((instruction >> 24) & 0x01) != 0,
			U:              ((instruction >> 23) & 0x01) != 0,
			B:              ((instruction >> 22) & 0x01) != 0,
			W:              ((instruction >> 21) & 0x01) != 0,
			L:              ((instruction >> 20) & 0x01) != 0,
			Rn:             uint8((instruction >> 16) & 0x0F),
			Rd:             uint8((instruction >> 12) & 0x0F),
			Offset:         uint32(instruction & 0x0FFF), // 12-bit immediate offset or register shifted immediate
		}
	case 2: // 10: Branch and Branch with Link (B, BL) OR LDM/STM (Block Data Transfer) OR SWP (Single Data Swap)
		// These patterns often share bit 27 = 1 and bit 26 = 0, so more specific checks are needed.
		// Block Data Transfer (LDM/STM): Cond_100_P_U_S_W_L_Rn_RegisterList
		// Bit 25 is '1' for LDM/STM.
		if ((instruction >> 25) & 0x01) == 1 {
			return ARMBlockDataTransferInstruction{
				ARMInstruction: ARMInstruction{Cond: cond},
				P:              ((instruction >> 24) & 0x01) != 0,
				U:              ((instruction >> 23) & 0x01) != 0,
				S:              ((instruction >> 22) & 0x01) != 0, // S-bit
				W:              ((instruction >> 21) & 0x01) != 0,
				L:              ((instruction >> 20) & 0x01) != 0,
				Rn:             uint8((instruction >> 16) & 0x0F),
				RegisterList:   uint16(instruction & 0xFFFF), // Register list mask
			}
		}

		// SWP (Single Data Swap): Cond_0001_0_0_0_0_B_0_0_0_Rn_Rd_0000_1001_Rm (This is more specific, might be 00010000xxxxxxxx00001001)
		// This also falls under the 'case 00' top level bits. For now, let's assume it's handled by generic Data Processing if not explicitly caught by Multiply.
		// For proper SWP decoding, you'd need a pattern like:
		// if (instruction & 0x0FB00FF0) == 0x01000090 { // Example simplified mask for SWP
		//    return ARMSingleDataSwapInstruction{...}
		// }

		// Defaulting to Branch if not LDM/STM (or SWP if implemented)
		offset := instruction & 0x00FFFFFF // Treat as signed 24-bit
		if offset&0x00800000 != 0 {        // Check if the 24th bit (sign bit) is set
			offset |= 0xFF000000 // Sign extend
		}
		targetOffset := offset << 2
		return ARMBranchInstruction{
			ARMInstruction: ARMInstruction{Cond: cond},
			Link:           ((instruction >> 24) & 0x01) == 1, // Link bit (BL vs B)
			TargetAddr:     targetOffset,
		}

	case 3: // 11: Software Interrupt (SWI) or Coprocessor (CP) instructions
		// SWI instruction: Cond_1111_Immediate
		if ((instruction >> 24) & 0x0F) == 0x0F { // Bits 24-27 are 1111 for SWI
			return ARMSWIInstruction{
				ARMInstruction: ARMInstruction{Cond: cond},
				Immediate:      instruction & 0x00FFFFFF, // 24-bit immediate
			}
		}

		// Coprocessor Instructions (Often fall here if not specifically identified by other means)
		// For GBA, coprocessor instructions usually result in an undefined instruction exception.
		// You might want to return a specific struct for coprocessor if you plan to handle them,
		// or just let it fall through to a generic "unimplemented control" or "panic".

		return ARMControlInstruction{ // Generic fallback for other 11-prefixed instructions not yet handled
			ARMInstruction: ARMInstruction{Cond: cond},
			Opcode:         instruction & 0x0FFFFFFF, // General control opcode
		}
	default:
		// Should never happen with 2 bits, but good for safety
		panic(fmt.Sprintf("DecodeInstruction_Arm: Unknown instruction type (bits 26-27): %d", (instruction>>26)&0x03))
	}
}
