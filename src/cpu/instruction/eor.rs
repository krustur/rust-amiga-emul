use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
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

fn get_eor_operation_size(instr_word: u16) -> Option<OperationSize> {
    let opmode = (instr_word >> 6) & 0b111;
    match opmode {
        0b100 => Some(OperationSize::Byte),
        0b101 => Some(OperationSize::Word),
        0b110 => Some(OperationSize::Long),
        _ => None,
    }
}
pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    let exgmode = get_eor_operation_size(instr_word);
    match exgmode {
        Some(_) => crate::cpu::match_check_ea_1011111__110_00_ea(instr_word, 3, 0),
        None => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let size = match (instr_word >> 6) & 0x07 {
        0b100 => OperationSize::Byte,
        0b101 => OperationSize::Word,
        0b110 => OperationSize::Long,
        _ => panic!(),
    };
    let dst_ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(size),
    )?;

    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;

    let status_register_result = match dst_ea_data.operation_size {
        OperationSize::Byte => {
            let source = reg.get_d_reg_byte(register, step_log);
            let dest = dst_ea_data.get_value_byte(pc, reg, mem, step_log, true);
            let result = Cpu::eor_bytes(source, dest);

            dst_ea_data.set_value_byte(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Word => {
            let source = reg.get_d_reg_word(register, step_log);
            let dest = dst_ea_data.get_value_word(pc, reg, mem, step_log, true);

            let result = Cpu::eor_words(source, dest);

            dst_ea_data.set_value_word(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Long => {
            let source = reg.get_d_reg_long(register, step_log);
            let dest = dst_ea_data.get_value_long(pc, reg, mem, step_log, true);

            let result = Cpu::eor_longs(source, dest);
            dst_ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
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
    let size = match (instr_word >> 6) & 0x07 {
        0b100 => OperationSize::Byte,
        0b101 => OperationSize::Word,
        0b110 => OperationSize::Long,
        _ => panic!(),
    };
    let dst_ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(size),
    )?;
    let dst_ea_mode = dst_ea_data.ea_mode;

    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;

    let dst_ea_format = Cpu::get_ea_format(dst_ea_mode, pc, Some(dst_ea_data.operation_size), mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        format!("EOR.{}", dst_ea_data.operation_size.get_format()),
        format!("D{},{}", register, dst_ea_format),
    ))
}
