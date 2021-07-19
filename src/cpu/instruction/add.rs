use crate::register::{Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO};
use crate::mem::Mem;
use crate::cpu::instruction::{OperationSize, PcResult};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem<'a>,
    ea_format: String,
    ea_opmode: usize,
    register: usize,
    ea: u32,
) -> InstructionExecutionResult {
    const BYTE_WITH_DN_AS_DEST: usize = 0b000;
    const WORD_WITH_DN_AS_DEST: usize = 0b001;
    const LONG_WITH_DN_AS_DEST: usize = 0b010;
    const BYTE_WITH_EA_AS_DEST: usize = 0b100;
    const WORD_WITH_EA_AS_DEST: usize = 0b101;
    const LONG_WITH_EA_AS_DEST: usize = 0b110;
    let status_register_mask = 0xffe0;
    // TODO: Condition codes
    match ea_opmode {
        BYTE_WITH_DN_AS_DEST => {
            let in_mem = mem.get_unsigned_byte(ea);
            let in_reg = (reg.reg_d[register] & 0x000000ff) as u8;
            let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            let in_mem_signed = mem.get_signed_byte(ea);
            let in_reg_signed = (reg.reg_d[register] & 0x000000ff) as i8;
            let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            reg.reg_d[register] = (reg.reg_d[register] & 0xffffff00) | (in_reg as u32);
            let instr_comment = format!("adding {:#04x} to D{}", in_mem, register);

            let mut status_register_flags = 0x0000;
            match carry {
                true => status_register_flags |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
                false => (),
            }
            match overflow {
                true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                false => (),
            }
            match in_mem_signed {
                0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return InstructionExecutionResult {
                name: String::from("ADD.B"),
                operands_format: format!("{},D{}", ea_format, register),
                comment: instr_comment,
                op_size: OperationSize::Byte,
                pc_result: PcResult::Increment(2),
            };
        }
        LONG_WITH_DN_AS_DEST => {
            let in_mem = mem.get_unsigned_longword(ea);
            let in_reg = reg.reg_d[register];
            let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            let in_mem_signed = mem.get_signed_longword(ea);
            let in_reg_signed = reg.reg_d[register] as i32;
            let (in_reg_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            reg.reg_d[register] = in_reg;
            let instr_comment = format!("adding {:#010x} to D{}", in_mem, register);

            let mut status_register_flags = 0x0000;
            match carry {
                true => status_register_flags |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
                false => (),
            }
            match overflow {
                true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                false => (),
            }
            match in_mem_signed {
                0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                i32::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return InstructionExecutionResult {
                name: String::from("ADD.L"),
                operands_format: format!("{},D{}", ea_format, register),
                comment: instr_comment,
                op_size: OperationSize::Long,
                pc_result: PcResult::Increment(2),
            };
        }
        _ => panic!("Unhandled ea_opmode"),
    }
}