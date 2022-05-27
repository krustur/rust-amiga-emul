use crate::{cpu::instruction::{PcResult}, mem::Mem, register::Register};

use super::{DisassemblyResult, InstructionExecutionResult};

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    return InstructionExecutionResult::Done {
        // name: "ADDX",
        // operands_format: "operands_format",
        // comment: "comment",
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment(2),
    };
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    return DisassemblyResult::Done {
        name: String::from("ADDX"),
        operands_format: String::from("operands_format"),
        instr_address,
        next_instr_address: instr_address + 2,
    };
}