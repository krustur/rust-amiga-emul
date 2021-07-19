use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem<'a>,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let conditional_test = Cpu::extract_conditional_test(instr_word);
    // let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    // let operand = instr_bytes.read_i8().unwrap();
    // let operand_ptr = Cpu::sign_extend_i8(operand);

    let displacement_8bit = (instr_word & 0x00ff) as i8;
    let operands_format = format!("[{:?}] {}", conditional_test, displacement_8bit);

    let branch_to_address = match displacement_8bit {
        0 => todo!(),
        -1 => todo!(),
        _ => Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit),
    };
    if displacement_8bit == 0 || displacement_8bit == -1 {
        panic!("TODO: Word and Long branches")
    }
    match Cpu::evaluate_condition(reg, &conditional_test) {
        true => todo!(),
        false => InstructionExecutionResult {
            name: format!("B{:?}", conditional_test),
            operands_format: format!("{}", displacement_8bit),
            comment: format!("not branching"),
            op_size: OperationSize::Byte,
            pc_result: PcResult::Increment(2),
        },
    }
}