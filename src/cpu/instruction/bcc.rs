use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::Register,
};

use super::{ConditionalTest, InstructionDebugResult, InstructionExecutionResult};

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
        _ => branch_8bit(reg, conditional_test, condition, displacement_8bit), //,
    };

    result
}

fn branch_8bit<'a>(
    reg: &mut Register,
    conditional_test: ConditionalTest,
    condition: bool,
    displacement_8bit: i8,
) -> InstructionExecutionResult {
    let branch_to = Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit);

    match condition {
        true => InstructionExecutionResult::Done {
            pc_result: PcResult::Set(branch_to),
        },
        false => InstructionExecutionResult::Done {
            pc_result: PcResult::Increment(2),
        },
    }
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> InstructionDebugResult {
    // TODO: Condition codes
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);

    let displacement_8bit = (instr_word & 0x00ff) as i8;

    let (size_format, num_extension_words, operands_format) = match displacement_8bit {
        0x00 => (String::from("W"), 1, String::from("0x666")),
        -1 => (String::from("L"), 2, String::from("0x666")), // 0xff
        _ => (String::from("B"), 0, format!("${:02X} [${:08X}]", displacement_8bit, Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit))),  //,
    };

    InstructionDebugResult::Done {
        name: format!("B{}.{}", conditional_test, size_format),
        operands_format: operands_format,
        next_instr_address: instr_address + 2 + (num_extension_words << 1),
    }
}

#[cfg(test)]
mod tests {
    use crate::register::{
        STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    };

    #[test]
    fn step_bcc_b_when_carry_clear() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;
                                      // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x080004, cpu.register.reg_pc);
    }

    #[test]
    fn step_bcc_b_when_carry_set() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x080002, cpu.register.reg_pc);
    }
}
