use crate::{cpu::Cpu, mem::Mem, register::Register};

use super::{DisassemblyResult, InstructionExecutionResult};

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
    //     pc_result: PcResult::Increment(4),
    // }
}

pub fn common_get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    ea_format: String,
    ea: u32,
) -> DisassemblyResult {
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

pub fn areg_direct_step<'a>(
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

pub fn areg_direct_get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    ea_register: usize
) -> DisassemblyResult {
    // TODO: Tests
    let size = Cpu::extract_size_from_bit_pos_6(instr_word);
    let size = match size {
        Some(size) => size,
        None => return DisassemblyResult::Done{
            name: String::from("SUBQ"),
            operands_format: String::from("operands"),
            instr_address,
            next_instr_address: instr_address + 4
        }
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