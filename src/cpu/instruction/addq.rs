use crate::{cpu::instruction::{PcResult}, mem::Mem, register::Register};

use super::{DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: TODO
// get_disassembly tests: TODO

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // println!("Execute addq: {:#010x} {:#06x}", instr_address, instr_word);
    return InstructionExecutionResult::Done {
        // name: "ADDQ",
        // operands_format: "operands_format",
        // comment: "comment",
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment(2),
    };
}

pub fn get_disassembly<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // println!("Execute addq: {:#010x} {:#06x}", instr_address, instr_word);
    return DisassemblyResult::Done {
        name: String::from("ADDQ"),
        operands_format: String::from("operands_format"),
        instr_address,
        next_instr_address: instr_address + 2,
    };
}