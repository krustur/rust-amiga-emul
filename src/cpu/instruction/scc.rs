use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word),
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
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);

    let ea_data = pc.get_effective_addressing_data_from_bit_pos(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(OperationSize::Byte),
        3,
        0,
    )?;

    match reg.reg_sr.evaluate_condition(&conditional_test) {
        true => {
            ea_data.set_value_byte(pc, reg, mem, step_log, 0xff, true);
        }
        false => {
            ea_data.set_value_byte(pc, reg, mem, step_log, 0x00, true);
        }
    };
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);

    let ea_data = pc.get_effective_addressing_data_from_bit_pos(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(OperationSize::Byte),
        3,
        0,
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        format!(
            "S{}.{}",
            conditional_test,
            ea_data.operation_size.get_format()
        ),
        format!("{}", ea_format),
    ))
}
