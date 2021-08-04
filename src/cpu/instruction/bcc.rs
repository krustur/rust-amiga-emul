use crate::{cpu::{Cpu, instruction::{OperationSize, PcResult}}, mem::Mem, register::Register};

use super::{ConditionalTest, InstructionExecutionResult};

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition = Cpu::evaluate_condition(reg, &conditional_test);

    let displacement_8bit = (instr_word & 0x00ff) as i8;

    let result = match displacement_8bit {
        0x00 => todo!("16 bit displacement"),
        -1 => todo!("32 bit displacement"), // 0xff
        _ => branch_8bit(reg, conditional_test, condition, displacement_8bit) //,
    };

    result
}

fn branch_8bit<'a>(reg: &mut Register, conditional_test: ConditionalTest, condition: bool, displacement_8bit: i8) -> InstructionExecutionResult {
    let branch_to = Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit);

    match condition {
        true => InstructionExecutionResult::Done {
            name: &format!("B{:?}.B", conditional_test),
            // operands_format: &format!("{}", displacement_8bit),
            // comment: &format!("branching to {:#010x}", branch_to),
            op_size: OperationSize::Byte,
            pc_result: PcResult::Set(branch_to),
        },
        false => InstructionExecutionResult::Done {
            name: &format!("B{:?}.B", conditional_test),
            // operands_format: &format!("{}", displacement_8bit),
            // comment: &format!("not branching"),
            op_size: OperationSize::Byte,
            pc_result: PcResult::Increment(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::register::{STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO};

    #[test]
    fn step_bcc_b_when_carry_clear() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code);
        cpu.register.reg_sr = 0x0000;//STATUS_REGISTER_MASK_CARRY;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x080004, cpu.register.reg_pc);
    }
    
    #[test]
    fn step_bcc_b_when_carry_set() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x080002, cpu.register.reg_pc);
    }
}