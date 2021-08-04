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
    // TODO: Tests
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    reg.reg_a[register] = ea;
    let instr_comment = format!("moving {:#010x} into A{}", ea, register);
    InstructionExecutionResult::Done {
        name: "LEA",
        // operands_format: &format!("{},A{}", ea_format, register),
        // comment: &instr_comment,
        op_size: OperationSize::Long,
        pc_result: PcResult::Increment(4),
    }
}

pub fn areg_direct_step_func<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea_register: usize
) -> InstructionExecutionResult {
    todo!("LEA addres register direct");
}