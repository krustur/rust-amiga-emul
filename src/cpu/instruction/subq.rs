use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea_format: String,
    ea: u32,
) -> InstructionExecutionResult {
    // TODO: Tests
    // let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    int 
    reg.reg_a[register] = ea;
    let instr_comment = format!("subtracting {:#010x} into A{}", ea, register);
    InstructionExecutionResult {
        name: String::from("LEA"),
        operands_format: format!("{},A{}", ea_format, register),
        comment: instr_comment,
        op_size: OperationSize::Long,
        pc_result: PcResult::Increment(4),
    }
}