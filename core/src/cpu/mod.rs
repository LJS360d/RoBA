use std::fmt;
use crate::bus::BusAccess;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CpuState { Arm, Thumb }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CpuMode {
    User,
    Fiq,
    Irq,
    Supervisor,
    Abort,
    Undefined,
    System,
}

impl CpuMode {
    fn from_bits(bits: u32) -> Self {
        match bits & 0x1F {
            0b10000 => CpuMode::User,
            0b10001 => CpuMode::Fiq,
            0b10010 => CpuMode::Irq,
            0b10011 => CpuMode::Supervisor,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            0b11111 => CpuMode::System,
            _ => CpuMode::User,
        }
    }

    fn to_bits(self) -> u32 {
        match self {
            CpuMode::User => 0b10000,
            CpuMode::Fiq => 0b10001,
            CpuMode::Irq => 0b10010,
            CpuMode::Supervisor => 0b10011,
            CpuMode::Abort => 0b10111,
            CpuMode::Undefined => 0b11011,
            CpuMode::System => 0b11111,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Cpsr(u32);

impl fmt::Debug for Cpsr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cpsr")
            .field("N", &self.n())
            .field("Z", &self.z())
            .field("C", &self.c())
            .field("V", &self.v())
            .field("Q", &self.q())
            .field("I", &self.i())
            .field("F", &self.f())
            .field("T", &self.t())
            .field("mode", &self.mode())
            .finish()
    }
}

impl Cpsr {
    pub fn new() -> Self { Self(0x0000_0010) } // User mode, ARM state by default

    pub fn raw(&self) -> u32 { self.0 }
    pub fn set_raw(&mut self, value: u32) { self.0 = value; }

    pub fn n(&self) -> bool { self.0 & (1 << 31) != 0 }
    pub fn z(&self) -> bool { self.0 & (1 << 30) != 0 }
    pub fn c(&self) -> bool { self.0 & (1 << 29) != 0 }
    pub fn v(&self) -> bool { self.0 & (1 << 28) != 0 }
    pub fn q(&self) -> bool { self.0 & (1 << 27) != 0 }
    pub fn i(&self) -> bool { self.0 & (1 << 7) != 0 }
    pub fn f(&self) -> bool { self.0 & (1 << 6) != 0 }
    pub fn t(&self) -> bool { self.0 & (1 << 5) != 0 }

    pub fn set_n(&mut self, v: bool) { self.set_bit(31, v); }
    pub fn set_z(&mut self, v: bool) { self.set_bit(30, v); }
    pub fn set_c(&mut self, v: bool) { self.set_bit(29, v); }
    pub fn set_v(&mut self, v: bool) { self.set_bit(28, v); }
    pub fn set_q(&mut self, v: bool) { self.set_bit(27, v); }
    pub fn set_i(&mut self, v: bool) { self.set_bit(7, v); }
    pub fn set_f(&mut self, v: bool) { self.set_bit(6, v); }
    pub fn set_t(&mut self, v: bool) { self.set_bit(5, v); }

    fn set_bit(&mut self, bit: u32, set: bool) { if set { self.0 |= 1 << bit } else { self.0 &= !(1 << bit) } }

    pub fn mode(&self) -> CpuMode { CpuMode::from_bits(self.0) }
    pub fn set_mode(&mut self, mode: CpuMode) {
        self.0 = (self.0 & !0x1F) | mode.to_bits();
    }

    pub fn state(&self) -> CpuState { if self.t() { CpuState::Thumb } else { CpuState::Arm } }
    pub fn set_state(&mut self, state: CpuState) { self.set_t(matches!(state, CpuState::Thumb)); }
}

#[derive(Default, Clone)]
struct BankedRegs {
    r8_fiq: [u32; 5],   // r8..r12 for FIQ
    r8_shared: [u32; 5], // r8..r12 shared across non-FIQ modes
    r13_banked: [u32; 7], // r13 for: USR/SYS, FIQ, IRQ, SVC, ABT, UND (index by mode mapping)
    r14_banked: [u32; 7], // r14 for same
    spsr_banked: [u32; 6], // SPSR for: FIQ, IRQ, SVC, ABT, UND (USR/SYS none). We'll map with helper.
}

impl BankedRegs {
    fn new() -> Self { Self::default() }
}

#[derive(Default, Clone)]
struct ArmPipeline {
    fetch: u32,
    decode: u32,
    valid: bool,
}

pub struct Cpu {
    // Unbanked base registers hold the current view for r0..r15
    regs: [u32; 16],
    cpsr: Cpsr,
    banked: BankedRegs,
    arm_pipe: ArmPipeline,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self { regs: [0; 16], cpsr: Cpsr::new(), banked: BankedRegs::new(), arm_pipe: ArmPipeline::default() };
        cpu.cpsr.set_mode(CpuMode::System);
        cpu.regs[13] = 0; // SP
        cpu.regs[15] = 0; // PC
        // Initialize shared r8..r12 snapshot from the current (System) view
        cpu.banked.r8_shared.copy_from_slice(&cpu.regs[8..=12]);
        cpu
    }

    pub fn cpsr(&self) -> Cpsr { self.cpsr }
    pub fn cpsr_mut(&mut self) -> &mut Cpsr { &mut self.cpsr }

    pub fn read_reg(&self, index: usize) -> u32 { self.regs[index] }
    pub fn write_reg(&mut self, index: usize, value: u32) { self.regs[index] = value; }

    pub fn mode(&self) -> CpuMode { self.cpsr.mode() }
    pub fn state(&self) -> CpuState { self.cpsr.state() }

    pub fn spsr(&self) -> Option<u32> { self.spsr_for_mode(self.mode()) }
    pub fn set_spsr(&mut self, value: u32) { self.set_spsr_for_mode(self.mode(), value); }

    pub fn set_mode(&mut self, new_mode: CpuMode) {
        let old_mode = self.mode();
        if old_mode == new_mode { return; }
        self.save_banked(old_mode);
        self.cpsr.set_mode(new_mode);
        self.restore_banked(new_mode);
    }

    fn bank_index_for_r13_r14(mode: CpuMode) -> usize {
        match mode {
            CpuMode::User | CpuMode::System => 0,
            CpuMode::Fiq => 1,
            CpuMode::Irq => 2,
            CpuMode::Supervisor => 3,
            CpuMode::Abort => 4,
            CpuMode::Undefined => 5,
        }
    }

    fn spsr_index_for_mode(mode: CpuMode) -> Option<usize> {
        match mode {
            CpuMode::User | CpuMode::System => None,
            CpuMode::Fiq => Some(0),
            CpuMode::Irq => Some(1),
            CpuMode::Supervisor => Some(2),
            CpuMode::Abort => Some(3),
            CpuMode::Undefined => Some(4),
        }
    }

    fn save_banked(&mut self, mode: CpuMode) {
        // Save r8..r12 set based on current mode
        if matches!(mode, CpuMode::Fiq) {
            self.banked.r8_fiq.copy_from_slice(&self.regs[8..=12]);
        } else {
            self.banked.r8_shared.copy_from_slice(&self.regs[8..=12]);
        }
        // Save r13/r14 for this mode
        let idx = Self::bank_index_for_r13_r14(mode);
        self.banked.r13_banked[idx] = self.regs[13];
        self.banked.r14_banked[idx] = self.regs[14];
        // Do not implicitly modify SPSR here. It is set explicitly by exception entry/MSR.
    }

    fn restore_banked(&mut self, mode: CpuMode) {
        // Restore r8..r12 depending on target mode
        if matches!(mode, CpuMode::Fiq) {
            self.regs[8..=12].copy_from_slice(&self.banked.r8_fiq);
        } else {
            self.regs[8..=12].copy_from_slice(&self.banked.r8_shared);
        }
        // Restore r13/r14 for this mode
        let idx = Self::bank_index_for_r13_r14(mode);
        self.regs[13] = self.banked.r13_banked[idx];
        self.regs[14] = self.banked.r14_banked[idx];
    }

    fn spsr_for_mode(&self, mode: CpuMode) -> Option<u32> {
        Self::spsr_index_for_mode(mode).map(|i| self.banked.spsr_banked[i])
    }

    fn set_spsr_for_mode(&mut self, mode: CpuMode, value: u32) {
        if let Some(i) = Self::spsr_index_for_mode(mode) {
            self.banked.spsr_banked[i] = value;
        }
    }

    // ----- Barrel shifter -----
    pub fn lsl_with_carry(value: u32, amount: u32, carry_in: bool, immediate: bool) -> (u32, bool) {
        if amount == 0 {
            return (value, carry_in);
        }
        if immediate {
            let n = amount;
            if n < 32 {
                let result = value.wrapping_shl(n);
                let carry = ((value >> (32 - n)) & 1) != 0;
                (result, carry)
            } else if n == 32 {
                let result = 0u32;
                let carry = (value & 1) != 0;
                (result, carry)
            } else {
                (0, false)
            }
        } else {
            let n = amount & 0xFF;
            if n == 0 { return (value, carry_in); }
            if n < 32 {
                let result = value.wrapping_shl(n);
                let carry = ((value >> (32 - n)) & 1) != 0;
                (result, carry)
            } else if n == 32 {
                let result = 0u32;
                let carry = (value & 1) != 0;
                (result, carry)
            } else {
                (0, false)
            }
        }
    }

    pub fn lsr_with_carry(value: u32, amount: u32, carry_in: bool, immediate: bool) -> (u32, bool) {
        if immediate {
            let n = amount;
            if n == 0 {
                let result = 0u32;
                let carry = (value >> 31) != 0;
                return (result, carry);
            }
            if n < 32 {
                let result = value.wrapping_shr(n);
                let carry = ((value >> (n - 1)) & 1) != 0;
                (result, carry)
            } else if n == 32 {
                (0, (value >> 31) != 0)
            } else {
                (0, false)
            }
        } else {
            let n = amount & 0xFF;
            if n == 0 { return (value, carry_in); }
            if n < 32 {
                let result = value.wrapping_shr(n);
                let carry = ((value >> (n - 1)) & 1) != 0;
                (result, carry)
            } else if n == 32 {
                (0, (value >> 31) != 0)
            } else {
                (0, false)
            }
        }
    }

    pub fn asr_with_carry(value: u32, amount: u32, carry_in: bool, immediate: bool) -> (u32, bool) {
        let sign = (value >> 31) != 0;
        if immediate {
            let n = amount;
            if n == 0 {
                let result = if sign { u32::MAX } else { 0 };
                let carry = (value >> 31) != 0;
                return (result, carry);
            }
            if n < 32 {
                let result = ((value as i32) >> n) as u32;
                let carry = ((value >> (n - 1)) & 1) != 0;
                (result, carry)
            } else {
                let result = if sign { u32::MAX } else { 0 };
                let carry = sign;
                (result, carry)
            }
        } else {
            let n = amount & 0xFF;
            if n == 0 { return (value, carry_in); }
            if n < 32 {
                let result = ((value as i32) >> n) as u32;
                let carry = ((value >> (n - 1)) & 1) != 0;
                (result, carry)
            } else {
                let result = if sign { u32::MAX } else { 0 };
                let carry = sign;
                (result, carry)
            }
        }
    }

    pub fn ror_with_carry(value: u32, amount: u32, carry_in: bool, immediate: bool) -> (u32, bool) {
        if immediate {
            let n = amount & 31;
            if amount == 0 {
                let result = ((carry_in as u32) << 31) | (value >> 1);
                let carry = (value & 1) != 0;
                return (result, carry);
            }
            if n == 0 {
                (value, (value >> 31) != 0)
            } else {
                let result = value.rotate_right(n);
                let carry = (result >> 31) != 0;
                (result, carry)
            }
        } else {
            let amt = amount & 0xFF;
            if amt == 0 { return (value, carry_in); }
            let n = amt & 31;
            if n == 0 {
                (value, (value >> 31) != 0)
            } else {
                let result = value.rotate_right(n);
                let carry = (result >> 31) != 0;
                (result, carry)
            }
        }
    }

    // ----- Condition evaluation -----
    fn condition_passed(&self, cond: u32) -> bool {
        let n = self.cpsr.n();
        let z = self.cpsr.z();
        let c = self.cpsr.c();
        let v = self.cpsr.v();
        match cond {
            0x0 => z,                         // EQ
            0x1 => !z,                        // NE
            0x2 => c,                         // CS/HS
            0x3 => !c,                        // CC/LO
            0x4 => n,                         // MI
            0x5 => !n,                        // PL
            0x6 => v,                         // VS
            0x7 => !v,                        // VC
            0x8 => c && !z,                   // HI
            0x9 => !c || z,                   // LS
            0xA => n == v,                    // GE
            0xB => n != v,                    // LT
            0xC => !z && (n == v),            // GT
            0xD => z || (n != v),             // LE
            0xE => true,                      // AL
            0xF => false,                     // NV (reserved / never)
            _ => true,
        }
    }

    // ----- Operand2 decode and shift -----
    fn decode_operand2(&self, opcode: u32) -> (u32, bool) {
        let i = (opcode >> 25) & 1;
        if i == 1 {
            // Immediate: rotate right even number of bits
            let imm8 = opcode & 0xFF;
            let rot = ((opcode >> 8) & 0xF) * 2;
            if rot == 0 {
                (imm8, self.cpsr.c())
            } else {
                let val = (imm8 as u32).rotate_right(rot);
                let carry = (val >> 31) != 0;
                (val, carry)
            }
        } else {
            // Register shifter operand
            let rm = (opcode & 0xF) as usize;
            let typ = (opcode >> 5) & 0x3; // 0 LSL,1 LSR,2 ASR,3 ROR
            let by_reg = ((opcode >> 4) & 1) == 1;
            if by_reg {
                let rs = ((opcode >> 8) & 0xF) as usize;
                let amount = self.regs[rs] & 0xFF;
                match typ {
                    0 => Self::lsl_with_carry(self.regs[rm], amount, self.cpsr.c(), false),
                    1 => Self::lsr_with_carry(self.regs[rm], amount, self.cpsr.c(), false),
                    2 => Self::asr_with_carry(self.regs[rm], amount, self.cpsr.c(), false),
                    _ => Self::ror_with_carry(self.regs[rm], amount, self.cpsr.c(), false),
                }
            } else {
                let imm5 = (opcode >> 7) & 0x1F;
                match typ {
                    0 => Self::lsl_with_carry(self.regs[rm], imm5, self.cpsr.c(), true),
                    1 => Self::lsr_with_carry(self.regs[rm], imm5, self.cpsr.c(), true),
                    2 => Self::asr_with_carry(self.regs[rm], imm5, self.cpsr.c(), true),
                    _ => Self::ror_with_carry(self.regs[rm], imm5, self.cpsr.c(), true),
                }
            }
        }
    }

    // ----- Flag helpers -----
    fn add_with_carry(a: u32, b: u32, carry: bool) -> (u32, bool, bool) {
        let carry_in = if carry { 1u64 } else { 0u64 };
        let ua = a as u64;
        let ub = b as u64;
        let sum = ua + ub + carry_in;
        let result = sum as u32;
        let c_out = sum > 0xFFFF_FFFF;
        // Signed overflow: if signs of a and b same, and sign of result differs from a
        let sa = (a >> 31) & 1;
        let sb = (b >> 31) & 1;
        let sr = (result >> 31) & 1;
        let v_out = (sa == sb) && (sa != sr);
        (result, c_out, v_out)
    }

    fn sub_with_borrow(a: u32, b: u32, borrow: bool) -> (u32, bool, bool) {
        // SBC semantics: a - b - (1 - C)
        let borrow_in = if borrow { 0u64 } else { 1u64 };
        let ua = a as u64;
        let ub = b as u64;
        let diff = ua.wrapping_sub(ub).wrapping_sub(borrow_in);
        let result = diff as u32;
        // C flag is NOT borrow: set when a >= b + borrow_in
        let c_out = ua >= (ub + borrow_in);
        // Signed overflow: if signs of a and b differ, and sign of result differs from a
        let sa = (a >> 31) & 1;
        let sb = (b >> 31) & 1;
        let sr = (result >> 31) & 1;
        let v_out = (sa != sb) && (sa != sr);
        (result, c_out, v_out)
    }

    // ----- Execute ARM Data-Processing -----
    pub fn execute_arm_data_processing(&mut self, opcode: u32) {
        let cond = (opcode >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let op = (opcode >> 21) & 0xF; // opcode
        let s = ((opcode >> 20) & 1) == 1; // set flags
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let (op2, sh_carry) = self.decode_operand2(opcode);

        let mut write_result = true;
        let result: u32;
        let rn_val = self.regs[rn];
        match op {
            0x0 => { result = rn_val & op2; if s { self.cpsr.set_c(sh_carry); } },              // AND
            0x1 => { result = rn_val ^ op2; if s { self.cpsr.set_c(sh_carry); } },              // EOR
            0x2 => { let (r,c,v) = Self::sub_with_borrow(rn_val, op2, true); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // SUB
            0x3 => { let (r,c,v) = Self::sub_with_borrow(op2, rn_val, true); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // RSB
            0x4 => { let (r,c,v) = Self::add_with_carry(rn_val, op2, false); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // ADD
            0x5 => { let (r,c,v) = Self::add_with_carry(rn_val, op2, self.cpsr.c()); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // ADC
            0x6 => { let (r,c,v) = Self::sub_with_borrow(rn_val, op2, self.cpsr.c()); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // SBC
            0x7 => { let (r,c,v) = Self::sub_with_borrow(op2, rn_val, self.cpsr.c()); result=r; if s { self.cpsr.set_c(c); self.cpsr.set_v(v); } }, // RSC
            0x8 => { result = rn_val & op2; write_result = false; self.cpsr.set_c(sh_carry); }, // TST
            0x9 => { result = rn_val ^ op2; write_result = false; self.cpsr.set_c(sh_carry); }, // TEQ
            0xA => { let (r,c,v) = Self::sub_with_borrow(rn_val, op2, true); result=r; write_result=false; self.cpsr.set_c(c); self.cpsr.set_v(v); }, // CMP
            0xB => { let (r,c,v) = Self::add_with_carry(rn_val, op2, false); result=r; write_result=false; self.cpsr.set_c(c); self.cpsr.set_v(v); }, // CMN
            0xC => { result = rn_val | op2; if s { self.cpsr.set_c(sh_carry); } },              // ORR
            0xD => { result = op2;           if s { self.cpsr.set_c(sh_carry); } },              // MOV
            0xE => { result = rn_val & !op2; if s { self.cpsr.set_c(sh_carry); } },             // BIC
            0xF => { result = !op2;          if s { self.cpsr.set_c(sh_carry); } },             // MVN
            _ => { return; }
        }

        // N and Z set for S=1 and for test ops (write_result=false)
        if s || !write_result {
            self.cpsr.set_n((result >> 31) != 0);
            self.cpsr.set_z(result == 0);
        }

        if write_result {
            self.regs[rd] = result;
        }
    }

    fn execute_arm_multiply(&mut self, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let s = ((instr >> 20) & 1) != 0;
        let a = ((instr >> 21) & 1) != 0; // accumulate -> MLA
        let rd = ((instr >> 16) & 0xF) as usize;
        let rn = ((instr >> 12) & 0xF) as usize; // accumulate source for MLA
        let rs = ((instr >> 8) & 0xF) as usize;
        let rm = (instr & 0xF) as usize;

        let mut result = self.regs[rm].wrapping_mul(self.regs[rs]);
        if a { result = result.wrapping_add(self.regs[rn]); }
        self.regs[rd] = result;

        if s {
            self.cpsr.set_n((result >> 31) != 0);
            self.cpsr.set_z(result == 0);
            // C and V are unchanged for MUL/MLA on ARM7TDMI
        }
    }

    pub fn pc(&self) -> u32 { self.regs[15] }
    pub fn set_pc(&mut self, value: u32) { self.regs[15] = value; }

    fn reset_pipeline<B: BusAccess>(&mut self, bus: &mut B) {
        match self.state() {
            CpuState::Arm => {
                let pc = self.pc() & !3;
                let decode = bus.read32(pc.wrapping_add(4));
                let fetch = bus.read32(pc.wrapping_add(8));
                self.arm_pipe.fetch = fetch;
                self.arm_pipe.decode = decode;
                self.arm_pipe.valid = true;
            }
            CpuState::Thumb => {
                // Thumb not yet pipelined
            }
        }
    }

    fn flush_pipeline<B: BusAccess>(&mut self, bus: &mut B) {
        self.reset_pipeline(bus);
    }

    fn execute_arm_single_data_transfer<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let i = ((instr >> 25) & 1) != 0; // immediate/register offset; we support immediate only here
        let p = ((instr >> 24) & 1) != 0; // pre-index
        let u = ((instr >> 23) & 1) != 0; // add/subtract offset
        let b = ((instr >> 22) & 1) != 0; // byte/word
        let w = ((instr >> 21) & 1) != 0; // write-back (ignored unless pre-index)
        let l = ((instr >> 20) & 1) != 0; // load/store
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;
        let base = self.regs[rn];

        // Offset
        let offset = if i {
            // Register offset not implemented yet
            0
        } else {
            instr & 0xFFF
        };
        let off = if u { offset } else { 0u32.wrapping_sub(offset) };

        let address = if p { base.wrapping_add(off) } else { base };

        if l {
            if b {
                let value = (bus.read16(address & !1) >> ((address & 1) * 8)) as u8 as u32;
                self.regs[rd] = value;
            } else {
                let aligned = address & !3;
                let value = bus.read32(aligned);
                // ARM: for aligned word load, direct; for misaligned, rotate. Here keep aligned only.
                self.regs[rd] = value;
            }
        } else {
            if b {
                bus.write8(address, (self.regs[rd] & 0xFF) as u8);
            } else {
                let aligned = address & !3;
                bus.write32(aligned, self.regs[rd]);
            }
        }

        if p && w {
            self.regs[rn] = base.wrapping_add(off);
        } else if !p {
            // post-indexing: base updated after
            self.regs[rn] = base.wrapping_add(off);
        }
    }

    fn execute_arm_halfword_transfer<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
     let cond = (instr >> 28) & 0xF;
     if !self.condition_passed(cond) { return; }
     // Format: 000P U 1 W L Rn Rd 0000 (S H) imm8
     let p = ((instr >> 24) & 1) != 0;
     let u = ((instr >> 23) & 1) != 0;
     let w = ((instr >> 21) & 1) != 0;
     let l = ((instr >> 20) & 1) != 0;
     let rn = ((instr >> 16) & 0xF) as usize;
     let rd = ((instr >> 12) & 0xF) as usize;
     let s = ((instr >> 6) & 1) != 0; // In 1SH1, S is bit6
     let h = ((instr >> 5) & 1) != 0; // H is bit5
     let imm8 = (((instr >> 8) & 0xF) << 4) | (instr & 0xF);
     let base = self.regs[rn];
     let off = if u { imm8 } else { 0u32.wrapping_sub(imm8) };
     let address = if p { base.wrapping_add(off) } else { base };

     if l {
         let value = match (s, h) {
             (false, true) => { // LDRH
                 bus.read16(address & !1) as u32
             }
             (true, false) => { // LDRSB
                 let b = bus.read8(address) as i8 as i32 as u32;
                 b
             }
             (true, true) => { // LDRSH
                 let half = bus.read16(address & !1) as i16 as i32 as u32;
                 half
             }
             _ => 0,
         };
         self.regs[rd] = value;
     } else {
         // STRH only
         if h {
             bus.write16(address & !1, (self.regs[rd] & 0xFFFF) as u16);
         }
     }

     if p && w { self.regs[rn] = base.wrapping_add(off); }
     if !p { self.regs[rn] = base.wrapping_add(off); }
 }

    fn execute_arm_swp<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let byte = ((instr >> 22) & 1) != 0;
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;
        let rm = (instr & 0xF) as usize;
        let address = self.regs[rn];
        if byte {
            let old = bus.read8(address) as u32;
            bus.write8(address, (self.regs[rm] & 0xFF) as u8);
            self.regs[rd] = old;
        } else {
            let aligned = address & !3;
            let old = bus.read32(aligned);
            bus.write32(aligned, self.regs[rm]);
            self.regs[rd] = old;
        }
    }

    fn execute_arm_psr_transfer(&mut self, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let r = ((instr >> 22) & 1) != 0; // 0=CPSR, 1=SPSR (unsupported)
        let mrs = ((instr >> 21) & 1) == 0 && (((instr >> 4) & 0xFF) == 0);
        if mrs {
            if r { return; }
            let rd = ((instr >> 12) & 0xF) as usize;
            self.regs[rd] = self.cpsr.raw();
            return;
        }
        // MSR
        let immediate = ((instr >> 25) & 1) == 1;
        if r { return; }
        let field_mask = (instr >> 16) & 0xF; // f,s,x,c
        let operand = if immediate {
            let imm8 = instr & 0xFF;
            let rot = ((instr >> 8) & 0xF) * 2;
            (imm8 as u32).rotate_right(rot)
        } else {
            let rm = (instr & 0xF) as usize;
            self.regs[rm]
        };
        let mut cpsr = self.cpsr.raw();
        // Only handle f (flags) and c (control) minimally; here apply flags when bit3 (f) set
        if (field_mask & 0b1000) != 0 {
            // Derive NZCV from operand. Prefer bits31..28; if zero (immediate low form), use bits7..4 mapping.
            let nzcv = if (operand & 0xF000_0000) != 0 {
                (operand >> 28) & 0xF
            } else {
                (operand >> 4) & 0xF
            };
            // Clear flags then set from nzcv
            cpsr &= 0x0FFF_FFFF;
            cpsr |= nzcv << 28;
        }
        // Optionally update I,F,T and mode if c bit set (lowest nibble). For safety, ignore mode changes here.
        if (field_mask & 0b0001) != 0 {
            // Update only I,F,T bits (7,6,5)
            let mask = (1<<7) | (1<<6) | (1<<5);
            cpsr = (cpsr & !mask) | (operand & mask);
        }
        self.cpsr.set_raw(cpsr);
    }

    fn execute_arm_block_transfer<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let cond = (instr >> 28) & 0xF;
        if !self.condition_passed(cond) { return; }
        let p = ((instr >> 24) & 1) != 0; // pre
        let u = ((instr >> 23) & 1) != 0; // up
        let s = ((instr >> 22) & 1) != 0; // s (user mode registers)
        let w = ((instr >> 21) & 1) != 0; // writeback
        let l = ((instr >> 20) & 1) != 0; // load
        let rn = ((instr >> 16) & 0xF) as usize;
        let reg_list = instr & 0xFFFF;

        // Handle empty register list - special case
        if reg_list == 0 {
            if l {
                // LDM with empty list: load PC from address
                let addr = if p {
                    if u { self.regs[rn].wrapping_add(4) } else { self.regs[rn].wrapping_sub(4) }
                } else {
                    self.regs[rn]
                };
                let pc_val = bus.read32(addr & !3);
                self.regs[15] = pc_val;
                if w {
                    self.regs[rn] = if u {
                        self.regs[rn].wrapping_add(4)
                    } else {
                        self.regs[rn].wrapping_sub(4)
                    };
                }
            } else {
                // STM with empty list: store PC+12 to address
                let addr = if p {
                    if u { self.regs[rn].wrapping_add(4) } else { self.regs[rn].wrapping_sub(4) }
                } else {
                    self.regs[rn]
                };
                bus.write32(addr & !3, self.regs[15].wrapping_add(12));
                if w {
                    self.regs[rn] = if u {
                        self.regs[rn].wrapping_add(4)
                    } else {
                        self.regs[rn].wrapping_sub(4)
                    };
                }
            }
            return;
        }

        let base = self.regs[rn];

        // Collect registers in ascending order
        let mut regs: Vec<usize> = Vec::new();
        for r in 0..16 {
            if (reg_list >> r) & 1 == 1 { regs.push(r as usize); }
        }
        let count = regs.len() as u32;

        // Calculate start address based on addressing mode
        let start_addr = match (u, p) {
            (true, false) => base,                          // IA (Increment After)
            (true, true)  => base.wrapping_add(4),          // IB (Increment Before)
            (false, false)=> base.wrapping_sub(4 * count),  // DA (Decrement After)
            (false, true) => base.wrapping_sub(4).wrapping_sub(4 * count), // DB (Decrement Before)
        };

        // Perform transfers in ascending register order
        for (i, &reg) in regs.iter().enumerate() {
            let addr = start_addr.wrapping_add((i as u32) * 4);

            if l {
                // Load operation
                let val = bus.read32(addr & !3);
                self.regs[reg] = val;

                // Special handling for PC load
                if reg == 15 {
                    // PC load causes pipeline flush
                    self.flush_pipeline(bus);
            }
        } else {
                // Store operation
                let val = if reg == 15 {
                    // Store PC+12 for return address
                    self.regs[15].wrapping_add(12)
                } else {
                    self.regs[reg]
                };
                bus.write32(addr & !3, val);
            }
        }

        // Update base register if writeback is enabled
        if w {
            let new_base = match (u, p) {
                (true, false) => base.wrapping_add(4 * count),      // IA: base + count*4
                (true, true)  => base.wrapping_add(4).wrapping_add(4 * count), // IB: base + 4 + count*4
                (false, false)=> base.wrapping_sub(4 * count),      // DA: base - count*4
                (false, true) => base.wrapping_sub(4).wrapping_sub(4 * count), // DB: base - 4 - count*4
            };
            self.regs[rn] = new_base;
        }

        // Note: S bit (user mode registers) not implemented yet
        let _ = s;
    }

    // THUMB instruction implementations

    fn execute_thumb_move_shifted_register(&mut self, instr: u32) {
        let op = (instr >> 11) & 0x3; // 00=LSL, 01=LSR, 10=ASR, 11=ADD/SUB
        let offset5 = (instr >> 6) & 0x1F;
        let rs = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        match op {
            0 => { // LSL
                let (result, carry) = Self::lsl_with_carry(self.regs[rs as usize], offset5, self.cpsr.c(), true);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            1 => { // LSR
                let (result, carry) = Self::lsr_with_carry(self.regs[rs as usize], offset5, self.cpsr.c(), true);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            2 => { // ASR
                let (result, carry) = Self::asr_with_carry(self.regs[rs as usize], offset5, self.cpsr.c(), true);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            3 => { // ADD/SUB
                let op2 = (instr >> 9) & 0x1; // 0=ADD, 1=SUB
                let rm = (instr >> 6) & 0x7;
                let rs_val = self.regs[rs as usize];
                let rm_val = self.regs[rm as usize];

                if op2 == 0 { // ADD
                    let (result, carry, overflow) = Self::add_with_carry(rs_val, rm_val, false);
                    self.regs[rd as usize] = result;
                    self.cpsr.set_n((result >> 31) != 0);
                    self.cpsr.set_z(result == 0);
                    self.cpsr.set_c(carry);
                    self.cpsr.set_v(overflow);
                } else { // SUB
                    let (result, carry, overflow) = Self::sub_with_borrow(rs_val, rm_val, true);
                    self.regs[rd as usize] = result;
                    self.cpsr.set_n((result >> 31) != 0);
                    self.cpsr.set_z(result == 0);
                    self.cpsr.set_c(carry);
                    self.cpsr.set_v(overflow);
                }
            }
            _ => {}
        }
    }

    fn execute_thumb_add_subtract(&mut self, instr: u32) {
        let op = (instr >> 9) & 0x1; // 0=ADD, 1=SUB
        let rn = (instr >> 6) & 0x7;
        let rs = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rs_val = self.regs[rs as usize];
        let rn_val = self.regs[rn as usize];

        if op == 0 { // ADD
            let (result, carry, overflow) = Self::add_with_carry(rs_val, rn_val, false);
            self.regs[rd as usize] = result;
            self.cpsr.set_n((result >> 31) != 0);
            self.cpsr.set_z(result == 0);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
        } else { // SUB
            let (result, carry, overflow) = Self::sub_with_borrow(rs_val, rn_val, true);
            self.regs[rd as usize] = result;
            self.cpsr.set_n((result >> 31) != 0);
            self.cpsr.set_z(result == 0);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
        }
    }

    fn execute_thumb_move_compare_add_subtract_immediate(&mut self, instr: u32) {
        let op = (instr >> 10) & 0x3; // 00=MOV, 01=CMP, 10=ADD, 11=SUB
        let rd = (instr >> 8) & 0x7;
        let imm8 = instr & 0xFF;

        match op {
            0 => { // MOV
                self.regs[rd as usize] = imm8;
                self.cpsr.set_n((imm8 >> 31) != 0);
                self.cpsr.set_z(imm8 == 0);
            }
            1 => { // CMP
                let rd_val = self.regs[rd as usize];
                let (result, carry, overflow) = Self::sub_with_borrow(rd_val, imm8, true);
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            2 => { // ADD
                let rd_val = self.regs[rd as usize];
                let (result, carry, overflow) = Self::add_with_carry(rd_val, imm8, false);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            3 => { // SUB
                let rd_val = self.regs[rd as usize];
                let (result, carry, overflow) = Self::sub_with_borrow(rd_val, imm8, true);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            _ => {}
        }
    }

    fn execute_thumb_alu_operations(&mut self, instr: u32) {
        let op = (instr >> 6) & 0xF;
        let rs = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rs_val = self.regs[rs as usize];
        let rd_val = self.regs[rd as usize];

        match op {
            0 => { // AND
                let result = rd_val & rs_val;
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            1 => { // EOR
                let result = rd_val ^ rs_val;
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            2 => { // LSL
                let shift = rs_val & 0xFF;
                let (result, carry) = Self::lsl_with_carry(rd_val, shift, self.cpsr.c(), false);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            3 => { // LSR
                let shift = rs_val & 0xFF;
                let (result, carry) = Self::lsr_with_carry(rd_val, shift, self.cpsr.c(), false);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            4 => { // ASR
                let shift = rs_val & 0xFF;
                let (result, carry) = Self::asr_with_carry(rd_val, shift, self.cpsr.c(), false);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            5 => { // ADC
                let (result, carry, overflow) = Self::add_with_carry(rd_val, rs_val, self.cpsr.c());
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            6 => { // SBC
                let (result, carry, overflow) = Self::sub_with_borrow(rd_val, rs_val, self.cpsr.c());
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            7 => { // ROR
                let shift = rs_val & 0xFF;
                let (result, carry) = Self::ror_with_carry(rd_val, shift, self.cpsr.c(), false);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
            }
            8 => { // TST
                let result = rd_val & rs_val;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            9 => { // NEG
                let (result, carry, overflow) = Self::sub_with_borrow(0, rs_val, true);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            10 => { // CMP
                let (result, carry, overflow) = Self::sub_with_borrow(rd_val, rs_val, true);
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            11 => { // CMN
                let (result, carry, overflow) = Self::add_with_carry(rd_val, rs_val, false);
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            12 => { // ORR
                let result = rd_val | rs_val;
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            13 => { // MUL
                let result = rd_val.wrapping_mul(rs_val);
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                // C and V are undefined for MUL in THUMB
            }
            14 => { // BIC
                let result = rd_val & !rs_val;
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            15 => { // MVN
                let result = !rs_val;
                self.regs[rd as usize] = result;
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
            }
            _ => {}
        }
    }

    fn execute_thumb_hi_register_operations_branch_exchange(&mut self, instr: u32) {
        let op = (instr >> 8) & 0x3;
        let h1 = (instr >> 7) & 0x1;
        let h2 = (instr >> 6) & 0x1;
        let rs = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rd_idx = if h1 == 1 { rd + 8 } else { rd } as usize;
        let rs_idx = if h2 == 1 { rs + 8 } else { rs } as usize;

        match op {
            0 => { // ADD
                let rd_val = self.regs[rd_idx];
                let rs_val = self.regs[rs_idx];
                let (result, carry, overflow) = Self::add_with_carry(rd_val, rs_val, false);
                self.regs[rd_idx] = result;
                if rd_idx < 8 { // Only set flags for low registers
                    self.cpsr.set_n((result >> 31) != 0);
                    self.cpsr.set_z(result == 0);
                    self.cpsr.set_c(carry);
                    self.cpsr.set_v(overflow);
                }
            }
            1 => { // CMP
                let rd_val = self.regs[rd_idx];
                let rs_val = self.regs[rs_idx];
                let (result, carry, overflow) = Self::sub_with_borrow(rd_val, rs_val, true);
                self.cpsr.set_n((result >> 31) != 0);
                self.cpsr.set_z(result == 0);
                self.cpsr.set_c(carry);
                self.cpsr.set_v(overflow);
            }
            2 => { // MOV
                let rs_val = self.regs[rs_idx];
                self.regs[rd_idx] = rs_val;
                if rd_idx < 8 { // Only set flags for low registers
                    self.cpsr.set_n((rs_val >> 31) != 0);
                    self.cpsr.set_z(rs_val == 0);
                }
            }
            3 => { // BX
                let rs_val = self.regs[rs_idx];
                let new_pc = rs_val & !1; // Clear bit 0
                let new_state = if (rs_val & 1) != 0 { CpuState::Thumb } else { CpuState::Arm };

                self.regs[15] = new_pc;
                self.cpsr.set_state(new_state);
                // Pipeline flush will be handled by the step function
            }
            _ => {}
        }
    }

    fn execute_thumb_pc_relative_load<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let rd = (instr >> 8) & 0x7;
        let imm8 = instr & 0xFF;

        let pc = (self.regs[15] & !3) + 4; // PC + 4, word aligned
        let address = pc + (imm8 << 2);

        let value = bus.read32(address & !3);
        self.regs[rd as usize] = value;
    }

    fn execute_thumb_load_store_register_offset<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let op = (instr >> 10) & 0x3; // 00=STR, 01=STRH, 10=STRB, 11=LDRSB
        let ro = (instr >> 6) & 0x7;
        let rb = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rb_val = self.regs[rb as usize];
        let ro_val = self.regs[ro as usize];
        let address = rb_val + ro_val;

        match op {
            0 => { // STR
                let value = self.regs[rd as usize];
                bus.write32(address & !3, value);
            }
            1 => { // STRH
                let value = self.regs[rd as usize] as u16;
                bus.write16(address & !1, value);
            }
            2 => { // STRB
                let value = self.regs[rd as usize] as u8;
                bus.write8(address, value);
            }
            3 => { // LDRSB
                let value = bus.read8(address) as i8 as i32 as u32;
                self.regs[rd as usize] = value;
            }
            _ => {}
        }
    }

    fn execute_thumb_load_store_sign_extended<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let op = (instr >> 10) & 0x3; // 00=LDRH, 01=LDSB, 10=LDRB, 11=LDSH
        let ro = (instr >> 6) & 0x7;
        let rb = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rb_val = self.regs[rb as usize];
        let ro_val = self.regs[ro as usize];
        let address = rb_val + ro_val;

        match op {
            0 => { // LDRH
                let value = bus.read16(address & !1) as u32;
                self.regs[rd as usize] = value;
            }
            1 => { // LDSB (LDRSB)
                let value = bus.read8(address) as i8 as i32 as u32;
                self.regs[rd as usize] = value;
            }
            2 => { // LDRB
                let value = bus.read8(address) as u32;
                self.regs[rd as usize] = value;
            }
            3 => { // LDSH (LDRSH)
                let value = bus.read16(address & !1) as i16 as i32 as u32;
                self.regs[rd as usize] = value;
            }
            _ => {}
        }
    }

    fn execute_thumb_load_store_immediate_offset<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let op = (instr >> 11) & 0x1; // 0=STR, 1=LDR
        let imm5 = (instr >> 6) & 0x1F;
        let rb = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rb_val = self.regs[rb as usize];
        let address = rb_val + (imm5 << 2);

        if op == 0 { // STR
            let value = self.regs[rd as usize];
            bus.write32(address & !3, value);
        } else { // LDR
            let value = bus.read32(address & !3);
            self.regs[rd as usize] = value;
        }
    }

    fn execute_thumb_load_store_halfword<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let op = (instr >> 11) & 0x1; // 0=STRH, 1=LDRH
        let imm5 = (instr >> 6) & 0x1F;
        let rb = (instr >> 3) & 0x7;
        let rd = instr & 0x7;

        let rb_val = self.regs[rb as usize];
        let address = rb_val + (imm5 << 1);

        if op == 0 { // STRH
            let value = self.regs[rd as usize] as u16;
            bus.write16(address & !1, value);
        } else { // LDRH
            let value = bus.read16(address & !1) as u32;
            self.regs[rd as usize] = value;
        }
    }

    fn execute_thumb_sp_relative_load_store<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let op = (instr >> 11) & 0x1; // 0=STR, 1=LDR
        let rd = (instr >> 8) & 0x7;
        let imm8 = instr & 0xFF;

        let sp = self.regs[13];
        let address = sp + (imm8 << 2);

        if op == 0 { // STR
            let value = self.regs[rd as usize];
            bus.write32(address & !3, value);
        } else { // LDR
            let value = bus.read32(address & !3);
            self.regs[rd as usize] = value;
        }
    }

    fn execute_thumb_load_address(&mut self, instr: u32) {
        let sp = (instr >> 11) & 0x1; // 0=ADD to PC, 1=ADD to SP
        let rd = (instr >> 8) & 0x7;
        let imm8 = instr & 0xFF;

        if sp == 0 { // ADD to PC
            let pc = (self.regs[15] & !3) + 4; // PC + 4, word aligned
            let address = pc + (imm8 << 2);
            self.regs[rd as usize] = address;
        } else { // ADD to SP
            let sp_val = self.regs[13];
            let address = sp_val + (imm8 << 2);
            self.regs[rd as usize] = address;
        }
    }

    fn execute_thumb_add_offset_to_sp(&mut self, instr: u32) {
        let s = (instr >> 7) & 0x1; // 0=ADD, 1=SUB
        let imm7 = instr & 0x7F;

        let sp = self.regs[13];
        let offset = imm7 << 2;

        if s == 0 { // ADD
            self.regs[13] = sp + offset;
        } else { // SUB
            self.regs[13] = sp - offset;
        }
    }

    fn execute_thumb_push_pop_registers<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let l = (instr >> 11) & 0x1; // 0=PUSH, 1=POP
        let r = (instr >> 8) & 0x1; // 0=no PC/LR, 1=include PC/LR
        let reg_list = instr & 0xFF;

        let sp = self.regs[13];

        if l == 0 { // PUSH
            let mut count = 0;
            for i in 0..8 {
                if (reg_list >> i) & 1 == 1 {
                    count += 1;
                }
            }
            if r == 1 { count += 1; } // LR

            let start_addr = sp - (count << 2);
            let mut addr = start_addr;

            for i in 0..8 {
                if (reg_list >> i) & 1 == 1 {
                    bus.write32(addr & !3, self.regs[i]);
                    addr += 4;
                }
            }
            if r == 1 { // LR
                bus.write32(addr & !3, self.regs[14]);
            }

            self.regs[13] = start_addr;
        } else { // POP
            let mut addr = sp;

            for i in 0..8 {
                if (reg_list >> i) & 1 == 1 {
                    let value = bus.read32(addr & !3);
                    self.regs[i] = value;
                    addr += 4;
                }
            }
            if r == 1 { // PC
                let value = bus.read32(addr & !3);
                self.regs[15] = value;
                // Pipeline flush will be handled by the step function
            }

            self.regs[13] = addr;
        }
    }

    fn execute_thumb_multiple_load_store<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let l = (instr >> 11) & 0x1; // 0=STMIA, 1=LDMIA
        let rb = (instr >> 8) & 0x7;
        let reg_list = instr & 0xFF;

        let rb_val = self.regs[rb as usize];
        let mut addr = rb_val;

        if l == 0 { // STMIA
            for i in 0..8 {
                if (reg_list >> i) & 1 == 1 {
                    bus.write32(addr & !3, self.regs[i]);
                    addr += 4;
                }
            }
            self.regs[rb as usize] = addr; // Writeback
        } else { // LDMIA
            for i in 0..8 {
                if (reg_list >> i) & 1 == 1 {
                    let value = bus.read32(addr & !3);
                    self.regs[i] = value;
                    addr += 4;
                }
            }
            self.regs[rb as usize] = addr; // Writeback
        }
    }

    fn execute_thumb_conditional_branch<B: BusAccess>(&mut self, _bus: &mut B, instr: u32) {
        let cond = (instr >> 8) & 0xF;
        let imm8 = instr & 0xFF;

        if self.condition_passed(cond) {
            let offset = ((imm8 as i8) as i32) << 1;
            let pc = self.regs[15]; // PC is already advanced by 2, so this is PC+2
            self.regs[15] = (pc as i32 + offset) as u32;
            // Pipeline flush will be handled by the step function
        }
    }

    fn execute_thumb_software_interrupt(&mut self, instr: u32) {
        // SWI - not implemented yet
        let _ = instr;
    }

    fn execute_thumb_unconditional_branch<B: BusAccess>(&mut self, _bus: &mut B, instr: u32) {
        let imm11 = instr & 0x7FF;
        let offset = ((imm11 as i16) << 5) >> 4; // Sign extend 11-bit to 16-bit, then to 32-bit
        let pc = self.regs[15] - 2; // PC is already advanced by 2
        self.regs[15] = (pc as i32 + offset as i32) as u32;
        // Pipeline flush will be handled by the step function
    }

    fn execute_thumb_long_branch_with_link<B: BusAccess>(&mut self, _bus: &mut B, instr: u32) {
        let h = (instr >> 11) & 0x1;
        let imm11 = instr & 0x7FF;

        if h == 0 { // First instruction
            let offset = ((imm11 as i16) << 5) >> 4; // Sign extend
            let pc = self.regs[15] - 2;
            self.regs[14] = pc.wrapping_add(offset as u32);
        } else { // Second instruction
            let offset = ((imm11 as i16) << 5) >> 4; // Sign extend
            let lr = self.regs[14];
            let pc = self.regs[15] - 2;
            let new_pc = lr.wrapping_add(offset as u32);

            self.regs[14] = pc | 1; // Set bit 0 to indicate THUMB return
            self.regs[15] = new_pc;
            // Pipeline flush will be handled by the step function
        }
    }

    fn execute_thumb_instruction<B: BusAccess>(&mut self, bus: &mut B, instr: u32) {
        let opcode = (instr >> 11) & 0x1F;

        match opcode {
            0x00..=0x07 => {
                // Format 1: Move Shifted Register
                self.execute_thumb_move_shifted_register(instr);
            }
            0x08..=0x0F => {
                // Format 2: Add/Subtract
                self.execute_thumb_add_subtract(instr);
            }
            0x10..=0x11 => {
                // Format 3: Move/Compare/Add/Subtract Immediate
                self.execute_thumb_move_compare_add_subtract_immediate(instr);
            }
            0x12..=0x13 => {
                // Format 4: ALU Operations
                self.execute_thumb_alu_operations(instr);
            }
            0x14..=0x15 => {
                // Format 5: Hi Register Operations/Branch Exchange
                self.execute_thumb_hi_register_operations_branch_exchange(instr);
            }
            0x16..=0x17 => {
                // Format 6: PC-Relative Load
                self.execute_thumb_pc_relative_load(bus, instr);
            }
            0x18..=0x19 => {
                // Format 7: Load/Store with Register Offset
                self.execute_thumb_load_store_register_offset(bus, instr);
            }
            0x1B => {
                // Format 8: Load/Store Sign-Extended Byte/Halfword
                self.execute_thumb_load_store_sign_extended(bus, instr);
            }
            0x1C..=0x1D => {
                // Format 9: Load/Store with Immediate Offset
                self.execute_thumb_load_store_immediate_offset(bus, instr);
            }
            0x1E..=0x1F => {
                // Format 10: Load/Store Halfword
                self.execute_thumb_load_store_halfword(bus, instr);
            }
            0x20..=0x21 => {
                // Format 11: SP-Relative Load/Store
                self.execute_thumb_sp_relative_load_store(bus, instr);
            }
            0x22..=0x23 => {
                // Format 12: Load Address
                self.execute_thumb_load_address(instr);
            }
            0x24..=0x25 => {
                // Format 13: Add Offset to Stack Pointer
                self.execute_thumb_add_offset_to_sp(instr);
            }
            0x26..=0x27 => {
                // Format 14: Push/Pop Registers
                self.execute_thumb_push_pop_registers(bus, instr);
            }
            0x28..=0x2F => {
                // Format 15: Multiple Load/Store
                self.execute_thumb_multiple_load_store(bus, instr);
            }
            0x1A => {
                // Format 16: Conditional Branch
                self.execute_thumb_conditional_branch(bus, instr);
            }
            0x38..=0x3F => {
                // Format 17: Software Interrupt
                self.execute_thumb_software_interrupt(instr);
            }
            0x40..=0x47 => {
                // Format 18: Unconditional Branch
                self.execute_thumb_unconditional_branch(bus, instr);
            }
            0x48..=0x4F => {
                // Format 19: Long Branch with Link
                self.execute_thumb_long_branch_with_link(bus, instr);
            }
            _ => {
                // Unknown instruction - should not happen with 5-bit opcode
            }
        }
    }

    pub fn step<B: BusAccess>(&mut self, bus: &mut B) {
        match self.state() {
            CpuState::Arm => {
                if !self.arm_pipe.valid { self.reset_pipeline(bus); }
                let instr = self.arm_pipe.decode;
                let next_pc = (self.pc() & !3).wrapping_add(4);
                let new_decode = self.arm_pipe.fetch;
                let new_fetch = bus.read32(next_pc.wrapping_add(8));
                self.arm_pipe.decode = new_decode;
                self.arm_pipe.fetch = new_fetch;
                self.regs[15] = next_pc;

                let top2 = (instr >> 26) & 0x3;
                let top3 = (instr >> 25) & 0x7;
                if ((instr >> 22) & 0x3F) == 0 && ((instr >> 4) & 0xF) == 0b1001 {
                    let before_pc = self.pc();
                    self.execute_arm_multiply(instr);
                    if self.pc() != before_pc { self.flush_pipeline(bus); }
                } else if (((instr >> 23) & 0x1F) == 0b00010) && (((instr >> 21) & 0x3) == 0) && (((instr >> 4) & 0xF) == 0b1001) {
                    self.execute_arm_swp(bus, instr);
                } else if (instr & 0x0FBF0FFF) == 0x010F0000
                    || (instr & 0x0DBFF000) == 0x0320F000
                    || (instr & 0x0FBFF000) == 0x0120F000
                {
                    self.execute_arm_psr_transfer(instr);
                } else if (instr & 0x0E400090) == 0x00400090 && (((instr >> 4) & 0xF) != 0b1001) {
                    self.execute_arm_halfword_transfer(bus, instr);
                } else if top3 == 0b100 {
                    self.execute_arm_block_transfer(bus, instr);
                } else if top2 == 0 {
                    let before_pc = self.pc();
                    self.execute_arm_data_processing(instr);
                    if self.pc() != before_pc { self.flush_pipeline(bus); }
                } else if top3 == 0b101 {
                    let cond = (instr >> 28) & 0xF;
                    if self.condition_passed(cond) {
                        let l = ((instr >> 24) & 1) != 0;
                        let imm24 = (instr & 0x00FF_FFFF) as u32;
                        let offset = (((imm24 as i32) << 8) >> 6) as u32;
                        let base = self.pc().wrapping_add(8);
                        if l { self.regs[14] = base.wrapping_sub(4); }
                        self.regs[15] = base.wrapping_add(offset);
                        self.flush_pipeline(bus);
                    }
                } else if top3 == 0b010 {
                    self.execute_arm_single_data_transfer(bus, instr);
                }
            }
            CpuState::Thumb => {
                let pc = self.pc();
                let instr16 = bus.read16(pc & !1) as u32;
                self.regs[15] = pc.wrapping_add(2);
                self.execute_thumb_instruction(bus, instr16);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBus { mem: Vec<u8> }
    impl MockBus {
        fn new(size: usize) -> Self { Self { mem: vec![0; size] } }

        fn ensure_size(&mut self, addr: u32, size: usize) {
            let addr = addr as usize;
            if addr + size > self.mem.len() {
                self.mem.resize(addr + size, 0);
            }
        }
    }
    impl BusAccess for MockBus {
        fn read32(&mut self, addr: u32) -> u32 {
            self.ensure_size(addr, 4);
            let a = addr as usize;
            (self.mem[a] as u32)
                | ((self.mem[a + 1] as u32) << 8)
                | ((self.mem[a + 2] as u32) << 16)
                | ((self.mem[a + 3] as u32) << 24)
        }
        fn read16(&mut self, addr: u32) -> u16 {
            self.ensure_size(addr, 2);
            let a = addr as usize;
            (self.mem[a] as u16) | ((self.mem[a + 1] as u16) << 8)
        }
        fn read8(&mut self, addr: u32) -> u8 {
            self.ensure_size(addr, 1);
            self.mem[addr as usize]
        }
        fn write32(&mut self, addr: u32, value: u32) {
            self.ensure_size(addr, 4);
            let a = addr as usize;
            self.mem[a] = (value & 0xFF) as u8;
            self.mem[a + 1] = ((value >> 8) & 0xFF) as u8;
            self.mem[a + 2] = ((value >> 16) & 0xFF) as u8;
            self.mem[a + 3] = ((value >> 24) & 0xFF) as u8;
        }
        fn write16(&mut self, addr: u32, value: u16) {
            self.ensure_size(addr, 2);
            let a = addr as usize;
            self.mem[a] = (value & 0xFF) as u8;
            self.mem[a + 1] = ((value >> 8) & 0xFF) as u8;
        }
        fn write8(&mut self, addr: u32, value: u8) {
            self.ensure_size(addr, 1);
            self.mem[addr as usize] = value;
        }
    }

    fn write32_le(mem: &mut Vec<u8>, addr: usize, value: u32) {
        if addr + 4 > mem.len() {
            mem.resize(addr + 4, 0);
        }
        mem[addr] = (value & 0xFF) as u8;
        mem[addr + 1] = ((value >> 8) & 0xFF) as u8;
        mem[addr + 2] = ((value >> 16) & 0xFF) as u8;
        mem[addr + 3] = ((value >> 24) & 0xFF) as u8;
    }

    #[test]
    fn cpu_step_arm_pc_advance_and_execute_dp() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(64);
        // Encode: MOV r1, #1 (ARM): cond=E, I=1, op=0xD, S=1, rn=0, rd=1, imm=1
        let mov = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (1 << 12) | 0x01;
        // With pipeline, first executed instruction is at PC+4 on first step
        write32_le(&mut bus.mem, 4, mov);
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(1), 1);
        assert_eq!(cpu.pc(), 4);
        assert!(cpu.cpsr().z() == false);
    }

    #[test]
    fn cpu_step_thumb_fetch_only() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);
        bus.mem[0] = 0x00;
        bus.mem[1] = 0xB5; // push {lr} example encoding upper byte pattern, unused
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc(), 2);
    }

    #[test]
    fn thumb_mov_immediate() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // MOV r1, #0x42 (Format 3: Move/Compare/Add/Subtract Immediate)
        // op=00 (MOV), rd=1, imm8=0x42
        let mov_instr = (0x10 << 11) | (1 << 8) | 0x42;
        bus.write16(0, mov_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(1), 0x42);
        assert!(!cpu.cpsr().n());
        assert!(!cpu.cpsr().z());
    }

    #[test]
    fn thumb_add_immediate() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // Set up initial value
        cpu.write_reg(1, 0x10);

        // ADD r1, r1, #0x20 (Format 3: Move/Compare/Add/Subtract Immediate)
        // op=10 (ADD), rd=1, imm8=0x20
        let add_instr = (0x10 << 11) | (2 << 10) | (1 << 8) | 0x20;
        bus.write16(0, add_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(1), 0x30);
        assert!(!cpu.cpsr().n());
        assert!(!cpu.cpsr().z());
    }

    #[test]
    fn thumb_lsl_immediate() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // Set up initial value
        cpu.write_reg(1, 0x1);

        // LSL r2, r1, #2 (Format 1: Move Shifted Register)
        // op=00 (LSL), offset5=2, rs=1, rd=2
        let lsl_instr = (0x00 << 11) | (2 << 6) | (1 << 3) | 2;
        bus.write16(0, lsl_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2), 0x4);
        assert!(!cpu.cpsr().n());
        assert!(!cpu.cpsr().z());
        assert!(!cpu.cpsr().c());
    }

    #[test]
    fn thumb_ldr_immediate_offset() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // Set up base address and memory
        cpu.write_reg(0, 0x100);
        bus.write32(0x108, 0xDEADBEEF); // offset 8 (imm5=2, so 2*4=8)

        // LDR r1, [r0, #8] (Format 9: Load/Store with Immediate Offset)
        // op=1 (LDR), imm5=2, rb=0, rd=1
        let ldr_instr = (0x1D << 11) | (2 << 6) | (0 << 3) | 1;
        bus.write16(0, ldr_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(1), 0xDEADBEEF);
    }

    #[test]
    fn thumb_bx_branch_exchange() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // Set up return address with ARM state bit
        cpu.write_reg(0, 0x1000); // ARM state (bit 0 = 0)

        // BX r0 (Format 5: Hi Register Operations/Branch Exchange)
        // op=3 (BX), h1=0, h2=0, rs=0, rd=0
        let bx_instr = (0x15 << 11) | (3 << 8) | (0 << 7) | (0 << 6) | (0 << 3) | 0;
        bus.write16(0, bx_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc(), 0x1000);
        assert_eq!(cpu.state(), CpuState::Arm);
    }

    #[test]
    fn thumb_conditional_branch() {
        let mut cpu = Cpu::new();
        cpu.cpsr_mut().set_state(CpuState::Thumb);
        let mut bus = MockBus::new(64);

        // Set up condition (Z=1 for EQ)
        cpu.cpsr_mut().set_z(true);

        // BEQ #4 (Format 16: Conditional Branch)
        // cond=0000 (EQ), imm8=4
        let beq_instr = 0xD004;
        bus.write16(0, beq_instr as u16);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        // PC should be 0 + 2 (advance) + 4*2 (offset) = 10
        assert_eq!(cpu.pc(), 10);
    }

    #[test]
    fn cpsr_mode_bits_roundtrip() {
        let mut cpsr = Cpsr::new();
        cpsr.set_mode(CpuMode::Irq);
        assert_eq!(cpsr.mode(), CpuMode::Irq);
        cpsr.set_mode(CpuMode::Supervisor);
        assert_eq!(cpsr.mode(), CpuMode::Supervisor);
        cpsr.set_mode(CpuMode::System);
        assert_eq!(cpsr.mode(), CpuMode::System);
    }

    #[test]
    fn cpsr_state_bits_roundtrip() {
        let mut cpsr = Cpsr::new();
        assert_eq!(cpsr.state(), CpuState::Arm);
        cpsr.set_state(CpuState::Thumb);
        assert_eq!(cpsr.state(), CpuState::Thumb);
        cpsr.set_state(CpuState::Arm);
        assert_eq!(cpsr.state(), CpuState::Arm);
    }

    #[test]
    fn banked_sp_lr_switching() {
        let mut cpu = Cpu::new();
        cpu.write_reg(13, 0xAAAABBBB);
        cpu.write_reg(14, 0xCCCCDDDD);

        cpu.set_mode(CpuMode::Irq);
        cpu.write_reg(13, 0x11112222);
        cpu.write_reg(14, 0x33334444);

        cpu.set_mode(CpuMode::Supervisor);
        cpu.write_reg(13, 0x55556666);
        cpu.write_reg(14, 0x77778888);

        cpu.set_mode(CpuMode::System);
        assert_eq!(cpu.read_reg(13), 0xAAAABBBB);
        assert_eq!(cpu.read_reg(14), 0xCCCCDDDD);

        cpu.set_mode(CpuMode::Irq);
        assert_eq!(cpu.read_reg(13), 0x11112222);
        assert_eq!(cpu.read_reg(14), 0x33334444);

        cpu.set_mode(CpuMode::Supervisor);
        assert_eq!(cpu.read_reg(13), 0x55556666);
        assert_eq!(cpu.read_reg(14), 0x77778888);
    }

    #[test]
    fn fiq_r8_r12_banked() {
        let mut cpu = Cpu::new();
        for i in 8..=12 { cpu.write_reg(i, 0x1000_0000 + i as u32); }

        cpu.set_mode(CpuMode::Fiq);
        for i in 8..=12 { cpu.write_reg(i, 0x2000_0000 + i as u32); }

        cpu.set_mode(CpuMode::System);
        for i in 8..=12 { assert_eq!(cpu.read_reg(i), 0x1000_0000 + i as u32); }

        cpu.set_mode(CpuMode::Fiq);
        for i in 8..=12 { assert_eq!(cpu.read_reg(i), 0x2000_0000 + i as u32); }
    }

    #[test]
    fn spsr_per_mode_storage() {
        let mut cpu = Cpu::new();
        cpu.set_mode(CpuMode::Irq);
        cpu.set_spsr(0xDEAD_BEEF);
        assert_eq!(cpu.spsr(), Some(0xDEAD_BEEF));
        cpu.set_mode(CpuMode::System);
        assert_eq!(cpu.spsr(), None);
        cpu.set_mode(CpuMode::Irq);
        assert_eq!(cpu.spsr(), Some(0xDEAD_BEEF));
    }

    #[test]
    fn shifter_lsl_immediate_edges() {
        // amount 0 keeps carry
        let (r, c) = Cpu::lsl_with_carry(0x12345678, 0, true, true);
        assert_eq!(r, 0x12345678);
        assert!(c);
        let (r, c) = Cpu::lsl_with_carry(0x8000_0001, 1, false, true);
        assert_eq!(r, 0x0000_0002);
        assert!(c); // bit 31 shifted out
        let (r, c) = Cpu::lsl_with_carry(0x0000_0001, 32, false, true);
        assert_eq!(r, 0);
        assert!(c);
        let (r, c) = Cpu::lsl_with_carry(0x0000_0001, 33, true, true);
        assert_eq!(r, 0);
        assert!(!c);
    }

    #[test]
    fn shifter_lsr_immediate_edges() {
        let (r, c) = Cpu::lsr_with_carry(0x8000_0000, 0, false, true);
        assert_eq!(r, 0);
        assert!(c);
        let (r, c) = Cpu::lsr_with_carry(0x0000_0003, 1, false, true);
        assert_eq!(r, 0x0000_0001);
        assert!(c);
        let (r, c) = Cpu::lsr_with_carry(0x8000_0000, 32, false, true);
        assert_eq!(r, 0);
        assert!(c);
        let (r, c) = Cpu::lsr_with_carry(0x8000_0000, 40, true, true);
        assert_eq!(r, 0);
        assert!(!c);
    }

    #[test]
    fn shifter_asr_immediate_edges() {
        let (r, c) = Cpu::asr_with_carry(0x8000_0000, 0, false, true);
        assert_eq!(r, 0xFFFF_FFFF);
        assert!(c);
        let (r, c2) = Cpu::asr_with_carry(0x7FFF_FFFF, 0, true, true);
        assert_eq!(r, 0x0000_0000);
        assert!(!c2);
        let (r, _) = Cpu::asr_with_carry(0xF000_0001u32, 4, false, true);
        assert_eq!(r, 0xFF00_0000);
        assert!(((0xF000_0001u32 >> 3) & 1) == 0);
        let (r, c3) = Cpu::asr_with_carry(0x8000_0000, 40, false, true);
        assert_eq!(r, 0xFFFF_FFFF);
        assert!(c3);
    }

    #[test]
    fn shifter_ror_immediate_and_rrx() {
        let (r, c) = Cpu::ror_with_carry(0x0000_0001, 0, true, true);
        assert_eq!(r, 0x8000_0000);
        assert!(c);
        let (r, _) = Cpu::ror_with_carry(0x8000_0000, 1, false, true);
        assert_eq!(r, 0x4000_0000);
        assert!((r >> 31) == 0);
        let (r, _) = Cpu::ror_with_carry(0x1234_5678, 28, false, true);
        assert_eq!(r, 0x2345_6781);
        assert!((r >> 31) == 0);
    }

    #[test]
    fn shifter_register_amount_behaviors() {
        // amount 0 keeps carry
        let (r, c) = Cpu::lsl_with_carry(0x1, 0, true, false);
        assert_eq!(r, 0x1);
        assert!(c);
        // amounts >=32
        let (r, c2) = Cpu::lsl_with_carry(0x1, 32, false, false);
        assert_eq!(r, 0);
        assert!(c2);
        let (r, c3) = Cpu::lsl_with_carry(0x2, 40, false, false);
        assert_eq!(r, 0);
        assert!(!c3);

        let (r, c4) = Cpu::lsr_with_carry(0x8000_0000, 32, false, false);
        assert_eq!(r, 0);
        assert!(c4);
        let (r, c5) = Cpu::lsr_with_carry(0x8000_0000, 40, false, false);
        assert_eq!(r, 0);
        assert!(!c5);

        // ASR large keeps sign
        let (r, c6) = Cpu::asr_with_carry(0x8000_0000, 32, false, false);
        assert_eq!(r, 0xFFFF_FFFF);
        assert!(c6);
        let (r, c7) = Cpu::asr_with_carry(0x7FFF_FFFF, 40, true, false);
        assert_eq!(r, 0x0000_0000);
        assert!(!c7);

        // ROR with amount%32 == 0 and amount!=0 sets carry to bit31
        let (r, c8) = Cpu::ror_with_carry(0x8000_0000, 32, false, false);
        assert_eq!(r, 0x8000_0000);
        assert!(c8);
        let (r, c9) = Cpu::ror_with_carry(0x0000_0001, 64, true, false);
        assert_eq!(r, 0x0000_0001);
        assert!(!c9);
    }

    #[test]
    fn dp_and_orr_eor_mov_bic_mvn_immediate() {
        let mut cpu = Cpu::new();
        cpu.write_reg(0, 0xF0F0_0F0F);
        // AND r1, r0, #0xFF rotated right by 8 -> 0xFF000000
        let opcode_and = (0xE << 28) | (1 << 25) | (0x0 << 21) | (1 << 20) | (0 << 16) | (1 << 12) | (4 << 8) | 0xFF;
        cpu.execute_arm_data_processing(opcode_and);
        assert_eq!(cpu.read_reg(1), 0xF000_0000);
        assert!(cpu.cpsr().n());
        assert!(!cpu.cpsr().z());

        // ORR r2, r0, #1
        let opcode_orr = (0xE << 28) | (1 << 25) | (0xC << 21) | (1 << 20) | (0 << 16) | (2 << 12) | 0x01;
        cpu.execute_arm_data_processing(opcode_orr);
        assert_eq!(cpu.read_reg(2), 0xF0F0_0F0F | 1);

        // EOR r3, r0, #0xFF -> flags
        let opcode_eor = (0xE << 28) | (1 << 25) | (0x1 << 21) | (1 << 20) | (0 << 16) | (3 << 12) | 0xFF;
        cpu.execute_arm_data_processing(opcode_eor);
        assert_eq!(cpu.read_reg(3), 0xF0F0_0FF0);

        // MOV r4, #0, S
        let opcode_mov = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (4 << 12) | 0;
        cpu.execute_arm_data_processing(opcode_mov);
        assert_eq!(cpu.read_reg(4), 0);
        assert!(cpu.cpsr().z());

        // BIC r5, r0, #0xF0
        let opcode_bic = (0xE << 28) | (1 << 25) | (0xE << 21) | (1 << 20) | (0 << 16) | (5 << 12) | 0xF0;
        cpu.execute_arm_data_processing(opcode_bic);
        assert_eq!(cpu.read_reg(5), cpu.read_reg(0) & !0xF0);

        // MVN r6, #0x00
        let opcode_mvn = (0xE << 28) | (1 << 25) | (0xF << 21) | (1 << 20) | (0 << 16) | (6 << 12) | 0x00;
        cpu.execute_arm_data_processing(opcode_mvn);
        assert_eq!(cpu.read_reg(6), 0xFFFF_FFFF);
    }

    #[test]
    fn dp_add_sub_adc_sbc_cmp_cmn_flags() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x7FFF_FFFF);
        // ADD r2, r1, #1, S -> overflow
        let opcode_add = (0xE << 28) | (1 << 25) | (0x4 << 21) | (1 << 20) | (1 << 16) | (2 << 12) | 0x01;
        cpu.execute_arm_data_processing(opcode_add);
        assert_eq!(cpu.read_reg(2), 0x8000_0000);
        assert!(cpu.cpsr().v());
        assert!(cpu.cpsr().n());

        // ADC r3, r1, #0, with C=1 -> r3 = r1 + 1
        cpu.cpsr_mut().set_c(true);
        let opcode_adc = (0xE << 28) | (1 << 25) | (0x5 << 21) | (1 << 20) | (1 << 16) | (3 << 12) | 0x00;
        cpu.execute_arm_data_processing(opcode_adc);
        assert_eq!(cpu.read_reg(3), 0x8000_0000);

        // SUB r4, r3, #1, S -> result positive, overflow set
        let opcode_sub = (0xE << 28) | (1 << 25) | (0x2 << 21) | (1 << 20) | (3 << 16) | (4 << 12) | 0x01;
        cpu.execute_arm_data_processing(opcode_sub);
        assert_eq!(cpu.read_reg(4), 0x7FFF_FFFF);
        assert!(!cpu.cpsr().n());
        assert!(cpu.cpsr().v());

        // CMP r1, r0 -> result 0 -> Z=1, C=1
        cpu.write_reg(0, 0x7FFF_FFFF);
        let opcode_cmp = (0xE << 28) | (0xA << 21) | (1 << 16) | (1 << 12) | 0x0; // I=0, Rm=0
        cpu.execute_arm_data_processing(opcode_cmp);
        assert!(cpu.cpsr().z());
        assert!(cpu.cpsr().c());

        // SBC r5, r4, #0 with C=0 -> r5 = r4 - 1
        cpu.cpsr_mut().set_c(false);
        let opcode_sbc = (0xE << 28) | (1 << 25) | (0x6 << 21) | (1 << 20) | (4 << 16) | (5 << 12) | 0x00;
        cpu.execute_arm_data_processing(opcode_sbc);
        assert_eq!(cpu.read_reg(5), 0x7FFF_FFFE);
    }

    #[test]
    fn pipeline_flush_on_mov_pc_immediate() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(64);
        // MOV r15, #0x10 (pc = 0x10)
        let mov_pc = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (15 << 12) | 0x10;
        // Place MOV PC at 0x4 so it executes on first step (pipeline executes PC+4 initially)
        write32_le(&mut bus.mem, 4, mov_pc);
        // Target region: write a MOV r1, #2 at 0x10 so we can observe execution after flush
        let mov_r1_2 = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (1 << 12) | 0x02;
        write32_le(&mut bus.mem, 0x10, mov_r1_2);
        write32_le(&mut bus.mem, 0x14, mov_r1_2);

        cpu.set_pc(0);
        cpu.step(&mut bus); // executes MOV PC at 0x4, flushes to 0x10 and preloads pipeline
        assert_eq!(cpu.pc(), 0x10);

        cpu.step(&mut bus); // should execute MOV r1, #2 from new pipeline
        assert_eq!(cpu.read_reg(1), 2);
    }

    #[test]
    fn arm_branch_and_link_updates_pc_lr_and_flushes() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        // Place BL at 0x4 (first executed)
        // Target = 0x20, Ai=0x4, Ai+8=0xC, diff=0x14, imm24=0x5
        let imm24 = 0x5;
        let bl = (0xE << 28) | (0b101 << 25) | (1 << 24) | imm24;
        write32_le(&mut bus.mem, 4, bl);
        // At 0x20, put MOV r1,#3 to verify execution after branch
        let mov_r1_3 = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (1 << 12) | 0x03;
        write32_le(&mut bus.mem, 0x20, mov_r1_3);
        write32_le(&mut bus.mem, 0x24, mov_r1_3);

        cpu.set_pc(0);
        cpu.step(&mut bus); // executes BL
        assert_eq!(cpu.pc(), 0x20);
        assert_eq!(cpu.read_reg(14), 0x8); // LR = Ai+4

        cpu.step(&mut bus); // execute MOV at target
        assert_eq!(cpu.read_reg(1), 3);
    }

    #[test]
    fn arm_branch_without_link_updates_pc_and_flushes() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        // Place B to 0x1C at 0x4
        // Ai=0x4, Ai+8=0xC, target=0x1C, diff=0x10, imm24=0x4
        let b = (0xE << 28) | (0b101 << 25) | 0x4;
        write32_le(&mut bus.mem, 4, b);
        // At 0x1C, MOV r2,#7
        let mov_r2_7 = (0xE << 28) | (1 << 25) | (0xD << 21) | (1 << 20) | (0 << 16) | (2 << 12) | 0x07;
        write32_le(&mut bus.mem, 0x1C, mov_r2_7);
        write32_le(&mut bus.mem, 0x20, mov_r2_7);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.pc(), 0x1C);
        assert_eq!(cpu.read_reg(14), 0); // LR unchanged
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2), 7);
    }

    #[test]
    fn arm_mul_and_mla_set_flags_and_write_result() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(64);
        // Place MUL r2, r0, r1 (r2 = r0*r1), S=1 at 0x4
        cpu.write_reg(0, 3);
        cpu.write_reg(1, 5);
        let mul = (0xE << 28) | (1 << 20) | (2 << 16) | (1 << 8) | (1 << 7) | (1 << 4);
        // MLA r3, r0, r1, r2 (r3 = r0*r1 + r2), S=1 at 0x8
        let mla = (0xE << 28) | (1 << 21) | (1 << 20) | (3 << 16) | (2 << 12) | (1 << 8) | (1 << 7) | (1 << 4);
        // Write both before first step to populate pipeline correctly
        write32_le(&mut bus.mem, 4, mul);
        write32_le(&mut bus.mem, 8, mla);
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2), 15);
        assert!(!cpu.cpsr().n());
        assert!(!cpu.cpsr().z());
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(3), 30);
        assert!(!cpu.cpsr().n());
        assert!(!cpu.cpsr().z());
    }

    #[test]
    fn arm_str_and_ldr_word_immediate_preindexed_aligned() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        // Instruction at 0x4 executes first. We'll STR r1, [r0,#16], then LDR r2, [r0,#16]
        cpu.write_reg(0, 0x40);
        cpu.write_reg(1, 0xDEADBEEF);
        // STR: cond=E, I=0, P=1, U=1, B=0, W=0, L=0, rn=0, rd=1, imm12=16
        let str_instr = (0xE << 28) | (1 << 26) | (0 << 25) | (1 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (0 << 20) | (0 << 16) | (1 << 12) | 16;
        // LDR: cond=E, I=0, P=1, U=1, B=0, W=0, L=1, rn=0, rd=2, imm12=16
        let ldr_instr = (0xE << 28) | (1 << 26) | (0 << 25) | (1 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (1 << 20) | (0 << 16) | (2 << 12) | 16;
        write32_le(&mut bus.mem, 4, str_instr);
        write32_le(&mut bus.mem, 8, ldr_instr);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        // Check memory written
        let word = (bus.mem[0x50] as u32)
            | ((bus.mem[0x51] as u32) << 8)
            | ((bus.mem[0x52] as u32) << 16)
            | ((bus.mem[0x53] as u32) << 24);
        assert_eq!(word, 0xDEADBEEF);

        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2), 0xDEADBEEF);
    }

    #[test]
    fn arm_halfword_and_signed_transfers() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        cpu.write_reg(0, 0x40);
        cpu.write_reg(1, 0x1234_5678);
        // STRH r1, [r0,#6] at 0x4; LDRH r2, [r0,#6] at 0x8
        let imm6: u32 = 6;
        let imm6_hi = (imm6 & 0xF0) << 4;
        let imm6_lo = imm6 & 0x0F;
        // Base: cond=E, 000, P=1, U=1, bit22=1, W=0, L=0, rn=0, rd=1, bits7:4=1011, S=0,H=1
        let strh = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | (1 << 12) | imm6_hi | (1 << 7) | (0 << 6) | (1 << 5) | (1 << 4) | imm6_lo;
        // LDRH: W=0, L=1, rd=2
        let ldrh = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (2 << 12) | imm6_hi | (1 << 7) | (0 << 6) | (1 << 5) | (1 << 4) | imm6_lo;
        write32_le(&mut bus.mem, 4, strh);
        write32_le(&mut bus.mem, 8, ldrh);

        cpu.set_pc(0);
        cpu.step(&mut bus);
        // Expect low half of r1 stored at 0x46
        let half = (bus.mem[0x46] as u16) | ((bus.mem[0x47] as u16) << 8);
        assert_eq!(half, 0x5678);

        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2) & 0xFFFF, 0x5678);
    }

    #[test]
    fn arm_signed_byte_and_halfword_loads() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        cpu.write_reg(0, 0x40);
        // LDRSB r3, [r0,#5] at 0x4; LDRSH r4, [r0,#6] at 0x8
        bus.mem[0x45] = 0xF0;
        bus.mem[0x46] = 0x78; bus.mem[0x47] = 0x56;
        let imm5: u32 = 5; let imm5_hi = (imm5 & 0xF0) << 4; let imm5_lo = imm5 & 0x0F;
        let imm6: u32 = 6; let imm6_hi = (imm6 & 0xF0) << 4; let imm6_lo = imm6 & 0x0F;
        let ldrsb = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (3 << 12) | imm5_hi | (1 << 7) | (1 << 6) | (0 << 5) | (1 << 4) | imm5_lo;
        let ldrsh = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (4 << 12) | imm6_hi | (1 << 7) | (1 << 6) | (1 << 5) | (1 << 4) | imm6_lo;
        write32_le(&mut bus.mem, 4, ldrsb);
        write32_le(&mut bus.mem, 8, ldrsh);
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(3), 0xFFFF_FFF0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(4), 0x0000_5678);
    }

    #[test]
    fn arm_ldrsb_direct_execute() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(64);
        cpu.write_reg(0, 0x10);
        bus.mem[0x15] = 0xF0;
        let imm: u32 = 5;
        let imm_hi = (imm & 0xF0) << 8;
        let imm_lo = imm & 0x0F;
        let ldrsb = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (3 << 12) | imm_hi | (1 << 7) | (1 << 6) | (0 << 5) | (1 << 4) | imm_lo;
        cpu.execute_arm_halfword_transfer(&mut bus, ldrsb);
        assert_eq!(cpu.read_reg(3), 0xFFFF_FFF0);
    }

    #[test]
    fn arm_ldrsb_step_dispatch() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(64);
        cpu.write_reg(0, 0x10);
        bus.mem[0x15] = 0xF0;
        let imm: u32 = 5;
        let imm_hi = (imm & 0xF0) << 8;
        let imm_lo = imm & 0x0F;
        // LDRSB r3, [r0,#5] at 0x4
        let ldrsb = (0xE << 28) | (1 << 24) | (1 << 23) | (1 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (3 << 12) | imm_hi | (1 << 7) | (1 << 6) | (0 << 5) | (1 << 4) | imm_lo;
        // Sanity check bits
        assert_eq!(((ldrsb >> 6) & 1), 1);
        assert_eq!(((ldrsb >> 5) & 1), 0);
        write32_le(&mut bus.mem, 4, ldrsb);
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(3), 0xFFFF_FFF0);
    }

    #[test]
    fn arm_swp_and_swpb() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(128);
        cpu.write_reg(0, 0x40); // Rn
        cpu.write_reg(1, 0x1122_3344); // Rm
        // Initialize memory: word at 0x40 and byte at 0x41
        write32_le(&mut bus.mem, 0x40, 0xAABB_CCDD);
        bus.mem[0x41] = 0xFE;
        // SWP r2, r1, [r0] at 0x4
        let swp = (0xE << 28) | (0b00010 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | (2 << 12) | (0 << 8) | (0b1001 << 4) | 1;
        write32_le(&mut bus.mem, 4, swp);
        cpu.set_pc(0);

        // SWP: r2 <= [0x40] old, [0x40] <= r1
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(2), 0xAABB_FEDD);
        let word = (bus.mem[0x40] as u32) | ((bus.mem[0x41] as u32) << 8) | ((bus.mem[0x42] as u32) << 16) | ((bus.mem[0x43] as u32) << 24);
        assert_eq!(word, 0x1122_3344);
    }

    #[test]
    fn arm_psr_mrs_msr_flags() {
        let mut cpu = Cpu::new();
        // MSR CPSR_f, #imm set N and C
        let imm8 = 0b1010_0000; // N=1,C=1 after rotation 0
        let msr_imm = (0xE << 28) | (0b00110 << 23) | (1 << 21) | (0xF << 16) | (0 << 8) | imm8;
        cpu.execute_arm_psr_transfer(msr_imm);
        assert!(cpu.cpsr().n());
        assert!(cpu.cpsr().c());
        // MRS CPSR -> r1
        let mrs = (0xE << 28) | (0b00010 << 23) | (0 << 22) | (0 << 21) | (0xF << 16) | (1 << 12);
        cpu.execute_arm_psr_transfer(mrs);
        assert_eq!(cpu.read_reg(1) & 0xF000_0000, 0xA000_0000);
    }

    #[test]
    fn arm_block_transfer_stmia_ldmia() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(256);
        cpu.write_reg(0, 0x80); // base
        // Pre-fill memory for LDMIA
        write32_le(&mut bus.mem, 0x80, 0x1111_1111);
        write32_le(&mut bus.mem, 0x84, 0x2222_2222);
        write32_le(&mut bus.mem, 0x88, 0x3333_3333);
        // LDMIA r0, {r4-r6} at 0x4
        let ldmia = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | ((1<<4)|(1<<5)|(1<<6));
        write32_le(&mut bus.mem, 4, ldmia);
        cpu.set_pc(0);
        cpu.step(&mut bus);
        assert_eq!(cpu.read_reg(4), 0x1111_1111);
        assert_eq!(cpu.read_reg(5), 0x2222_2222);
        assert_eq!(cpu.read_reg(6), 0x3333_3333);
    }

    #[test]
    fn arm_block_transfer_addressing_modes() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(256);

        // Test STMIA (Increment After)
        cpu.write_reg(0, 0x100); // base
        cpu.write_reg(1, 0x1111_1111);
        cpu.write_reg(2, 0x2222_2222);
        let stmia = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | ((1<<1)|(1<<2));
        cpu.execute_arm_block_transfer(&mut bus, stmia);
        assert_eq!(bus.read32(0x100), 0x1111_1111);
        assert_eq!(bus.read32(0x104), 0x2222_2222);
        assert_eq!(cpu.read_reg(0), 0x100); // no writeback

        // Test STMIB (Increment Before) with writeback
        cpu.write_reg(0, 0x200); // base
        cpu.write_reg(3, 0x3333_3333);
        cpu.write_reg(4, 0x4444_4444);
        let stmib = (0xE << 28) | (0b100 << 25) | (1 << 24) | (1 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | ((1<<3)|(1<<4));
        cpu.execute_arm_block_transfer(&mut bus, stmib);
        assert_eq!(bus.read32(0x204), 0x3333_3333);
        assert_eq!(bus.read32(0x208), 0x4444_4444);
        assert_eq!(cpu.read_reg(0), 0x20C); // writeback enabled

        // Test STMDA (Decrement After)
        cpu.write_reg(0, 0x300); // base
        cpu.write_reg(5, 0x5555_5555);
        cpu.write_reg(6, 0x6666_6666);
        let stmda = (0xE << 28) | (0b100 << 25) | (0 << 24) | (0 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | ((1<<5)|(1<<6));
        cpu.execute_arm_block_transfer(&mut bus, stmda);
        assert_eq!(bus.read32(0x2F8), 0x5555_5555);
        assert_eq!(bus.read32(0x2FC), 0x6666_6666);
        assert_eq!(cpu.read_reg(0), 0x300); // no writeback

        // Test STMDB (Decrement Before) with writeback
        cpu.write_reg(0, 0x400); // base
        cpu.write_reg(7, 0x7777_7777);
        cpu.write_reg(8, 0x8888_8888);
        let stmdb = (0xE << 28) | (0b100 << 25) | (1 << 24) | (0 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | ((1<<7)|(1<<8));
        cpu.execute_arm_block_transfer(&mut bus, stmdb);
        assert_eq!(bus.read32(0x3F4), 0x7777_7777); // r7 at start address
        assert_eq!(bus.read32(0x3F8), 0x8888_8888); // r8 at start address + 4
        assert_eq!(cpu.read_reg(0), 0x3F4); // writeback enabled
    }

    #[test]
    fn arm_block_transfer_pc_handling() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(512); // Increase size to handle larger addresses

        // Test STM with PC (should store PC+12)
        cpu.write_reg(0, 0x100); // base
        cpu.set_pc(0x1000);
        let stm_pc = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | (1<<15); // store PC
        cpu.execute_arm_block_transfer(&mut bus, stm_pc);
        assert_eq!(bus.read32(0x100), 0x100C); // PC+12

        // Test LDM with PC (should cause pipeline flush)
        cpu.write_reg(0, 0x200); // base
        write32_le(&mut bus.mem, 0x200, 0x2000);
        let ldm_pc = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | (1<<15); // load PC
        cpu.execute_arm_block_transfer(&mut bus, ldm_pc);
        assert_eq!(cpu.read_reg(15), 0x2000);
    }

    #[test]
    fn arm_block_transfer_empty_list() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(512); // Increase size to handle larger addresses

        // Test LDM with empty list (should load PC from address)
        cpu.write_reg(0, 0x100); // base
        write32_le(&mut bus.mem, 0x100, 0x3000);
        let ldm_empty = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (1 << 20)
            | (0 << 16) | 0; // empty register list
        cpu.execute_arm_block_transfer(&mut bus, ldm_empty);
        assert_eq!(cpu.read_reg(15), 0x3000);

        // Test STM with empty list (should store PC+12 to address)
        cpu.write_reg(0, 0x200); // base
        cpu.set_pc(0x4000);
        let stm_empty = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | 0; // empty register list
        cpu.execute_arm_block_transfer(&mut bus, stm_empty);
        assert_eq!(bus.read32(0x200), 0x400C); // PC+12
    }

    #[test]
    fn arm_block_transfer_writeback_modes() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(256);

        // Test STMIA with writeback
        cpu.write_reg(0, 0x100); // base
        cpu.write_reg(1, 0x1111_1111);
        cpu.write_reg(2, 0x2222_2222);
        let stmia_wb = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | ((1<<1)|(1<<2));
        cpu.execute_arm_block_transfer(&mut bus, stmia_wb);
        assert_eq!(cpu.read_reg(0), 0x108); // base + 2*4

        // Test STMIB with writeback
        cpu.write_reg(0, 0x200); // base
        cpu.write_reg(3, 0x3333_3333);
        let stmib_wb = (0xE << 28) | (0b100 << 25) | (1 << 24) | (1 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | (1<<3);
        cpu.execute_arm_block_transfer(&mut bus, stmib_wb);
        assert_eq!(cpu.read_reg(0), 0x208); // base + 4 + 1*4

        // Test STMDA with writeback
        cpu.write_reg(0, 0x300); // base
        cpu.write_reg(4, 0x4444_4444);
        cpu.write_reg(5, 0x5555_5555);
        let stmda_wb = (0xE << 28) | (0b100 << 25) | (0 << 24) | (0 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | ((1<<4)|(1<<5));
        cpu.execute_arm_block_transfer(&mut bus, stmda_wb);
        assert_eq!(cpu.read_reg(0), 0x2F8); // base - 2*4

        // Test STMDB with writeback
        cpu.write_reg(0, 0x400); // base
        cpu.write_reg(6, 0x6666_6666);
        let stmdb_wb = (0xE << 28) | (0b100 << 25) | (1 << 24) | (0 << 23) | (0 << 22) | (1 << 21) | (0 << 20)
            | (0 << 16) | (1<<6);
        cpu.execute_arm_block_transfer(&mut bus, stmdb_wb);
        assert_eq!(cpu.read_reg(0), 0x3F8); // base - 4 - 1*4
    }

    #[test]
    fn arm_block_transfer_register_ordering() {
        let mut cpu = Cpu::new();
        let mut bus = MockBus::new(256);

        // Test that registers are transferred in ascending order regardless of bit order
        cpu.write_reg(0, 0x100); // base
        cpu.write_reg(1, 0x1111_1111);
        cpu.write_reg(3, 0x3333_3333);
        cpu.write_reg(7, 0x7777_7777);
        cpu.set_pc(0x1000); // Set PC to a known value

        // Register list: r1, r3, r7, r15 (not in bit order)
        let reg_list = (1<<1) | (1<<3) | (1<<7) | (1<<15);
        let stmia = (0xE << 28) | (0b100 << 25) | (0 << 24) | (1 << 23) | (0 << 22) | (0 << 21) | (0 << 20)
            | (0 << 16) | reg_list;
        cpu.execute_arm_block_transfer(&mut bus, stmia);

        // Should be stored in ascending register order: r1, r3, r7, r15
        assert_eq!(bus.read32(0x100), 0x1111_1111); // r1
        assert_eq!(bus.read32(0x104), 0x3333_3333); // r3
        assert_eq!(bus.read32(0x108), 0x7777_7777); // r7
        assert_eq!(bus.read32(0x10C), 0x100C); // r15 (PC+12)
    }
}
