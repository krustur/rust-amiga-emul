use crate::{cpu::{Cpu, instruction::{PcResult}}, mem::Mem, register::Register};

use super::{InstructionDebugResult, InstructionExecutionResult};

pub fn common_step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea: u32,
) -> InstructionExecutionResult {
    // TODO: Tests
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    reg.reg_a[register] = ea;
    // let instr_comment = format!("moving {:#010x} into A{}", ea, register);
    InstructionExecutionResult::Done {
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment(4),
    }
}

pub fn common_get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    ea_format: String,
    ea: u32,
) -> InstructionDebugResult {
    // TODO: Tests
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    // reg.reg_a[register] = ea;
    // let instr_comment = format!("moving {:#010x} into A{}", ea, register);
    InstructionDebugResult::Done {
        name: String::from("LEA"),
        operands_format: format!("{},A{}", ea_format, register),
        // comment: &instr_comment,
        // op_size: OperationSize::Long,
        next_instr_address: instr_address + 4,
    }
}

pub fn areg_direct_step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    ea_register: usize
) -> InstructionExecutionResult {
    todo!("LEA addres register direct");
}

pub fn areg_direct_get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    ea_register: usize
) -> InstructionDebugResult {
    todo!("LEA addres register direct");
}