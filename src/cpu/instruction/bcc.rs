use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::{ConditionalTest, InstructionExecutionResult};

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let conditional_test = Cpu::extract_conditional_test(instr_word);
    let condition = Cpu::evaluate_condition(reg, &conditional_test);

    let displacement_8bit = (instr_word & 0x00ff) as i8;
    let operands_format = format!("[{:?}] {}", conditional_test, displacement_8bit);

    let result = match displacement_8bit {
        0x00 => todo!("16 bit displacement"),
        -1 => todo!("32 bit displacement"), // 0xff
        _ => branch_8bit(reg, conditional_test, condition, displacement_8bit) //,
    };

    result
}

fn branch_8bit(reg: &mut Register, conditional_test: ConditionalTest, condition: bool, displacement_8bit: i8) -> InstructionExecutionResult {
    let a = Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit);

    match condition {
        true => todo!("8 bit branch"),
        false => InstructionExecutionResult {
            name: format!("B{:?}.B", conditional_test),
            operands_format: format!("{}", displacement_8bit as i8),
            comment: format!("not branching"),
            op_size: OperationSize::Byte,
            pc_result: PcResult::Increment(2),
        },
    }
}