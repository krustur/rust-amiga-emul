use crate::{cpu::instruction::{OperationSize, PcResult}, mem::Mem, register::Register};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    return InstructionExecutionResult {
        name: String::from("ADDX"),
        operands_format: String::from("operands_format"),
        comment: String::from("comment"),
        op_size: OperationSize::Long,
        pc_result: PcResult::Increment(2),
    };
}