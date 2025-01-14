use super::super::bus::cpu_bus::CpuBus;
use super::super::cpu_registers::CpuRegisters;
use super::super::helper::*;
use super::super::types::{Addr, Data, Word};

use std::num::Wrapping;

pub fn process_nmi<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    registers.set_break(false);
    push((registers.get_PC() >> 8) as u8, registers, bus);
    push(registers.get_PC() as u8, registers, bus);
    push_status(registers, bus, false);
    registers.set_interrupt(true);
    let next = bus.read_word(0xFFFA);
    registers.set_PC(next);
}

pub fn lda<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let computed = bus.read(operand);
    registers
        .set_A(computed)
        .update_negative_by(computed)
        .update_zero_by(computed);
}

pub fn lda_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    registers
        .set_A(operand as Data)
        .update_negative_by(operand as Data)
        .update_zero_by(operand as Data);
}

pub fn ldx<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let computed = bus.read(operand);
    registers
        .set_X(computed)
        .update_negative_by(computed)
        .update_zero_by(computed);
}

pub fn ldx_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    registers
        .set_X(operand as Data)
        .update_negative_by(operand as Data)
        .update_zero_by(operand as Data);
}

pub fn ldy<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let computed = bus.read(operand);
    registers
        .set_Y(computed)
        .update_negative_by(computed)
        .update_zero_by(computed);
}

pub fn ldy_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    registers
        .set_Y(operand as Data)
        .update_negative_by(operand as Data)
        .update_zero_by(operand as Data);
}

pub fn sta<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    bus.write(operand, registers.get_A());
}

pub fn stx<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    bus.write(operand, registers.get_X());
}

pub fn sty<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    bus.write(operand, registers.get_Y());
}

pub fn txa<T: CpuRegisters>(registers: &mut T) {
    let x = registers.get_X();
    registers.set_A(x).update_negative_by(x).update_zero_by(x);
}

pub fn tya<T: CpuRegisters>(registers: &mut T) {
    let y = registers.get_Y();
    registers.set_A(y).update_negative_by(y).update_zero_by(y);
}

pub fn txs<T: CpuRegisters>(registers: &mut T) {
    let x = registers.get_X();
    registers.set_SP(x);
}

pub fn tay<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    registers
        .set_Y(acc)
        .update_negative_by(acc)
        .update_zero_by(acc);
}

pub fn tax<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    registers
        .set_X(acc)
        .update_negative_by(acc)
        .update_zero_by(acc);
}

pub fn tsx<T: CpuRegisters>(registers: &mut T) {
    let sp = registers.get_SP();
    registers
        .set_X(sp)
        .update_negative_by(sp)
        .update_zero_by(sp);
}

pub fn php<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    // PHP pushes with the B flag as 1, no matter what
    push_status(registers, bus, true);
}

pub fn plp<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    // pull flags but keep B
    // registers.set_reserved(true);
    let previous_b = (registers.get_break() as u8) << 4;
    let status = pop(registers, bus) & 0xCF | previous_b;
    registers.set_P(status);
}

pub fn pha<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let acc = registers.get_A();
    push(acc, registers, bus);
}

pub fn pla<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let v = pop(registers, bus);
    registers.set_A(v).update_negative_by(v).update_zero_by(v);
}

pub fn adc_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed =
        (operand as u16) + registers.get_A() as u16 + bool_to_u8(registers.get_carry()) as u16;
    let acc = registers.get_A();
    registers
        .set_overflow(
            !(((acc ^ (operand as Data)) & 0x80) != 0) && ((acc ^ computed as Data) & 0x80) != 0,
        )
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed > 0xFF)
        .set_A(computed as Data);
}

pub fn adc<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed =
        fetched as u16 + registers.get_A() as u16 + bool_to_u8(registers.get_carry()) as u16;
    let acc = registers.get_A();
    registers
        .set_overflow(
            !(((acc ^ (fetched as Data)) & 0x80) != 0) && ((acc ^ computed as Data) & 0x80) != 0,
        )
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed > 0xFF)
        .set_A(computed as Data);
}

pub fn sbc_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed =
        registers.get_A() as i16 - (operand as i16) - bool_to_u8(!registers.get_carry()) as i16;
    let acc = registers.get_A();
    registers
        .set_overflow(
            (((acc ^ (operand as Data)) & 0x80) != 0) && ((acc ^ computed as Data) & 0x80) != 0,
        )
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16)
        .set_A(computed as Data);
}

pub fn sbc<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed =
        registers.get_A() as i16 - fetched as i16 - bool_to_u8(!registers.get_carry()) as i16;
    let acc = registers.get_A();
    registers
        .set_overflow(
            (((acc ^ (fetched as Data)) & 0x80) != 0) && ((acc ^ computed as Data) & 0x80) != 0,
        )
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16)
        .set_A(computed as Data);
}

pub fn cpx_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = registers.get_X() as i16 - (operand as i16);
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn cpx<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = registers.get_X() as i16 - fetched as i16;
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn cpy_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = registers.get_Y() as i16 - (operand as i16);
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn cpy<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = registers.get_Y() as i16 - fetched as i16;
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn cmp_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = (registers.get_A() as i16) - (operand as i16);
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn cmp<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = (registers.get_A() as i16) - (fetched as i16);
    registers
        .update_negative_by(computed as Data)
        .update_zero_by(computed as Data)
        .set_carry(computed >= 0 as i16);
}

pub fn and_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = registers.get_A() & (operand as u8);
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn and<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = registers.get_A() & fetched;
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn eor_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = registers.get_A() ^ (operand as u8);
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn eor<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = registers.get_A() ^ fetched;
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn ora_imm<T: CpuRegisters>(operand: Word, registers: &mut T) {
    let computed = registers.get_A() | (operand as u8);
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn ora<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let computed = registers.get_A() | fetched;
    registers
        .update_negative_by(computed)
        .update_zero_by(computed)
        .set_A(computed);
}

pub fn bit<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let acc = registers.get_A();
    registers
        .update_negative_by(fetched)
        .update_zero_by(fetched & acc)
        .set_overflow((fetched & 0x40) == 0x40);
}

pub fn asl_acc<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    let shifted = (acc << 1) as u8;
    registers
        .set_carry(acc & 0x80 == 0x80)
        .update_negative_by(shifted)
        .update_zero_by(shifted)
        .set_A(shifted);
}

pub fn asl<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let shifted = (fetched << 1) as u8;
    registers
        .set_carry(fetched & 0x80 == 0x80)
        .update_negative_by(shifted)
        .update_zero_by(shifted);
    bus.write(operand, shifted);
}

pub fn lsr_acc<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    let shifted = (acc >> 1) as u8;
    registers
        .set_carry((acc & 0x01) == 0x01)
        .update_negative_by(shifted)
        .update_zero_by(shifted)
        .set_A(shifted);
}

pub fn lsr<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let shifted = (fetched >> 1) as u8;
    registers
        .set_carry(fetched & 0x01 == 0x01)
        .update_negative_by(shifted)
        .update_zero_by(shifted);
    bus.write(operand, shifted);
}

pub fn rol_acc<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    let rotated = rotate_to_left(registers, acc);
    registers
        .set_carry(acc & 0x80 == 0x80)
        .update_negative_by(rotated)
        .update_zero_by(rotated)
        .set_A(rotated);
}

pub fn rol<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let rotated = rotate_to_left(registers, fetched);
    registers
        .set_carry(fetched & 0x80 == 0x80)
        .update_negative_by(rotated)
        .update_zero_by(rotated);
    bus.write(operand, rotated);
}

pub fn ror_acc<T: CpuRegisters>(registers: &mut T) {
    let acc = registers.get_A();
    let rotated = rotate_to_right(registers, acc);
    registers
        .set_carry(acc & 0x01 == 0x01)
        .update_negative_by(rotated)
        .update_zero_by(rotated)
        .set_A(rotated);
}

pub fn ror<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let fetched = bus.read(operand);
    let rotated = rotate_to_right(registers, fetched);
    registers
        .set_carry(fetched & 0x01 == 0x01)
        .update_negative_by(rotated)
        .update_zero_by(rotated);
    bus.write(operand, rotated);
}

pub fn inx<T: CpuRegisters>(registers: &mut T) {
    let x = registers.get_X().wrapping_add(1);
    registers.set_X(x).update_negative_by(x).update_zero_by(x);
}

pub fn iny<T: CpuRegisters>(registers: &mut T) {
    let y = registers.get_Y().wrapping_add(1);
    registers.set_Y(y).update_negative_by(y).update_zero_by(y);
}

pub fn inc<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let data = bus.read(operand) + 1 as u8;
    registers.update_negative_by(data).update_zero_by(data);
    bus.write(operand, data);
}

pub fn dex<T: CpuRegisters>(registers: &mut T) {
    let x = registers.get_X() as i8 - 1;
    registers
        .set_X(x as Data)
        .update_negative_by(x as Data)
        .update_zero_by(x as Data);
}

pub fn dey<T: CpuRegisters>(registers: &mut T) {
    let y = registers.get_Y() as i8 - 1;
    registers
        .set_Y(y as Data)
        .update_negative_by(y as Data)
        .update_zero_by(y as Data);
}

pub fn dec<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let data = bus.read(operand) as i8 - 1;
    registers
        .update_negative_by(data as Data)
        .update_zero_by(data as Data);
    bus.write(operand, data as Data);
}

pub fn clc<T: CpuRegisters>(registers: &mut T) {
    registers.set_carry(false);
}

pub fn cli<T: CpuRegisters>(registers: &mut T) {
    registers.set_interrupt(false);
}

pub fn clv<T: CpuRegisters>(registers: &mut T) {
    registers.set_overflow(false);
}

pub fn sec<T: CpuRegisters>(registers: &mut T) {
    registers.set_carry(true);
}

pub fn sei<T: CpuRegisters>(registers: &mut T) {
    registers.set_interrupt(true);
}

pub fn brk<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let interrupt = registers.get_interrupt();
    registers.inc_PC();
    push_pc(registers, bus);
    push_status(registers, bus, true);
    registers.set_interrupt(true);
    // Ignore interrupt when already set.
    if !interrupt {
        let fetched = bus.read_word(0xFFFE);
        registers.set_PC(fetched);
    }
    registers.dec_PC();
}

pub fn jsr<T: CpuRegisters, U: CpuBus>(operand: Word, registers: &mut T, bus: &mut U) {
    let pc = registers.get_PC() - 1;
    push((pc >> 8) as u8, registers, bus);
    push(pc as u8, registers, bus);
    registers.set_PC(operand);
}

pub fn jmp<T: CpuRegisters>(operand: Word, registers: &mut T) {
    registers.set_PC(operand);
}

pub fn rti<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    pop_status(registers, bus);
    pop_pc(registers, bus);
    registers.set_reserved(true);
}

pub fn rts<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    pop_pc(registers, bus);
    registers.inc_PC();
}

pub fn bcc<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if !registers.get_carry() {
        branch(registers, operand);
    }
}

pub fn bcs<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if registers.get_carry() {
        branch(registers, operand);
    }
}

pub fn beq<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if registers.get_zero() {
        branch(registers, operand);
    }
}

pub fn bmi<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if registers.get_negative() {
        branch(registers, operand);
    }
}

pub fn bne<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if !registers.get_zero() {
        branch(registers, operand);
    }
}

pub fn bpl<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if !registers.get_negative() {
        branch(registers, operand);
    }
}

pub fn bvs<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if registers.get_overflow() {
        branch(registers, operand);
    }
}

pub fn bvc<T: CpuRegisters>(operand: Word, registers: &mut T) {
    if !registers.get_overflow() {
        branch(registers, operand);
    }
}

pub fn cld<T: CpuRegisters>(registers: &mut T) {
    registers.set_decimal(false);
}

pub fn sed<T: CpuRegisters>(registers: &mut T) {
    registers.set_decimal(true);
}

// LAX. Load 'A' then Transfer X - LDA + TAX
pub fn lax<T: CpuRegisters, U: CpuBus>(addr: Word, registers: &mut T, bus: &mut U) {
    let a = bus.read(addr);
    registers
        .set_A(a)
        .set_X(a)
        .update_zero_by(a)
        .update_negative_by(a);
}

// SAX. Store A 'BitAnd' X
pub fn sax<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    let v = registers.get_A() & registers.get_X();
    bus.write(location, v);
}

// SAX. Store A 'BitAnd' X
pub fn dcp<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    let v = registers.get_A() & registers.get_X();
    bus.write(location, v);
}

pub fn isc<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    // INC
    let subtrahend = (Wrapping(bus.read(location)) + Wrapping(1)).0;
    bus.write(location, subtrahend);

    // SBC
    // High carry means "no borrow", thus negate and subtract
    let subtrahend = bus.read(location) as i16;
    let diff: i16 = {
        let a = Wrapping(registers.get_A() as i16);
        let v = Wrapping(subtrahend + !registers.get_carry() as i16);
        a - v
    }
    .0;
    // If the ninth bit is 1, the resulting number is negative => borrow => low carry
    registers.set_carry((diff & 0x0100) == 0);
    // Same as ADC, instead of the subtrahend, substitute with it's complement
    registers.set_overflow({
        let a = registers.get_A();
        let l = (a ^ diff as u8) & 0x80;
        let r = (a ^ (subtrahend as u8)) & 0x80;
        l != 0 && r != 0
    });
    let diff = (diff & 0xFF) as u8;
    registers
        .set_A(diff)
        .update_zero_by(diff)
        .update_negative_by(diff);
}

pub fn slo<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    // ASL
    let mut val = bus.read(location);
    registers.set_carry(val & 0x80 != 0);
    val <<= 1;
    bus.write(location, val);
    // ORA
    let a = registers.get_A() | val;
    registers.set_A(a).update_zero_by(a).update_negative_by(a);
}

pub fn rla<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    // ROL
    let prev_c = registers.get_carry();
    let mut operand = bus.read(location);
    registers.set_carry(operand & 0x80 != 0);
    operand <<= 1;
    operand |= prev_c as u8;
    bus.write(location, operand);
    // AND
    let a = operand & registers.get_A();
    registers.set_A(a).update_zero_by(a).update_negative_by(a);
}

pub fn sre<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    // LSR
    let mut operand = bus.read(location);
    registers.set_carry((operand & 0x01) != 0);
    operand >>= 1;
    bus.write(location, operand);
    // EOR
    let a = registers.get_A() ^ operand;
    registers.set_A(a).update_zero_by(a).update_negative_by(a);
}

pub fn rra<T: CpuRegisters, U: CpuBus>(location: Word, registers: &mut T, bus: &mut U) {
    // ROR
    let prev_c = registers.get_carry();
    let mut operand: u16 = bus.read(location) as u16;
    registers.set_carry(operand & 0x01 != 0);
    operand = (operand >> 1) | ((prev_c as u16) << 7);
    bus.write(location, (operand & 0xFF) as u8);
    // ADC
    let sum = {
        let a = Wrapping(registers.get_A() as u16);
        a + Wrapping(operand as u16) + Wrapping(registers.get_carry() as u16)
    }
    .0;
    registers.set_carry(sum & 0x0100 != 0);
    registers.set_overflow(((registers.get_A() as u16 ^ sum) & (operand ^ sum) & 0x80) != 0);
    let sum: u8 = (sum & 0xFF) as u8;
    registers
        .set_A(sum)
        .update_zero_by(sum)
        .update_negative_by(sum);
}

fn rotate_to_right<T: CpuRegisters>(registers: &mut T, v: Data) -> Data {
    ((v >> 1) as Data | if registers.get_carry() { 0x80 } else { 0x00 }) as Data
}

fn rotate_to_left<T: CpuRegisters>(registers: &mut T, v: Data) -> Data {
    ((v << 1) as Data | if registers.get_carry() { 0x01 } else { 0x00 }) as Data
}

fn push<T: CpuRegisters, U: CpuBus>(data: Data, registers: &mut T, bus: &mut U) {
    let addr = registers.get_SP() as Addr;
    bus.write(addr | 0x0100, data);
    registers.dec_SP();
}

fn push_status<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U, with_b_flag: bool) {
    let status = registers.get_P() | if with_b_flag { 1 << 4 } else { 0 };
    push(status, registers, bus);
}

fn pop<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) -> Data {
    let sp = registers.inc_SP().get_SP() as Addr;
    bus.read(0x0100 | sp)
}

fn pop_pc<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let lower = pop(registers, bus) as u16;
    let upper = pop(registers, bus) as u16;
    registers.set_PC(upper << 8 | lower);
}

fn pop_status<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let status = pop(registers, bus);
    registers.set_P(status);
}

fn push_pc<T: CpuRegisters, U: CpuBus>(registers: &mut T, bus: &mut U) {
    let pc = registers.get_PC();
    push((pc >> 8) as u8, registers, bus);
    push(pc as u8, registers, bus);
}

fn branch<T: CpuRegisters>(registers: &mut T, addr: Addr) {
    registers.set_PC(addr);
}

#[cfg(test)]
mod test {
    use super::super::super::cpu_registers::Registers;
    use super::*;

    struct MockBus {
        pub mem: Vec<Data>,
    }

    impl MockBus {
        pub fn new() -> Self {
            MockBus { mem: vec![0; 1024] }
        }
    }

    impl CpuBus for MockBus {
        fn read(&mut self, addr: Addr) -> Data {
            self.mem[addr as usize]
        }
        fn read_word(&mut self, addr: Addr) -> Word {
            let lower = self.read(addr) as u16;
            let upper = self.read(addr + 1) as u16;
            (upper << 8 | lower) as u16
        }
        fn write(&mut self, addr: Addr, data: Data) {
            self.mem[addr as usize] = data;
        }
    }

    #[test]
    fn test_lda_immediate() {
        let mut reg = Registers::new();
        lda_imm(0xA5, &mut reg);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_lda() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0xAA] = 0xA5;
        lda(0xAA, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_ldx_immediate() {
        let mut reg = Registers::new();
        ldx_imm(0xA5, &mut reg);
        assert_eq!(reg.get_X(), 0xA5);
    }

    #[test]
    fn test_ldx() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0xAA] = 0xA5;
        ldx(0xAA, &mut reg, &mut bus);
        assert_eq!(reg.get_X(), 0xA5);
    }

    #[test]
    fn test_ldy_immediate() {
        let mut reg = Registers::new();
        ldy_imm(0xA5, &mut reg);
        assert_eq!(reg.get_Y(), 0xA5);
    }

    #[test]
    fn test_ldy() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0xAA] = 0xA5;
        ldy(0xAA, &mut reg, &mut bus);
        assert_eq!(reg.get_Y(), 0xA5);
    }

    #[test]
    fn test_sta() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        let mut bus = MockBus::new();
        sta(0xAA, &mut reg, &mut bus);
        assert_eq!(bus.mem[0xAA], 0xA5);
    }

    #[test]
    fn test_stx() {
        let mut reg = Registers::new();
        reg.set_X(0xA5);
        let mut bus = MockBus::new();
        stx(0xAA, &mut reg, &mut bus);
        assert_eq!(bus.mem[0xAA], 0xA5);
    }

    #[test]
    fn test_sty() {
        let mut reg = Registers::new();
        reg.set_Y(0xA5);
        let mut bus = MockBus::new();
        sty(0xAA, &mut reg, &mut bus);
        assert_eq!(bus.mem[0xAA], 0xA5);
    }

    #[test]
    fn test_txa() {
        let mut reg = Registers::new();
        reg.set_X(0xA5);
        txa(&mut reg);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_tax() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        tax(&mut reg);
        assert_eq!(reg.get_X(), 0xA5);
    }

    #[test]
    fn test_tay() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        tay(&mut reg);
        assert_eq!(reg.get_Y(), 0xA5);
    }

    #[test]
    fn test_tya() {
        let mut reg = Registers::new();
        reg.set_Y(0xA5);
        tya(&mut reg);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_txs() {
        let mut reg = Registers::new();
        reg.set_X(0xA5);
        txs(&mut reg);
        assert_eq!(reg.get_SP(), 0xA5);
    }

    #[test]
    fn test_tsx() {
        let mut reg = Registers::new();
        reg.set_SP(0xA5);
        tsx(&mut reg);
        assert_eq!(reg.get_X(), 0xA5);
    }

    #[test]
    fn test_php() {
        {
            let mut reg = Registers::new();
            reg.set_SP(0xA5);
            reg.set_break(false);
            let mut bus = MockBus::new();
            php(&mut reg, &mut bus);
            assert_eq!(bus.mem[0x01A5], 0x34);
            assert_eq!(reg.get_break(), false);
        }
        {
            let mut reg = Registers::new();
            reg.set_SP(0xA5);
            reg.set_break(true);
            let mut bus = MockBus::new();
            php(&mut reg, &mut bus);
            assert_eq!(bus.mem[0x01A5], 0x34);
            assert_eq!(reg.get_break(), true);
        }
    }

    #[test]
    fn test_plp() {
        {
            let mut reg = Registers::new();
            reg.set_SP(0xA5);
            reg.set_break(false);
            let mut bus = MockBus::new();
            bus.mem[0x1A6] = 0xF5;
            plp(&mut reg, &mut bus);
            assert_eq!(reg.get_P(), 0xC5);
        }
        {
            let mut reg = Registers::new();
            reg.set_SP(0xA5);
            reg.set_break(true);
            let mut bus = MockBus::new();
            bus.mem[0x1A6] = 0xF5;
            plp(&mut reg, &mut bus);
            assert_eq!(reg.get_P(), 0xD5);
        }
    }

    #[test]
    fn test_pha() {
        let mut reg = Registers::new();
        reg.set_SP(0xA5).set_A(0x5A);
        let mut bus = MockBus::new();
        pha(&mut reg, &mut bus);
        assert_eq!(bus.mem[0x01A5], 0x5A);
    }

    #[test]
    fn test_pla() {
        let mut reg = Registers::new();
        reg.set_SP(0xA5);
        let mut bus = MockBus::new();
        bus.mem[0x1A6] = 0xAA;
        pla(&mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0xAA);
    }

    #[test]
    fn test_adc_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0x05);
        adc_imm(0xA5, &mut reg);
        assert_eq!(reg.get_A(), 0xAA);
    }

    #[test]
    fn test_adc() {
        let mut reg = Registers::new();
        reg.set_A(0x05);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0xAA;
        adc(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0xAF);
    }

    #[test]
    fn test_sbc_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0x10);
        sbc_imm(0x06, &mut reg);
        assert_eq!(reg.get_A(), 0x09);
    }

    #[test]
    fn test_sbc() {
        let mut reg = Registers::new();
        reg.set_A(0x10);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x06;
        sbc(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0x09);
    }

    #[test]
    fn test_cpx_immediate() {
        let mut reg = Registers::new();
        reg.set_X(0x05);
        cpx_imm(0x04, &mut reg);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_cpx() {
        let mut reg = Registers::new();
        reg.set_X(0x05);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x04;
        cpx(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_cpy_immediate() {
        let mut reg = Registers::new();
        reg.set_Y(0x05);
        cpy_imm(0x04, &mut reg);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_cpy() {
        let mut reg = Registers::new();
        reg.set_Y(0x05);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x04;
        cpy(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_cmp_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0x05);
        cmp_imm(0x04, &mut reg);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_cmp() {
        let mut reg = Registers::new();
        reg.set_A(0x05);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x04;
        cmp(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_carry(), true);
    }

    #[test]
    fn test_and_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        and_imm(0x05, &mut reg);
        assert_eq!(reg.get_A(), 0x05);
    }

    #[test]
    fn test_and() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x05;
        and(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0x05);
    }

    #[test]
    fn test_eor_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        eor_imm(0x05, &mut reg);
        assert_eq!(reg.get_A(), 0xA0);
    }

    #[test]
    fn test_eor() {
        let mut reg = Registers::new();
        reg.set_A(0xA5);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x05;
        eor(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0xA0);
    }

    #[test]
    fn test_ora_immediate() {
        let mut reg = Registers::new();
        reg.set_A(0xA0);
        ora_imm(0x05, &mut reg);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_ora() {
        let mut reg = Registers::new();
        reg.set_A(0xA0);
        let mut bus = MockBus::new();
        bus.mem[0xA5] = 0x05;
        ora(0xA5, &mut reg, &mut bus);
        assert_eq!(reg.get_A(), 0xA5);
    }

    #[test]
    fn test_asl_acc() {
        let mut reg = Registers::new();
        reg.set_A(0x55);
        asl_acc(&mut reg);
        assert_eq!(reg.get_A(), 0xAA);
    }

    #[test]
    fn test_asl() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0x00] = 0x55;
        asl(0x00, &mut reg, &mut bus);
        assert_eq!(bus.mem[0x00], 0xAA);
    }

    #[test]
    fn test_lsr_acc() {
        let mut reg = Registers::new();
        reg.set_A(0xAA);
        lsr_acc(&mut reg);
        assert_eq!(reg.get_A(), 0x55);
    }

    #[test]
    fn test_lsr() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0x00] = 0xAA;
        lsr(0x00, &mut reg, &mut bus);
        assert_eq!(bus.mem[0x00], 0x55);
    }

    #[test]
    fn test_rol_accumlator_with_carry() {
        let mut reg = Registers::new();
        reg.set_A(0x55).set_carry(true);
        rol_acc(&mut reg);
        assert_eq!(reg.get_A(), 0xAB);
    }

    #[test]
    fn test_ror_accumlator_with_carry() {
        let mut reg = Registers::new();
        reg.set_A(0x55).set_carry(true);
        ror_acc(&mut reg);
        assert_eq!(reg.get_A(), 0xAA);
    }

    #[test]
    fn test_ror_accumlator_without_carry() {
        let mut reg = Registers::new();
        reg.set_A(0x55);
        ror_acc(&mut reg);
        assert_eq!(reg.get_A(), 0x2A);
    }

    #[test]
    fn test_inx() {
        let mut reg = Registers::new();
        reg.set_X(0x55);
        inx(&mut reg);
        assert_eq!(reg.get_X(), 0x56);

        reg.set_X(0xFF);
        inx(&mut reg);
        assert_eq!(reg.get_X(), 0);
    }

    #[test]
    fn test_iny() {
        let mut reg = Registers::new();
        reg.set_Y(0x55);
        iny(&mut reg);
        assert_eq!(reg.get_Y(), 0x56);

        reg.set_Y(0xFF);
        iny(&mut reg);
        assert_eq!(reg.get_Y(), 0);
    }
    #[test]
    fn test_inc() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0x10] = 0xAA;
        inc(0x10, &mut reg, &mut bus);
        assert_eq!(bus.mem[0x10], 0xAB);
    }

    #[test]
    fn test_dex() {
        let mut reg = Registers::new();
        reg.set_X(0x55);
        dex(&mut reg);
        assert_eq!(reg.get_X(), 0x54);
    }

    #[test]
    fn test_dey() {
        let mut reg = Registers::new();
        reg.set_Y(0x55);
        dey(&mut reg);
        assert_eq!(reg.get_Y(), 0x54);
    }

    #[test]
    fn test_dec() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        bus.mem[0x10] = 0xAA;
        dec(0x10, &mut reg, &mut bus);
        assert_eq!(bus.mem[0x10], 0xA9);
    }

    #[test]
    fn test_jsr() {
        let mut reg = Registers::new();
        let mut bus = MockBus::new();
        jsr(0x10, &mut reg, &mut bus);
        assert_eq!(reg.get_PC(), 0x10);
    }

    #[test]
    fn test_jmp() {
        let mut reg = Registers::new();
        jmp(0x10, &mut reg);
        assert_eq!(reg.get_PC(), 0x10);
    }
}
