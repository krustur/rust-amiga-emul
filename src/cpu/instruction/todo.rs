use crate::mem::Mem;
use crate::register::Register;

use super::{InstructionDebugResult, InstructionExecutionResult};

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    todo!("{:#010x} step function not implemented", instr_word);
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> InstructionDebugResult {
    println!("{:#010x} get_debug function not implemented", instr_word);
    InstructionDebugResult::PassOn
}
