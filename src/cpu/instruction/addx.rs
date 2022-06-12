use crate::{cpu::instruction::{PcResult}, mem::Mem, register::{Register, ProgramCounter}};

use super::{DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: TODO
// get_disassembly tests: TODO

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

pub fn get_debug<'a>(
    pc: &ProgramCounter,    
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
    DisassemblyResult::from_pc(pc, String::from("ADDX"), String::from("operands_format"))
}