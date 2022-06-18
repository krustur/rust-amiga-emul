use crate::{mem::Mem, register::Register};

use super::{GetDisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn common_step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea: u32,
) -> InstructionExecutionResult {
    todo!("SUBQ common_exec_func");
    // TODO: Tests
    // let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    // reg.reg_a[register] = ea;
    // let instr_comment = format!("subtracting {:#010x} into A{}", ea, register);
    // InstructionExecutionResult {
    //     name: String::from("LEA"),
    //     operands_format: format!("{},A{}", ea_format, register),
    //     comment: instr_comment,
    //     op_size: OperationSize::Long,
    //     pc_result: PcResult::Increment,
    // }
}

pub fn common_get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    ea_format: String,
    ea: u32,
) -> GetDisassemblyResult {
    todo!("SUBQ common_exec_func");
    // TODO: Tests
    // let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    // reg.reg_a[register] = ea;
    // let instr_comment = format!("subtracting {:#010x} into A{}", ea, register);
    // InstructionExecutionResult {
    //     name: String::from("LEA"),
    //     operands_format: format!("{},A{}", ea_format, register),
    //     comment: instr_comment,
    //     op_size: OperationSize::Long,
    //     pc_result: PcResult::Increment,
    // }
}
