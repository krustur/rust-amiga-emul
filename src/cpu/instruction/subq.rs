use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::InstructionExecutionResult;

pub fn common_step_func<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea_format: String,
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
    //     pc_result: PcResult::Increment(4),
    // }
}

pub fn areg_direct_step_func<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea_register: usize
) -> InstructionExecutionResult {
    // TODO: Tests
    let size = Cpu::extract_size_from_bit_pos_6(instr_word);
    let size = match size {
        Some(size) => size,
        None => return InstructionExecutionResult::PassOn
    };
    todo!("SUBQ areg_direct_exec_func");
    // reg.reg_a[register] = ea;
    // let instr_comment = format!("subtracting {:#010x} into A{}", ea, register);
    // InstructionExecutionResult {
    //     name: String::from("LEA"),
    //     operands_format: format!("{},A{}", ea_format, register),
    //     comment: instr_comment,
    //     op_size: OperationSize::Long,
    //     pc_result: PcResult::Increment(4),
    // }
}