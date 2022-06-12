use crate::{
    cpu::instruction::PcResult,
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: TODO
// get_disassembly tests: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // println!("Execute addq: {:#010x} {:#06x}", instr_address, instr_word);
    InstructionExecutionResult::Done {
        // name: "ADDQ",
        // operands_format: "operands_format",
        // comment: "comment",
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment,
    }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // println!("Execute addq: {:#010x} {:#06x}", instr_address, instr_word);
    DisassemblyResult::from_pc(pc, String::from("ADDQ"), String::from("operands_format"))
}
