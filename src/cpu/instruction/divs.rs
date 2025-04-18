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
        true => crate::cpu::match_check_ea_1011111__111_11_ea(instr_word, 3, 0),
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
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            // DIVS for 68000 is always DIVS.W long/word => word+word. DIVS.L for 020+ in own get_disassembly_long function.
            Ok(OperationSize::Word)
        },
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let source = ea_data.get_value_word(pc, reg, mem, step_log, true);
    if source == 0 {
        // division by zero
        return Err(StepError::IntegerDivideByZero);
    }
    let dest = reg.get_d_reg_long(register, step_log);
    let result = Cpu::divs_long_by_word(source, dest);


    reg.set_d_reg_long(step_log, register, result.result);

    reg.reg_sr
        .merge_status_register(step_log, result.status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            // DIVU for 68000 is always DIVU.W long/word => word+word. DIVU.L for 020+ in own get_disassembly_long function.
            Ok(OperationSize::Word)
        },
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        String::from("DIVS.W"),
        format!("{},D{}", ea_format, register),
    ))
}

