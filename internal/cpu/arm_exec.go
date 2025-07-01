package cpu

import (
	"GoBA/util/convert"
	"GoBA/util/dbg"
	"fmt"
)

// PSR (Program Status Register) fields as constants for MSR field mask
const (
	PSR_FLAGS     uint32 = 0xFF000000 // Bits 31-24 (f)
	PSR_STATUS    uint32 = 0x00FF0000 // Bits 23-16 (s) - Reserved, don't change
	PSR_EXTENSION uint32 = 0x0000FF00 // Bits 15-8  (x) - Reserved, don't change
	PSR_CONTROL   uint32 = 0x000000FF // Bits 7-0   (c)
)

// execute ARM instruction based on opcode.
func (c *CPU) execute_Arm(instruction uint32) {
	// Extract condition code
	cond := (instruction >> 28) & 0xF
	// Check condition

	if !c.checkCondition_Arm(cond) {
		// fmt.Println("NOP")
		return // Condition not met, treat as NOP
	}
	dbg.Printf("0x%08X | 0b%032b\n", instruction, instruction)
	inst, err := DecodeInstruction_Arm(instruction)
	if err != nil {
		dbg.Printf("ARM Decode Error: %s\n", err)
		return
	}
	dbg.Printf("%s", inst.String())
	switch inst.Type {
	case ARMITDataProcessing:
		switch inst.OpcodeDP {
		case AND:
			c.execArm_And(inst)
		case EOR:
			c.execArm_Eor(inst)
		case SUB:
			c.execArm_Sub(inst)
		case RSB:
			c.execArm_Rsb(inst)
		case ADD:
			c.execArm_Add(inst)
		case ADC:
			c.execArm_Adc(inst)
		case SBC:
			c.execArm_Sbc(inst)
		case RSC:
			c.execArm_Rsc(inst)
		case TST:
			c.execArm_Tst(inst)
		case TEQ:
			c.execArm_Teq(inst)
		case CMP:
			c.execArm_Cmp(inst)
		case CMN:
			c.execArm_Cmn(inst)
		case ORR:
			c.execArm_Orr(inst)
		case MOV:
			c.execArm_Mov(inst)
		case BIC:
			c.execArm_Bic(inst)
		case MVN:
			c.execArm_Mvn(inst)
		}
		return

	case ARMITLoadStore:
		c.execArm_LoadStore(inst, c.Registers.PC-8)
		return

	case ARMITBranch:
		c.execArm_Branch(inst, c.Registers.PC-8)
		return

	case ARMITBlockDataTransfer:
		c.execArm_BlockDataTransfer(inst, c.Registers.PC-8)
		return

	case ARMITSWI:
		c.execArm_SWI(inst)
		return

	case ARMITMultiply:
		fallthrough

	case ARMITTransferMRS:
		c.execArm_Mrs(inst)
		return

	case ARMITTransferMSR:
		c.execArm_Msr(inst)
		return

	case ARMITUndefined:
		fallthrough

	default:
		// panic on unknown instruction
		panic(fmt.Sprintf("Unimplemented ARM instruction: %08X at PC=%08X",
			instruction, c.Registers.PC-8))
	}
}

func (c *CPU) checkCondition_Arm(cond uint32) bool {
	// Extract flags from CPSR
	n := c.Registers.GetFlagN()
	z := c.Registers.GetFlagZ()
	c_flag := c.Registers.GetFlagC()
	v := c.Registers.GetFlagV()

	switch ARMCondition(cond) {
	case EQ:
		return z // Z set
	case NE:
		return !z // Z clear
	case CS:
		return c_flag // C set
	case CC:
		return !c_flag // C clear
	case MI:
		return n // N set
	case PL:
		return !n // N clear
	case VS:
		return v // V set
	case VC:
		return !v // V clear
	case HI:
		return c_flag && !z // C set and Z clear
	case LS:
		return !c_flag || z // C clear or Z set
	case GE:
		return n == v // N equals V
	case LT:
		return n != v // N not equal to V
	case GT:
		return !z && (n == v) // Z clear and N equals V
	case LE:
		return z || (n != v) // Z set or N not equal to V
	case AL:
		return true // Always execute
	case NV:
		return false // Never execute (undefined, effectively NOP)
	default:
		return false // Should not happen
	}
}

// ##################################################
// ARM Data Processings Instructions Implementations
// ##################################################

// execute ADD instruction
func (c *CPU) execArm_Add(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation between Rn and operand2
	result := c.Registers.GetReg(rn) + op2
	c.Registers.SetReg(rn, result)
	// Store result in the destination register (Rd)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
}

// execute ADC instruction
func (c *CPU) execArm_Adc(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation between Rn and operand2
	// TODO get cy (carry) from prev
	cy := uint32(0)
	result := c.Registers.GetReg(rn) + op2 + cy

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

// execute SBC instruction
func (c *CPU) execArm_Sbc(instruction ARMInstruction) {
	rn := instruction.Rn
	// rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)
	// TODO get cy (carry) from prev
	cy := uint32(0)
	// Perform the operation between Rn and operand2
	result := c.Registers.GetReg(rn) - op2 + cy - 1

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

// execute RSC instruction
func (c *CPU) execArm_Rsc(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)
	// TODO get cy (carry) from prev
	cy := uint32(0)
	// Perform the operation between Rn and operand2
	result := op2 - c.Registers.GetReg(rn) + cy - 1

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

// execute TST instruction
func (c *CPU) execArm_Tst(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) & op2

}

// execute TEQ instruction
func (c *CPU) execArm_Teq(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the XOR operation between Rn and operand2
	_ = c.Registers.GetReg(rn) ^ op2

}

// execute CMP instruction
func (c *CPU) execArm_Cmp(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) - op2

}

// execute CMN instruction
func (c *CPU) execArm_Cmn(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) + op2

}

// execute SUB instruction
func (c *CPU) execArm_Sub(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// TODO dbchk
	// Perform the operation between Rn and operand2
	result := c.Registers.GetReg(rn) - op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

// execute RSB instruction
func (c *CPU) execArm_Rsb(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation between Rn and operand2
	result := op2 - c.Registers.GetReg(rn)

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

func (c *CPU) execArm_And(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the AND operation between Rn and operand2
	result := c.Registers.GetReg(rn) & op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S && instruction.Rd != 15 {
		c.setFlags(result, carryOut, instruction)
	}

}

// execute ORR instruction
func (c *CPU) execArm_Orr(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the ORR operation between Rn and operand2
	result := c.Registers.GetReg(rn) | op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
}

// execute MOV instruction
func (c *CPU) execArm_Mov(instruction ARMInstruction) {
	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation
	result := op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
}

// execute BIC instruction
func (c *CPU) execArm_Bic(instruction ARMInstruction) {
	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation
	result := c.Registers.GetReg(instruction.Rn) & ^op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
}

// execute MVN instruction
func (c *CPU) execArm_Mvn(instruction ARMInstruction) {
	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the operation
	result := ^op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}
}

// execute EOR instruction
func (c *CPU) execArm_Eor(instruction ARMInstruction) {
	rn := instruction.Rn

	// Handle the shift operation for the second operand (Rm)
	op2, carryOut := c.calcOp2(instruction)

	// Perform the EOR operation between Rn and operand2
	result := c.Registers.GetReg(rn) ^ op2

	// Store result in the destination register (Rd)
	c.Registers.SetReg(instruction.Rd, result)

	// Set flags if the instruction specifies (S = true)
	if instruction.S {
		c.setFlags(result, carryOut, instruction)
	}

}

// #############################
// ARM Branch Instructions Implementations
// #############################

// execArm_Branch executes B and BL instructions.
// `currentInstructionAddr` is the address of the branch instruction itself.
func (c *CPU) execArm_Branch(inst ARMInstruction, currentInstructionAddr uint32) {

	if !c.checkCondition_Arm((currentInstructionAddr >> 28) & 0xF) {
		// Condition not met, so the branch is NOT taken.
		// PC should simply advance to the next instruction in sequence.
		c.Registers.PC = currentInstructionAddr + 4
		c.FlushPipeline() // Conditional branches still flush the pipeline if not taken
		return
	}

	// The offset is relative to PC+8 (i.e., current instruction address + 8)
	// This sign extension logic correctly handles the 26-bit value now in inst.TargetAddr

	// targetAddress = (address of branch instruction + 8) + signed_offset
	targetAddress := (currentInstructionAddr + 8) + uint32(inst.OffsetBranch)

	if inst.Link {
		// BL instruction: Save return address (address of next instruction after BL) to R14 (LR)
		// The return address is currentInstructionAddr + 4
		c.Registers.SetReg(14, currentInstructionAddr+4)
	}

	// Set PC to the target address
	c.Registers.PC = targetAddress
	c.FlushPipeline() // Branch flushes the pipeline
}

// #############################
// ARM Load/Store Instructions Implementations
// #############################

// execArm_LoadStore executes LDR and STR instructions with immediate offset.
// `currentInstructionAddr` is the address of the instruction itself.
func (c *CPU) execArm_LoadStore(inst ARMInstruction, currentInstructionAddr uint32) {
	baseAddr := c.Registers.GetReg(inst.Rn)
	offset := inst.Offset // 12-bit immediate offset

	// Determine the effective offset (add or subtract)
	effectiveOffset := offset
	if inst.U { // U=0 means subtract
		effectiveOffset = ^offset + 1 // Two's complement for subtraction
	}

	var finalAddr uint32

	// Calculate address based on P (Pre/Post-indexed)
	if inst.P { // Pre-indexed addressing
		finalAddr = baseAddr + uint32(effectiveOffset)
	} else { // Post-indexed addressing
		finalAddr = baseAddr // Use baseAddr for memory access first
	}

	// Perform Load (L=1) or Store (L=0)
	if inst.L { // Load (LDR)
		var loadedValue uint32
		if inst.B { // Byte transfer (LDRB)
			loadedValue = uint32(c.Bus.Read8(finalAddr))
		} else { // Word transfer (LDR)
			loadedValue = c.Bus.Read32(finalAddr)
		}

		// Write loaded value to Rd
		c.Registers.SetReg(inst.Rd, loadedValue)

		// Special case: If Rd is PC (R15), a branch occurs and state might change
		if inst.Rd == 15 {
			// If loading into PC, the pipeline is flushed.
			// Bit 0 of the loaded value determines the new state (ARM/Thumb).
			if (loadedValue & 0x1) != 0 {
				c.Registers.SetThumbState(true)
				c.Registers.PC = loadedValue & 0xFFFFFFFE // Halfword align for Thumb
			} else {
				c.Registers.SetThumbState(false)
				c.Registers.PC = loadedValue & 0xFFFFFFFC // Word align for ARM
			}
			c.FlushPipeline()
		}

	} else { // Store (STR)
		valueToStore := c.Registers.GetReg(inst.Rd)
		if inst.B { // Byte transfer (STRB)
			c.Bus.Write8(finalAddr, uint8(valueToStore))
		} else { // Word transfer (STR)
			c.Bus.Write32(finalAddr, valueToStore)
		}
	}

	// Handle Write-back (W=1)
	if inst.W {
		// If P=1 (Pre-indexed), the base address was already updated to finalAddr
		// If P=0 (Post-indexed), the base address needs to be updated after memory access
		if inst.P { // Post-indexed write-back
			c.Registers.SetReg(inst.Rn, baseAddr+uint32(effectiveOffset))
		} else { // Pre-indexed write-back (finalAddr already has the updated value)
			c.Registers.SetReg(inst.Rn, finalAddr)
		}
	}

	// No pipeline flush for LDR/STR unless Rd is PC
	if inst.Rd != 15 {
		// If Rd is not PC, the pipeline continues normally.
		// The PC was already incremented in Step().
	}
}

// #############################
// ARM Control Instructions Implementations
// #############################

// Implementation for execArm_SWI
// This function handles the Software Interrupt (SWI) instruction,
// causing an exception to Supervisor mode and jumping to the SWI vector.

// 1. Save return address (PC + 4) to R14_svc.
// In ARM7TDMI, PC points to current_instruction_address + 8.
// So, the address of the instruction *after* the SWI is (current_PC - 8) + 4 = current_PC - 4.
// A full emulator would use a banked R14_svc. For this example, we will store it in R14.
func (c *CPU) execArm_SWI(inst ARMInstruction) {
	// Implementation for execArm_SWI
	// This function handles the Software Interrupt (SWI) instruction,
	// causing an exception to Supervisor mode and jumping to the SWI vector.

	c.Registers.SetMode(SVCMode)
	// Save return address (PC + 4) to R14_svc.
	c.Registers.SetReg(14, c.Registers.PC-4)

	// 2. Save current CPSR to SPSR_svc.
	c.Registers.SetSPSR(c.Registers.CPSR)

	// 3. Change CPU mode to Supervisor (0x13).
	// Clear current mode bits (M4:0) and set to Supervisor mode.
	c.Registers.CPSR = (c.Registers.CPSR & 0xFFFFFFE0) | 0x13

	// 4. Set IRQ disable bit (I flag, bit 7) in CPSR to 1.
	c.Registers.CPSR |= (1 << 7) // Set I bit

	// 5. Set PC to SWI exception vector (0x00000008).
	// The CPU pipeline means the actual jump happens after fetching from this address.
	c.Registers.PC = 0x08
}

func (c *CPU) execArm_BlockDataTransfer(inst ARMInstruction, currentInstructionAddr uint32) {
	baseAddr := c.Registers.GetReg(inst.Rn)
	numRegisters := 0
	for i := 0; i < 16; i++ {
		if (inst.RegisterList>>i)&1 != 0 {
			numRegisters++
		}
	}

	var currentTransferAddr uint32
	var finalBaseAddr uint32

	// This logic determines the initial address for the first transfer
	if inst.U { // Up (Incrementing addresses)
		if inst.P { // Pre-indexed: increment before transfer (LDMIA/STMDB)
			currentTransferAddr = baseAddr + 4
		} else { // Post-indexed: use base, then increment (LDMIA/STMIA)
			currentTransferAddr = baseAddr
		}
		finalBaseAddr = baseAddr + uint32(numRegisters)*4
	} else { // Down (Decrementing addresses)
		if inst.P { // Pre-indexed: decrement before transfer (LDMDA/STMIB)
			currentTransferAddr = baseAddr - uint32(numRegisters)*4 + 4 // Address of the first actual transfer in decrementing order
		} else { // Post-indexed: use base, then decrement (LDMDA/STMDB)
			currentTransferAddr = baseAddr
		}
		finalBaseAddr = baseAddr - uint32(numRegisters)*4
	}

	// --- Transfer Logic ---
	// Iterate through registers (R0 to R15)
	for i := 0; i < 16; i++ {
		if (inst.RegisterList>>i)&1 != 0 { // If this register is in the list
			// Correct the initial address for decrementing post-indexed mode (STMDB/LDMDA)
			// If post-indexed and decrementing, the first address should be baseAddr - (numRegisters - 1) * 4
			// and then decrement by 4 each time.
			// Simplified this by adjusting currentTransferAddr correctly based on P and U at start.

			if inst.L { // LDM (Load Multiple)
				val := c.Bus.Read32(currentTransferAddr)
				// Special handling for PC (R15): If PC is loaded, it triggers a branch
				if i == 15 { // Corrected: Check against 15 directly
					c.Registers.SetReg(15, val&0xFFFFFFFC) // PC must be word-aligned for ARM mode
					c.FlushPipeline()                      // PC change requires flushing the pipeline
				} else {
					c.Registers.SetReg(uint8(i), val)
				}
			} else { // STM (Store Multiple)
				val := c.Registers.GetReg(uint8(i))
				// Special handling for PC (R15):
				// When R15 is stored, the value stored is PC + 12 (address of next instruction fetch + 4)
				if i == 15 { // Corrected: Check against 15 directly
					val = currentInstructionAddr + 12
				}
				c.Bus.Write32(currentTransferAddr, val)
			}

			// Adjust address for next transfer
			// This needs to be consistent: if P is true, currentTransferAddr was already adjusted BEFORE transfer,
			// so we just increment/decrement for the next one. If P is false, currentTransferAddr was used AS IS,
			// so we increment/decrement AFTER the transfer.
			if inst.U { // Up
				currentTransferAddr += 4
			} else { // Down
				currentTransferAddr -= 4
			}
		}
	}

	// --- Write-back Logic ---
	if inst.W {
		// If Rn is in the register list and it's an LDM, its final value is the one loaded from memory,
		// unless it's the last register. However, for simplicity and typical behavior,
		// the final base address is written back to Rn if W is set.
		c.Registers.SetReg(inst.Rn, finalBaseAddr)
	}

	// --- S-bit Handling (Optional) ---
	if inst.S {
		dbg.Printf("Warning: S-bit for Block Data Transfer (LDM/STM) is not fully emulated yet for instruction %s", inst.String())
	}
}

// #############################################
//   ARM Multiply Instructions Implementations
// #############################################

// ##################################################
//   ARM PSR Transfer Instructions Implementations
// ##################################################

func (c *CPU) execArm_Mrs(inst ARMInstruction) {
	// Determine which PSR to read from: CPSR (0) or SPSR_<current mode> (1)
	// The Psr bit (bit 22) indicates this.
	// In the ARMInstruction struct, this information isn't explicitly
	// stored as a separate boolean for MRS. It's implicitly part of the
	// "Psr" field in the opcode documentation.
	// We need to re-extract it from the raw instruction or modify ARMInstruction
	// to include it for PSR Transfer types.
	// Based on the documentation, bit 22 is Psr.
	rawInstruction := c.Bus.Read32(c.Registers.PC - 8)
	psrSourceBit := (rawInstruction >> 22) & 0x1

	var sourcePSR uint32
	if psrSourceBit == 0 { // CPSR
		sourcePSR = c.Registers.CPSR
	} else { // SPSR_<current mode>
		// In a real emulator, you'd need to determine the current mode
		// and access the correct SPSR. For simplicity, we'll assume a
		// common SPSR access, or panic if SPSR doesn't exist (e.g., User/System mode).
		if c.Registers.GetMode() == USRMode || c.Registers.GetMode() == SYSMode {
			panic(fmt.Sprintf("MRS: SPSR does not exist in current mode (%d)", c.Registers.GetMode()))
		}
		sourcePSR = c.Registers.GetSPSR()
	}

	// Rd = Psr
	c.Registers.SetReg(inst.Rd, sourcePSR)
}

// execArm_Msr executes the MSR (Move to PSR from Register/Immediate) instruction.
// MSR{cond} Psr{_field},Op
func (c *CPU) execArm_Msr(inst ARMInstruction) {
	// Determine which PSR to write to: CPSR (0) or SPSR_<current mode> (1)
	rawInstruction := c.Bus.Read32(c.Registers.PC - 8) // Assuming PC is already incremented
	psrDestBit := (rawInstruction >> 22) & 0x1

	// Extract field mask bits (f, s, x, c) from bits 19-16
	fieldMask := uint32((rawInstruction >> 16) & 0xF)
	writeFlags := ((fieldMask >> 3) & 0x1) == 1     // Bit 19 (f)
	writeStatus := ((fieldMask >> 2) & 0x1) == 1    // Bit 18 (s)
	writeExtension := ((fieldMask >> 1) & 0x1) == 1 // Bit 17 (x)
	writeControl := (fieldMask & 0x1) == 1          // Bit 16 (c)

	// Determine the operand value
	var operandValue uint32
	if inst.I { // Immediate operand
		// Immediate value already calculated and rotated by the decoder in inst.Immediate
		operandValue = inst.Immediate
	} else { // Register operand
		operandValue = c.Registers.GetReg(inst.Rm)
	}

	var targetPSR uint32
	if psrDestBit == 0 { // CPSR
		targetPSR = c.Registers.CPSR
	} else { // SPSR_<current mode>
		if c.Registers.GetMode() == USRMode || c.Registers.GetMode() == SYSMode {
			panic(fmt.Sprintf("MSR: SPSR does not exist in current mode (%d)", c.Registers.GetMode()))
		}
		targetPSR = c.Registers.GetSPSR()
	}

	currentPSRValue := targetPSR
	newPSRValue := currentPSRValue

	// Apply field masks
	if writeFlags {
		newPSRValue = (newPSRValue & ^PSR_FLAGS) | (operandValue & PSR_FLAGS)
	}
	if writeStatus {
		// Documentation states "reserved, don't change" for status field.
		// However, a real MSR might try to write to it, and the hardware
		// would simply ignore the write for those bits. For an emulator,
		// we can choose to warn, ignore, or strictly adhere to "don't change".
		// For now, let's allow the write but acknowledge the documentation.
		dbg.Printf("MSR: Attempting to write to reserved status field (bits 23-16)\n")
		newPSRValue = (newPSRValue & ^PSR_STATUS) | (operandValue & PSR_STATUS)
	}
	if writeExtension {
		// Documentation states "reserved, don't change" for extension field.
		dbg.Printf("MSR: Attempting to write to reserved extension field (bits 15-8)\n")
		newPSRValue = (newPSRValue & ^PSR_EXTENSION) | (operandValue & PSR_EXTENSION)
	}
	if writeControl {
		// In non-privileged mode (user mode): only condition code bits of CPSR can be changed, control bits can’t.
		if psrDestBit == 0 && c.Registers.GetMode() == USRMode { // Writing to CPSR in User Mode
			// Only allow writing to flags (bits 31-24)
			dbg.Printf("MSR: Attempting to write to control field (bits 7-0) in User mode. Only flags are writable.\n")
			// Only the condition code bits should be updated.
			// The flags field is part of PSR_FLAGS, which is already handled by writeFlags.
			// No action needed here to restrict control field write in User mode as we only update flags.
			// If we wanted to strictly prevent it, we would mask it out.
		} else {
			newPSRValue = (newPSRValue & ^PSR_CONTROL) | (operandValue & PSR_CONTROL)
		}
	}

	// The T-bit (bit 5) may not be changed; for THUMB/ARM switching use BX instruction.
	// Ensure the T-bit remains unchanged.
	// Preserve the original T-bit from the current PSR value.
	tBitOriginal := (currentPSRValue >> 5) & 0x1
	newPSRValue = (newPSRValue &^ (1 << 5)) | (tBitOriginal << 5)

	// Update the target PSR
	// TODO what the fuck. Why is this not a pointer?
	targetPSR = newPSRValue

	dbg.Printf("MSR: Writing 0x%08X to Psr (PsrDest: %d, Flags: %t, Status: %t, Ext: %t, Control: %t)\n",
		operandValue, psrDestBit, writeFlags, writeStatus, writeExtension, writeControl)
	dbg.Printf("MSR: New PSR value: 0x%08X\n", targetPSR)
}

// #############
// ### Utils ###
// #############

// applyShift performs the specified barrel shift operation on a value.
// It returns the shifted value and the carry-out bit from the shifter.
// This function handles various shift types and special shift amounts as per ARM architecture.
func (c *CPU) applyShift(value uint32, shiftType ARMShiftType, shiftAmount uint32) (uint32, bool) {
	carryOut := false

	if shiftAmount == 0 {
		if shiftType == ROR { // ROR #0 is RRX (Rotate Right Extended)
			carryOut = (value & 0x1) == 1                                                // Bit 0 of original value becomes C flag
			value = (value >> 1) | uint32(convert.BoolToInt(c.Registers.GetFlagC())<<31) // Old C flag into bit 31
		}
		// For LSL/LSR/ASR #0, no shift, carry is unchanged by shifter.
		return value, carryOut
	}

	switch shiftType {
	case LSL: // Logical Shift Left
		if shiftAmount >= 32 {
			if shiftAmount == 32 {
				carryOut = (value & 0x1) == 1 // Bit 0 of original value is shifted out
			} else { // shiftAmount > 32
				carryOut = false // Result is 0, carry is 0
			}
			value = 0
		} else {
			carryOut = (value>>(32-shiftAmount))&0x1 == 1
			value <<= shiftAmount
		}
	case LSR: // Logical Shift Right
		if shiftAmount >= 32 {
			if shiftAmount == 32 {
				carryOut = (value>>31)&0x1 == 1 // Bit 31 of original value is shifted out
			} else { // shiftAmount > 32
				carryOut = false // Result is 0, carry is 0
			}
			value = 0
		} else {
			carryOut = (value>>(shiftAmount-1))&0x1 == 1
			value >>= shiftAmount
		}
	case ASR: // Arithmetic Shift Right
		if shiftAmount >= 32 {
			carryOut = (value>>31)&0x1 == 1 // Bit 31 of original value is shifted out
			if carryOut {                   // If sign bit was 1, result is all 1s
				value = 0xFFFFFFFF
			} else { // If sign bit was 0, result is all 0s
				value = 0
			}
		} else {
			carryOut = (value>>(shiftAmount-1))&0x1 == 1
			// Arithmetic shift: preserve sign bit
			if (value & 0x80000000) != 0 { // If negative (MSB is 1)
				value = (value >> shiftAmount) | (0xFFFFFFFF << (32 - shiftAmount))
			} else { // If positive (MSB is 0)
				value >>= shiftAmount
			}
		}
	case ROR: // Rotate Right
		// ROR by N is equivalent to ROR by N % 32. If N % 32 is 0, it's ROR #0 (RRX).
		actualShift := shiftAmount % 32
		if actualShift == 0 { // ROR #0 is RRX, handled by the initial if block
			return c.applyShift(value, ROR, 0) // Recurse for RRX
		}
		// For ROR, the carry is the bit that was shifted out of bit 0.
		// This is the bit that was at position (actualShift - 1) of the original value.
		carryOut = (value>>(actualShift-1))&0x1 == 1
		value = (value >> actualShift) | (value << (32 - actualShift))
	}
	return value, carryOut
}

// calcOp2 calculates the second operand (Operand2) for Data Processing instructions.
// It handles both immediate and register-based operands, including shifts.
// It returns the calculated operand value and the carry-out from the barrel shifter.
func (c *CPU) calcOp2(instruction ARMInstruction) (uint32, bool) {
	if instruction.I { // Immediate operand
		// The immediate value (instruction.Immediate) is already rotated by DecodeARMInstruction.
		// For immediate operands, the carry-out from the barrel shifter is typically only
		// relevant for MOV/MVN instructions (when S bit is set).
		// The carry for ROR is the bit that was shifted out of bit 0 of the original 8-bit value,
		// which ends up at bit 31 of the rotated 32-bit result.
		carryOut := false
		if instruction.RotateImm != 0 { // If rotation occurred, the MSB of the result is the carry-out
			carryOut = (instruction.Immediate>>31)&0x1 == 1
		}
		return instruction.Immediate, carryOut
	} else { // Register operand
		rmVal := c.Registers.GetReg(instruction.Rm)
		var shiftAmount uint32

		// Determine if the shift amount is an immediate value or from a register.
		// DecodeARMInstruction already populates either ShiftImm or Rs.
		if instruction.Rs != 0 { // If Rs is non-zero, it implies a register shift (bit 4 was 1)
			shiftAmount = c.Registers.GetReg(instruction.Rs) & 0xFF // Only lower 8 bits of Rs are used for shift amount
		} else { // Otherwise, it's an immediate shift amount (bit 4 was 0)
			shiftAmount = uint32(instruction.ShiftImm)
		}

		// Perform the shift and get the carry-out from the barrel shifter.
		shiftedVal, carryOut := c.applyShift(rmVal, instruction.ShiftType, shiftAmount)
		return shiftedVal, carryOut
	}
}
