use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::step_log::StepLog,
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    // todo!("RESET instruction")
    step_log.add_log_string(String::from("RESET"));
    Err(StepError::InstructionError{details: String::from("TODO: RESET instruction")})
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("RESET"),
        String::from(""),
    ))
}
