use emulator::cpu;
use emulator::util;

pub type Operation = fn(&mut cpu::CPU, cpu::addressing::AddressingMode) -> u32;

fn update_zero_flag(cpu: &mut cpu::CPU, result: u8) {
    if result == 0 {
        cpu.p.set(cpu::flags::Flag::Z);
    } else {
        cpu.p.clear(cpu::flags::Flag::Z);
    }
}

fn update_negative_flag(cpu: &mut cpu::CPU, result: u8) {
    if (result & 0b1000_0000) != 0 {
        cpu.p.set(cpu::flags::Flag::N);
    } else {
        cpu.p.clear(cpu::flags::Flag::N);
    }
}

/* 2.1 The Accumulator */

// LDA: Load Accumulator with Memory
// A -> M
pub fn lda(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let res = cpu.load_memory(addr);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;

    addr_cycles
}

// STA: Store Accumulator in Memory
// M -> A
pub fn sta(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.a;
    cpu.store_memory(addr, byte);
    addr_cycles
}

/* 2.2 The Arithmetic Unit */

// ADC: Add Memory to Accumulator with Carry
// A + M + C -> A, C
pub fn adc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let carry_val: u8 = if cpu.p.is_set(cpu::flags::Flag::C) { 1 } else { 0 };
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) {
        // BCD arithmetic.
        let hex_a = util::bcd_to_hex(cpu.a);
        let hex_mem = util::bcd_to_hex(mem);

        // Cannot be > 255 so don't need to check for wrapping.
        let hex_res = hex_a + hex_mem + carry_val;

        // Wrap to <99.  Max value is 199 so only need to check once.
        if hex_res <= 99 {
            (util::hex_to_bcd(hex_res), false)
        } else {
            (util::hex_to_bcd(hex_res - 100), true)
        }
    } else {
        // Normal arithmetic.
        let (res, carry1) = cpu.a.overflowing_add(mem);
        let (res, carry2) = res.overflowing_add(carry_val);
        (res, carry1 || carry2)
    };
    
    // Set carry flag.
    if carry {
        cpu.p.set(cpu::flags::Flag::C);
    } else {
        cpu.p.clear(cpu::flags::Flag::C);
    }

    // Set overflow flag.
    let a_sign = cpu.a & 0b1000_0000;
    let mem_sign = mem & 0b1000_0000;
    let res_sign = res & 0b1000_0000;
    if (a_sign == mem_sign) && (a_sign != res_sign) {
        cpu.p.set(cpu::flags::Flag::V);
    } else {
        cpu.p.clear(cpu::flags::Flag::V);
    }

    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);

    cpu.a = res;
    addr_cycles
}

// SBC: Subtract Memory from Accumulator with Borrow
// A - M - ~C -> A
// Borrow = Complement of carry
pub fn sbc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let carry_val: u8 = if cpu.p.is_set(cpu::flags::Flag::C) { 1 } else { 0 };
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) {
        // BCD arithmetic.
        let hex_a = util::bcd_to_hex(cpu.a);
        let hex_mem = util::bcd_to_hex(mem);
        let borrow = 1 - carry_val;
        let hex_sub_amount = hex_mem + borrow;
        let (res, borrow) = hex_a.overflowing_sub(hex_sub_amount);

        // If we wrapped then we wrapped to 255.  Fudge it so we actually wrap to 99.
        if borrow {
            (util::hex_to_bcd(res - (255 - 99)), false)
        } else {
            (util::hex_to_bcd(res), true)
        }
    } else {
        // Normal arithmetic.
        let (minus_m, _) = (!mem).overflowing_add(carry_val);
        cpu.a.overflowing_add(minus_m)
    };
    
    // Set carry flag.
    if carry {
        cpu.p.set(cpu::flags::Flag::C);
    } else {
        cpu.p.clear(cpu::flags::Flag::C);
    }

    //  Set overflow flag.
    let a_sign = cpu.a & 0b1000_0000;
    let mem_sign = mem & 0b1000_0000;
    let res_sign = res & 0b1000_0000;
    if (a_sign != mem_sign) && (mem_sign == res_sign) {
        cpu.p.set(cpu::flags::Flag::V);
    } else {
        cpu.p.clear(cpu::flags::Flag::V);
    }

    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);

    cpu.a = res;
    addr_cycles
}

// AND: Bitwise AND Memory with Accumulator
// A /\ M -> A
pub fn and(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem & cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

// ORA: Bitwise OR Memory with Accumulator
// A \/ M -> A
pub fn ora(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem | cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

// EOR: Bitwise Exclusive OR Memory with Accumulator
// A \-/ M -> A
pub fn eor(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem ^ cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

/* 3. Flags and Status Register */

// SEC: Set Carry Flag
// 1 -> C
pub fn sec(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::C);
    0
}

// CLC: Clear Carry Flag
// 0 -> C
pub fn clc(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::C);
    0
}

// SEI: Set Interrupt Disable
// 1 -> I
pub fn sei(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::I);
    0
}

// CLI: Clear Interrupt Disable
// 0 -> I
pub fn cli(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::I);
    0
}

// SED: Set Decimal Mode
// 1 -> D
pub fn sed(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::D);
    0
}

// CLD: Clear Decimal Mode
// 0 -> D
pub fn cld(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::D);
    0
}

// CLV: Clear Overflow Flag
// 0 -> V
pub fn clv(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::V);
    0
}

/* 4. Test, Branch and Jump Instructions */

// JMP: Jump to New Location
// (PC + 1) -> PCL, (PC + 2) -> PCH
pub fn jmp(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    cpu.pc = addr;
    addr_cycles
}

// Common functionality for branch instructions.
fn branch_if(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode, should_branch: bool) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    if should_branch {
        cpu.pc = addr;
        addr_cycles + 1
    } else {
        // If not branching we don't incur any of the extra cycles.
        0
    }
}

// BMI - Branch on Result Minus
pub fn bmi(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::N);
    branch_if(cpu, load_addr, should_branch)
}

// BPL - Branch on Result Plus
pub fn bpl(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::N);
    branch_if(cpu, load_addr, should_branch)
}

// BCC - Branch on Carry Clear
pub fn bcc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::C);
    branch_if(cpu, load_addr, should_branch)
}

// BCS - Branch on Carry Set
pub fn bcs(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::C);
    branch_if(cpu, load_addr, should_branch)
}

// BEQ - Branch on Result Zero
pub fn beq(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::Z);
    branch_if(cpu, load_addr, should_branch)
}

// BNE - Branch on Result Not Zero
pub fn bne(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::Z);
    branch_if(cpu, load_addr, should_branch)
}

// BVS - Branch on Overflow Set
pub fn bvs(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::V);
    branch_if(cpu, load_addr, should_branch)
}

// BVC - Branch on Overflow Clear
pub fn bvc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::V);
    branch_if(cpu, load_addr, should_branch)
}

// CMP - Compare Memory and Accumulator
// A - M
pub fn cmp(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let diff = cpu.a.wrapping_sub(mem);
    update_zero_flag(cpu, diff);
    update_negative_flag(cpu, diff);

    if cpu.a < mem {
        cpu.p.clear(cpu::flags::Flag::C);
    } else {
        cpu.p.set(cpu::flags::Flag::C);
    }

    addr_cycles
}
