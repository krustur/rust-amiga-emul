use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition = Cpu::evaluate_condition(reg, &conditional_test);
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

    let displacement_16bit = mem.get_signed_word(instr_address + 2);
    // let operands_format = format!("[{:?}] {}", conditional_test, displacement_8bit);

    let branch_to = Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, displacement_16bit);

    let result = match condition {
        true => InstructionExecutionResult::Done {
            name: &format!("DB{:?}", conditional_test),
            // operands_format: &format!("D{},{}", register, displacement_16bit),
            // comment: &format!("not branching"),
            op_size: OperationSize::Word,
            pc_result: PcResult::Increment(4),
        },
        false => InstructionExecutionResult::Done {
            name: &format!("DB{:?}", conditional_test),
            // operands_format: &format!("D{},{}", register, displacement_16bit),
            // comment: &format!("branching to {}", branch_to),
            op_size: OperationSize::Word,
            pc_result: PcResult::Set(branch_to),
        }
    };

    result
}



#[cfg(test)]
mod tests {
    // use crate::register::{STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO};

    // #[test]
    // fn step_dbcc_b_when_carry_clear() {
    //     // arrange
    //     let code = [0x51, 0xc9].to_vec(); // DB cc xxx,xxx
    //     let mut cpu = crate::instr_test_setup(code);
    //     cpu.register.reg_sr = 0x0000;//STATUS_REGISTER_MASK_CARRY;
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x080004, cpu.register.reg_pc);
    // }
    
    // #[test]
    // fn step_dbcc_b_when_carry_set() {
    //     // arrange
    //     let code = [0x64, 0x02].to_vec(); // BCC.B 2
    //     let mut cpu = crate::instr_test_setup(code);
    //     cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x080002, cpu.register.reg_pc);
    // }
}