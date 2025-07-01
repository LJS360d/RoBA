package cpu

import (
	"GoBA/internal/interfaces"
	"GoBA/util/dbg"
	"fmt"
	"strconv"
)

// ARM7TDMI CPU operating modes
const (
	USRMode = 0b10000 // User mode
	FIQMode = 0b10001 // FIQ mode (Fast Interrupt Request)
	IRQMode = 0b10010 // IRQ mode (Interrupt Request)
	SVCMode = 0b10011 // Supervisor mode
	ABTMode = 0b10111 // Abort mode
	UNDMode = 0b11011 // Undefined instruction mode
	SYSMode = 0b11111 // System mode (shares User mode registers)
)

// Registers holds the state of the ARM7TDMI CPU registers.
// The GBA's ARM7TDMI has 16 general-purpose registers (R0-R15),
// though some are banked depending on the CPU mode.
// R0-R12: General purpose registers. In FIQ mode, R8-R12 are banked.
// R13: Stack Pointer (SP), banked.
// R14: Link Register (LR), banked.
// R15: Program Counter (PC).
type Registers struct {
	interfaces.RegistersInterface
	// General purpose registers R0-R7 (common to all modes)
	// and R8-R12 for User/System/IRQ/SVC/ABT/UND modes.
	// FIQ mode has its own R8_fiq-R12_fiq.
	R [13]uint32 // Stores R0-R12 for non-FIQ modes.

	// Banked registers for SP (R13) and LR (R14)
	SP_usr uint32 // R13_usr / R13_sys
	LR_usr uint32 // R14_usr / R14_sys

	SP_svc uint32 // R13_svc
	LR_svc uint32 // R14_svc

	SP_abt uint32 // R13_abt
	LR_abt uint32 // R14_abt

	SP_und uint32 // R13_und
	LR_und uint32 // R14_und

	SP_irq uint32 // R13_irq
	LR_irq uint32 // R14_irq

	// FIQ mode has its own R8-R12, SP, LR
	R8_fiq  uint32
	R9_fiq  uint32
	R10_fiq uint32
	R11_fiq uint32
	R12_fiq uint32
	SP_fiq  uint32 // R13_fiq
	LR_fiq  uint32 // R14_fiq

	// Program Counter R15
	PC uint32

	// Current Program Status Register
	CPSR uint32

	// Saved Program Status Registers (for exception handling)
	SPSR_svc uint32
	SPSR_abt uint32
	SPSR_und uint32
	SPSR_irq uint32
	SPSR_fiq uint32

	// Internal state for convenience (derived from CPSR, but can be cached)
	currentMode uint8
}

// NewRegisters creates and initializes a new Registers struct.
// CPU typically starts in Supervisor mode after reset.
// BIOS entry point is 0x00000000. PC is usually set after loading BIOS/ROM.
func NewRegisters() *Registers {
	regs := &Registers{
		// currentMode will be derived from CPSR, but initialize for clarity
		currentMode: SVCMode,
	}
	// Initialize CPSR: Supervisor mode, IRQ/FIQ disabled, ARM state.
	// Bits: M4-M0=Mode, T=Thumb, F=FIQ disable, I=IRQ disable, V,C,Z,N flags
	// SVCMode: 0b10011
	// ARM State (T=0)
	// FIQ Disabled (F=1)
	// IRQ Disabled (I=1)
	// Initial flags are typically 0
	regs.CPSR = uint32(SVCMode) | (1 << 7) | (1 << 6) // SVC, FIQ disabled, IRQ disabled, ARM state
	regs.currentMode = regs.GetMode()                 // Ensure currentMode matches initial CPSR
	return regs
}

// GetMode returns the current CPU operating mode from CPSR.
func (r *Registers) GetMode() uint8 {
	return uint8(r.CPSR & 0x1F) // Lower 5 bits define the mode
}

// SetMode updates the CPU operating mode in CPSR.
// This function is typically called when an exception occurs or when
// an MSR instruction writes to the mode bits of the CPSR.
// The GetReg/SetReg methods are responsible for accessing the correct
// physical (banked) registers based on the mode set in CPSR.
func (r *Registers) SetMode(mode uint8) {
	if r.GetMode() == mode {
		return // No change
	}

	// Update CPSR mode bits
	r.CPSR = (r.CPSR &^ 0x1F) | uint32(mode)
	r.currentMode = mode // Update internal convenience tracker

	// No explicit register value shuffling (like R[13] = SP_usr) is needed here.
	// The GetReg/SetReg methods will automatically use the correct banked register
	// fields (e.g., SP_usr, R8_fiq) based on the new mode in CPSR.
	// Each banked register (e.g., SP_svc, LR_fiq) retains its value independently.
}

// GetReg returns the value of a general-purpose register (R0-R15).
// It handles banked registers based on the current CPU mode.
// Note: R15 (PC) reads should ideally account for prefetch (PC+8 for ARM, PC+4 for Thumb).
// This simplified GetReg returns the raw PC value for now.
func (r *Registers) GetReg(regNum uint8) uint32 {
	if regNum > 15 {
		panic("read from undefined register R" + strconv.Itoa(int(regNum)))
	}

	mode := r.GetMode() // Use the mode from CPSR

	if regNum == 15 { // R15 is PC
		return r.PC
	}

	// Handle FIQ's banked R8-R12, SP, LR
	if mode == FIQMode {
		switch regNum {
		case 8:
			return r.R8_fiq
		case 9:
			return r.R9_fiq
		case 10:
			return r.R10_fiq
		case 11:
			return r.R11_fiq
		case 12:
			return r.R12_fiq
		case 13:
			return r.SP_fiq // R13_fiq
		case 14:
			return r.LR_fiq // R14_fiq
		}
	}

	// Handle banked SP (R13) and LR (R14) for non-FIQ modes
	if regNum == 13 { // SP
		switch mode {
		case USRMode, SYSMode:
			return r.SP_usr
		case SVCMode:
			return r.SP_svc
		case ABTMode:
			return r.SP_abt
		case UNDMode:
			return r.SP_und
		case IRQMode:
			return r.SP_irq
		default: // Should ideally not happen if mode is always valid
			dbg.Printf("Warning: GetReg(R13) in unknown mode %02X\n", mode)
			return r.SP_usr // Fallback, or panic
		}
	}

	if regNum == 14 { // LR
		switch mode {
		case USRMode, SYSMode:
			return r.LR_usr
		case SVCMode:
			return r.LR_svc
		case ABTMode:
			return r.LR_abt
		case UNDMode:
			return r.LR_und
		case IRQMode:
			return r.LR_irq
		default: // Should ideally not happen
			dbg.Printf("Warning: GetReg(R14) in unknown mode %02X\n", mode)
			return r.LR_usr // Fallback, or panic
		}
	}

	// For R0-R12 in non-FIQ modes (or R0-R7 in FIQ mode, as R8-R12 FIQ is handled above)
	// The R array stores R0-R12 for non-FIQ modes.
	return r.R[regNum]
}

// SetReg sets the value of a general-purpose register (R0-R15).
// It handles banked registers based on the current CPU mode.
// Writing to R15 (PC) performs a branch.
func (r *Registers) SetReg(regNum uint8, value uint32) {
	if regNum > 15 {
		panic("write to undefined register R" + strconv.Itoa(int(regNum)))
	}

	mode := r.GetMode() // Use the mode from CPSR

	if regNum == 15 { // R15 is PC
		r.PC = value
		return
	}

	// Handle FIQ's banked R8-R12, SP, LR
	if mode == FIQMode {
		switch regNum {
		case 8:
			r.R8_fiq = value
			return
		case 9:
			r.R9_fiq = value
			return
		case 10:
			r.R10_fiq = value
			return
		case 11:
			r.R11_fiq = value
			return
		case 12:
			r.R12_fiq = value
			return
		case 13:
			r.SP_fiq = value
			return // R13_fiq
		case 14:
			r.LR_fiq = value
			return // R14_fiq
		}
	}

	// Handle banked SP (R13) and LR (R14) for non-FIQ modes
	if regNum == 13 { // SP
		switch mode {
		case USRMode, SYSMode:
			r.SP_usr = value
			return
		case SVCMode:
			r.SP_svc = value
			return
		case ABTMode:
			r.SP_abt = value
			return
		case UNDMode:
			r.SP_und = value
			return
		case IRQMode:
			r.SP_irq = value
			return
		default: // Should ideally not happen
			dbg.Printf("Warning: SetReg(R13) in unknown mode %02X\n", mode)
			r.SP_usr = value // Fallback, or panic
			return
		}
	}

	if regNum == 14 { // LR
		switch mode {
		case USRMode, SYSMode:
			r.LR_usr = value
			return
		case SVCMode:
			r.LR_svc = value
			return
		case ABTMode:
			r.LR_abt = value
			return
		case UNDMode:
			r.LR_und = value
			return
		case IRQMode:
			r.LR_irq = value
			return
		default: // Should ideally not happen
			dbg.Printf("Warning: SetReg(R14) in unknown mode %02X\n", mode)
			r.LR_usr = value // Fallback, or panic
			return
		}
	}

	// For R0-R12 in non-FIQ modes (or R0-R7 in FIQ mode)
	r.R[regNum] = value
}

// GetSPSR returns the SPSR for the current mode.
// Only valid for exception modes. Returns 0 for USR/SYS (or could panic).
func (r *Registers) GetSPSR() uint32 {
	switch r.GetMode() { // Use mode from CPSR
	case FIQMode:
		return r.SPSR_fiq
	case SVCMode:
		return r.SPSR_svc
	case ABTMode:
		return r.SPSR_abt
	case IRQMode:
		return r.SPSR_irq
	case UNDMode:
		return r.SPSR_und
	case USRMode, SYSMode:
		// Accessing SPSR in USR or SYS mode is unpredictable/not allowed by MRS.
		// However, the SPSR fields for these modes don't exist.
		// For emulation, returning 0 or a known value might be okay, or logging a warning.
		// GBATEK: "SPSR is accessible in all privileged modes, but NOT in User mode."
		// "SPSR_usr and SPSR_sys do not exist"
		// Let's return CPSR as some emulators do, or 0. For now, 0.
		// dbg.Printf("Warning: GetSPSR() called in USR/SYS mode\n")
		return 0
	default:
		dbg.Printf("Warning: GetSPSR() in unknown mode %02X\n", r.GetMode())
		return 0 // Should not happen
	}
}

// SetSPSR sets the SPSR for the current mode.
// Only valid for exception modes. Does nothing for USR/SYS.
func (r *Registers) SetSPSR(value uint32) {
	currentActualMode := r.GetMode() // Use mode from CPSR
	switch currentActualMode {
	case FIQMode:
		r.SPSR_fiq = value
	case SVCMode:
		r.SPSR_svc = value
	case ABTMode:
		r.SPSR_abt = value
	case IRQMode:
		r.SPSR_irq = value
	case UNDMode:
		r.SPSR_und = value
	case USRMode, SYSMode:
		// SPSR_usr and SPSR_sys do not exist. MSR to SPSR in USR/SYS is unpredictable.
		// dbg.Printf("Warning: SetSPSR() called in USR/SYS mode. No action taken.\n")
		return
	default:
		dbg.Printf("Warning: SetSPSR() in unknown mode %02X\n", currentActualMode)
		return // Should not happen
	}
}

// --- CPSR Flag getters/setters ---

// IsThumb returns true if T flag in CPSR is set (Thumb state).
func (r *Registers) IsThumb() bool {
	return (r.CPSR>>5)&1 == 1
}

// SetThumbState sets or clears the T flag in CPSR.
func (r *Registers) SetThumbState(thumb bool) {
	if thumb {
		r.CPSR |= (1 << 5) // Set T bit
	} else {
		r.CPSR &^= (1 << 5) // Clear T bit
	}
}

// IsFIQDisabled returns true if F flag in CPSR is set (FIQ disabled).
func (r *Registers) IsFIQDisabled() bool {
	return (r.CPSR>>6)&1 == 1
}

// SetFIQDisabled sets or clears the F flag in CPSR.
func (r *Registers) SetFIQDisabled(disabled bool) {
	if disabled {
		r.CPSR |= (1 << 6) // Set F bit
	} else {
		r.CPSR &^= (1 << 6) // Clear F bit
	}
}

// IsIRQDisabled returns true if I flag in CPSR is set (IRQ disabled).
func (r *Registers) IsIRQDisabled() bool {
	return (r.CPSR>>7)&1 == 1
}

// SetIRQDisabled sets or clears the I flag in CPSR.
func (r *Registers) SetIRQDisabled(disabled bool) {
	if disabled {
		r.CPSR |= (1 << 7) // Set I bit
	} else {
		r.CPSR &^= (1 << 7) // Clear I bit
	}
}

// GetFlagN returns the N (Negative) flag from CPSR.
func (r *Registers) GetFlagN() bool { return (r.CPSR>>31)&1 == 1 }

// GetFlagZ returns the Z (Zero) flag from CPSR.
func (r *Registers) GetFlagZ() bool { return (r.CPSR>>30)&1 == 1 }

// GetFlagC returns the C (Carry) flag from CPSR.
func (r *Registers) GetFlagC() bool { return (r.CPSR>>29)&1 == 1 }

// GetFlagV returns the V (Overflow) flag from CPSR.
func (r *Registers) GetFlagV() bool { return (r.CPSR>>28)&1 == 1 }

// SetFlagN sets the N flag in CPSR.
func (r *Registers) SetFlagN(set bool) {
	if set {
		r.CPSR |= (1 << 31)
	} else {
		r.CPSR &^= (1 << 31)
	}
}

// SetFlagZ sets the Z flag in CPSR.
func (r *Registers) SetFlagZ(set bool) {
	if set {
		r.CPSR |= (1 << 30)
	} else {
		r.CPSR &^= (1 << 30)
	}
}

// SetFlagC sets the C flag in CPSR.
func (r *Registers) SetFlagC(set bool) {
	if set {
		r.CPSR |= (1 << 29)
	} else {
		r.CPSR &^= (1 << 29)
	}
}

// SetFlagV sets the V flag in CPSR.
func (r *Registers) SetFlagV(set bool) {
	if set {
		r.CPSR |= (1 << 28)
	} else {
		r.CPSR &^= (1 << 28)
	}
}

// String returns a string representation of the registers for debugging.
func (r *Registers) String() string {
	mode := r.GetMode()
	modeStr := ""
	switch mode {
	case USRMode:
		modeStr = "USR"
	case FIQMode:
		modeStr = "FIQ"
	case IRQMode:
		modeStr = "IRQ"
	case SVCMode:
		modeStr = "SVC"
	case ABTMode:
		modeStr = "ABT"
	case UNDMode:
		modeStr = "UND"
	case SYSMode:
		modeStr = "SYS"
	default:
		modeStr = fmt.Sprintf("?%02X?", mode)
	}

	thumbState := "ARM"
	if r.IsThumb() {
		thumbState = "THUMB"
	}

	// Use GetReg for R0-R15 to ensure banked registers are shown correctly for current mode
	return fmt.Sprintf(
		"R0 =%08X  R1 =%08X  R2 =%08X  R3 =%08X\n"+
			"R4 =%08X  R5 =%08X  R6 =%08X  R7 =%08X\n"+
			"R8 =%08X  R9 =%08X  R10=%08X  R11=%08X\n"+
			"R12=%08X  SP =%08X  LR =%08X  PC =%08X\n"+
			"CPSR=%08X (%s %s N:%t Z:%t C:%t V:%t I:%t F:%t)\n"+
			"SPSR=%08X (current mode's SPSR, if applicable)",
		r.GetReg(0), r.GetReg(1), r.GetReg(2), r.GetReg(3),
		r.GetReg(4), r.GetReg(5), r.GetReg(6), r.GetReg(7),
		r.GetReg(8), r.GetReg(9), r.GetReg(10), r.GetReg(11),
		r.GetReg(12), r.GetReg(13), r.GetReg(14), r.GetReg(15), // PC
		r.CPSR, modeStr, thumbState,
		r.GetFlagN(), r.GetFlagZ(), r.GetFlagC(), r.GetFlagV(),
		r.IsIRQDisabled(), r.IsFIQDisabled(),
		r.GetSPSR(), // SPSR for the current mode
	)
}
