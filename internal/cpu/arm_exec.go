package cpu

import "fmt"

// execute ARM instruction based on opcode.
func (c *CPU) execute_Arm(instruction uint32) {
	// Extract condition code
	cond := (instruction >> 28) & 0xF
	// Check condition

	if !c.checkCondition_Arm(cond) {
		// fmt.Println("NOP")
		return // Condition not met, treat as NOP
	}
	decoded := DecodeInstruction_Arm(instruction)
	fmt.Printf("execute_Arm: %08X\n", instruction)
	switch inst := decoded.(type) {
	case ARMDataProcessingInstruction:
		// Handle DataProcessingInstruction
		switch inst.Opcode {
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

	case ARMLoadStoreInstruction:
		c.execArm_LoadStore(inst, c.Registers.PC-8)
		return

	case ARMBranchInstruction:
		c.execArm_Branch(inst, c.Registers.PC-8)
		return

	// Control Instructions
	case ARMBlockDataTransferInstruction:
		c.execArm_BlockDataTransfer(inst, c.Registers.PC-8)
		return

	case ARMSWIInstruction:
		c.execArm_SWI(inst)
		return

	case ARMControlInstruction:
		panic(fmt.Sprintf("Unhandled ARM control instruction: %08X at PC=%08X",
			instruction, c.Registers.PC-8))
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
func (c *CPU) execArm_Add(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM ADD R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute ADC instruction
func (c *CPU) execArm_Adc(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM ADC (Add with Carry) R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute SBC instruction
func (c *CPU) execArm_Sbc(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM SBC (Subtract with Carry) R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute RSC instruction
func (c *CPU) execArm_Rsc(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM RSC (Reversed Subtract with Carry) R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute TST instruction
func (c *CPU) execArm_Tst(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) & op2

	fmt.Printf("ARM TST R%d, R%d, Operand2: %d\n", rn, rm, op2)
}

// execute TEQ instruction
func (c *CPU) execArm_Teq(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the XOR operation between Rn and operand2
	_ = c.Registers.GetReg(rn) ^ op2

	fmt.Printf("ARM TEQ R%d, R%d, Operand2: %d\n", rn, rm, op2)
}

// execute CMP instruction
func (c *CPU) execArm_Cmp(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) - op2

	fmt.Printf("ARM CMP R%d, R%d, Operand2: %d\n", rn, rm, op2)
}

// execute CMN instruction
func (c *CPU) execArm_Cmn(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

	// Handle the shift operation for the second operand (Rm)
	op2, _ := c.calcOp2(instruction)
	// Perform the operation between Rn and operand2
	_ = c.Registers.GetReg(rn) + op2

	fmt.Printf("ARM CMN R%d, R%d, Operand2: %d\n", rn, rm, op2)
}

// execute SUB instruction
func (c *CPU) execArm_Sub(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM SUB R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute RSB instruction
func (c *CPU) execArm_Rsb(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM RSB (Reverse Sub) R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

func (c *CPU) execArm_And(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM AND R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute ORR instruction
func (c *CPU) execArm_Orr(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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
	fmt.Printf("ARM ORR R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

// execute MOV instruction
func (c *CPU) execArm_Mov(instruction ARMDataProcessingInstruction) {
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
	fmt.Printf("ARM MOV Operand2: %d, Result = %d\n", op2, result)
}

// execute BIC instruction
func (c *CPU) execArm_Bic(instruction ARMDataProcessingInstruction) {
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
	fmt.Printf("ARM BIC (Bit Clear) Operand2: %d, Result = %d\n", op2, result)
}

// execute MVN instruction
func (c *CPU) execArm_Mvn(instruction ARMDataProcessingInstruction) {
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
	fmt.Printf("ARM MVN (Not) Operand2: %d, Result = %d\n", op2, result)
}

// execute EOR instruction
func (c *CPU) execArm_Eor(instruction ARMDataProcessingInstruction) {
	rn := instruction.Rn
	rm := instruction.Rm

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

	fmt.Printf("ARM EOR R%d, R%d, Operand2: %d, Result = %d\n", rn, rm, op2, result)
}

func (c *CPU) calcOp2(instruction ARMDataProcessingInstruction) (uint32, bool) {
	if instruction.I {
		// Immediate operand case: instruction uses a rotated immediate value
		// Apply ROR to the immediate value (instruction.Nn) by instruction.Is * 2
		rotatedImmediate := applyShift(uint32(instruction.Nn), ROR, uint32(instruction.Is*2))
		carryOut := (instruction.Is != 0) && (uint32(instruction.Nn)&0x80000000 != 0) // Carry from the ROR
		return rotatedImmediate, carryOut
	} else {
		// Register operand case: Rm can be shifted by Is or Rs
		rm := c.Registers.GetReg(instruction.Rm)
		if instruction.ShiftType < 4 {
			return applyShift(rm, instruction.ShiftType, uint32(instruction.Is)), (rm & (1 << (instruction.Is - 1))) != 0
		}
	}

	return 0, false // Default case, shouldn't be hit
}

// #############################
// ARM Branch Instructions Implementations
// #############################

// execArm_Branch executes B and BL instructions.
// `currentInstructionAddr` is the address of the branch instruction itself.
func (c *CPU) execArm_Branch(inst ARMBranchInstruction, currentInstructionAddr uint32) {

	if !c.checkCondition_Arm((currentInstructionAddr >> 28) & 0xF) {
		// Condition not met, so the branch is NOT taken.
		// PC should simply advance to the next instruction in sequence.
		c.Registers.PC = currentInstructionAddr + 4
		c.FlushPipeline()                                                                              // Conditional branches still flush the pipeline if not taken
		fmt.Printf("Conditional Branch (Cond: %X) not taken. PC to %08X\n", inst.Cond, c.Registers.PC) // Optional debug log
		return
	}

	// The offset is relative to PC+8 (i.e., current instruction address + 8)
	// This sign extension logic correctly handles the 26-bit value now in inst.TargetAddr
	var signedOffset int32
	if (inst.TargetAddr & 0x02000000) != 0 { // Checks bit 25, the sign bit of a 26-bit value
		signedOffset = int32(inst.TargetAddr | 0xFC000000) // Correctly sign-extends 26-bit to 32-bit
	} else {
		signedOffset = int32(inst.TargetAddr)
	}

	// targetAddress = (address of branch instruction + 8) + signed_offset
	targetAddress := (currentInstructionAddr + 8) + uint32(signedOffset)

	if inst.Link {
		// BL instruction: Save return address (address of next instruction after BL) to R14 (LR)
		// The return address is currentInstructionAddr + 4
		c.Registers.SetReg(14, currentInstructionAddr+4)
	}

	// Set PC to the target address
	c.Registers.PC = targetAddress
	c.FlushPipeline() // Branch flushes the pipeline
	fmt.Printf("Branch to %08X\n", targetAddress)
}

// #############################
// ARM Load/Store Instructions Implementations
// #############################

// execArm_LoadStore executes LDR and STR instructions with immediate offset.
// `currentInstructionAddr` is the address of the instruction itself.
func (c *CPU) execArm_LoadStore(inst ARMLoadStoreInstruction, currentInstructionAddr uint32) {
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
		fmt.Printf("Pre-indexed address: %08X\n", finalAddr)
	} else { // Post-indexed addressing
		finalAddr = baseAddr // Use baseAddr for memory access first
		fmt.Printf("Post-indexed address: %08X\n", finalAddr)
	}

	// Perform Load (L=1) or Store (L=0)
	if inst.L { // Load (LDR)
		var loadedValue uint32
		if inst.B { // Byte transfer (LDRB)
			loadedValue = uint32(c.Bus.Read8(finalAddr))
			fmt.Printf("LDRB R%d, [%08X] = %02X\n", inst.Rd, finalAddr, loadedValue)
		} else { // Word transfer (LDR)
			loadedValue = c.Bus.Read32(finalAddr)
			fmt.Printf("LDR R%d, [%08X] = %08X\n", inst.Rd, finalAddr, loadedValue)
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
			fmt.Printf("LDR to PC, new PC: %08X, Thumb: %t\n", c.Registers.PC, c.Registers.IsThumb())
		}

	} else { // Store (STR)
		valueToStore := c.Registers.GetReg(inst.Rd)
		if inst.B { // Byte transfer (STRB)
			c.Bus.Write8(finalAddr, uint8(valueToStore))
			fmt.Printf("STRB R%d, [%08X] = %02X\n", inst.Rd, finalAddr, uint8(valueToStore))
		} else { // Word transfer (STR)
			c.Bus.Write32(finalAddr, valueToStore)
			fmt.Printf("STR R%d, [%08X] = %08X\n", inst.Rd, finalAddr, valueToStore)
		}
	}

	// Handle Write-back (W=1)
	if inst.W {
		// If P=1 (Pre-indexed), the base address was already updated to finalAddr
		// If P=0 (Post-indexed), the base address needs to be updated after memory access
		if inst.P { // Post-indexed write-back
			c.Registers.SetReg(inst.Rn, baseAddr+uint32(effectiveOffset))
			fmt.Printf("Post-indexed write-back to R%d: %08X\n", inst.Rn, baseAddr+uint32(effectiveOffset))
		} else { // Pre-indexed write-back (finalAddr already has the updated value)
			c.Registers.SetReg(inst.Rn, finalAddr)
			fmt.Printf("Pre-indexed write-back to R%d: %08X\n", inst.Rn, finalAddr)
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

func (c *CPU) execArm_SWI(inst ARMSWIInstruction) {
	fmt.Printf("SWI\n")
	panic("Unimplemented")
}

func (c *CPU) execArm_BlockDataTransfer(inst ARMBlockDataTransferInstruction, currentInstructionAddr uint32) {
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
					fmt.Printf("BDF: PC = %08X\n", val&0xFFFFFFFC)
					c.FlushPipeline() // PC change requires flushing the pipeline
				} else {
					c.Registers.SetReg(uint8(i), val)
					fmt.Printf("BDF: R%d = %08X\n", i, val)
				}
			} else { // STM (Store Multiple)
				val := c.Registers.GetReg(uint8(i))
				// Special handling for PC (R15):
				// When R15 is stored, the value stored is PC + 12 (address of next instruction fetch + 4)
				if i == 15 { // Corrected: Check against 15 directly
					val = currentInstructionAddr + 12
				}
				c.Bus.Write32(currentTransferAddr, val)
				fmt.Printf("BDF: Bus Write32 %08X = %08X\n", currentTransferAddr, val)
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
		fmt.Printf("BDF: R%d (Rn) written back = %08X\n", inst.Rn, finalBaseAddr)
	}

	// --- S-bit Handling (Optional) ---
	if inst.S {
		fmt.Printf("Warning: S-bit for Block Data Transfer (LDM/STM) is not fully emulated yet for instruction %08X\n", inst.ARMInstruction)
	}
}

// #############
// ### Utils ###
// #############

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
