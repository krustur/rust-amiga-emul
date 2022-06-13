use crate::{
    cpu::instruction::PcResult,
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    InstructionExecutionResult::Done {
        // name: "ADDX",
        // operands_format: "operands_format",
        // comment: "comment",
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment,
    }
}

pub fn get_debug<'a>(pc: &ProgramCounter, reg: &Register, mem: &Mem) -> DisassemblyResult {
    // println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    DisassemblyResult::from_pc(pc, String::from("ADDX"), String::from("operands_format"))
}
