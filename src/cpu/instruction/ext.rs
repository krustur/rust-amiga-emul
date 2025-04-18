use super::{GetDisassemblyResult, GetDisassemblyResultError, Instruction, StepError};
use crate::cpu::StatusRegisterResult;
use crate::register::{STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};
// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

enum ExtMode {
    ByteToWord,
    WordToLong,
    // ByteToLong, // 020+
}

fn get_ext_mode(instr_word: u16) -> Option<ExtMode> {
    let opmode = (instr_word >> 6) & 0b111;
    match opmode {
        0b010 => Some(ExtMode::ByteToWord),
        0b011 => Some(ExtMode::WordToLong),
        // 020+
        // 0b111 => Some(ExgMode::ByteToLong),
        _ => None,
    }
}

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let extmode = get_ext_mode(instr_word);
            match extmode {
                Some(_) => true,
                None => false,
            }
        }
        false => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    // todo!();

    let extmode = get_ext_mode(instr_word).unwrap();
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    let status_register= match extmode {
        ExtMode::ByteToWord => {
            let value = reg.get_d_reg_byte(register, step_log);
            let value_word = Cpu::sign_extend_byte(value);
            reg.set_d_reg_word(step_log, register, value_word);
            match value_word {
                0x0000 => STATUS_REGISTER_MASK_ZERO,
                0x8000..=0xffff => STATUS_REGISTER_MASK_NEGATIVE,
                _ => 0x0000,
            }
        }
        ExtMode::WordToLong => {
            let value = reg.get_d_reg_word(register, step_log);
            let value_long = Cpu::sign_extend_word(value);
            reg.set_d_reg_long(step_log, register, value_long);
            match value_long {
                0x00000000 => STATUS_REGISTER_MASK_ZERO,
                0x80000000..=0xffffffff => STATUS_REGISTER_MASK_NEGATIVE,
                _ => 0x0000,
            }
        }
        // 020+
        // ExtMode::ByteToLong => {
        // }
    };
    let status_register_result = StatusRegisterResult {
        status_register,
        status_register_mask: STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE,
    };
    reg.reg_sr
        .merge_status_register(step_log, status_register_result);
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let extmode = get_ext_mode(instr_word).unwrap();
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    match extmode {
        ExtMode::ByteToWord => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("EXT.W"),
            format!("D{}", register),
        )),
        ExtMode::WordToLong => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("EXT.L"),
            format!("D{}", register),
        )),
        // 020+
        // ExtMode::ByteToLong => Ok(GetDisassemblyResult::from_pc(
        //     pc,
        //     String::from("EXTB.L"),
        //     format!("D{}", register),
        // )),
    }
}

