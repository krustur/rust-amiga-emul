use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{ConditionalTest, DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    let instr_word = pc.fetch_next_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition = Cpu::evaluate_condition(reg, &conditional_test);

    let displacement_8bit = Cpu::get_byte_from_word(instr_word);

    let result = match displacement_8bit {
        0x00 => todo!("16 bit displacement"),
        0xff => todo!("32 bit displacement"),
        _ => branch_8bit(pc, conditional_test, condition, displacement_8bit),
    };

    result
}

fn branch_8bit<'a>(
    // reg: &mut Register,
    pc: &ProgramCounter,
    conditional_test: ConditionalTest,
    condition: bool,
    displacement_8bit: u8,
) -> InstructionExecutionResult {
    let branch_to = Cpu::get_address_with_byte_displacement_sign_extended(
        pc.get_address() + 2,
        displacement_8bit,
    );

    match condition {
        true => InstructionExecutionResult::Done {
            pc_result: PcResult::Set(branch_to),
        },
        false => InstructionExecutionResult::Done {
            pc_result: PcResult::Increment,
        },
    }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // TODO: Condition codes
    let instr_word = pc.fetch_next_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);

    let displacement_8bit = Cpu::get_byte_from_word(instr_word);

    let (size_format, operands_format) = match displacement_8bit {
        0x00 => (String::from("W"), String::from("0x666")),
        0xff => (String::from("L"), String::from("0x666")),
        _ => (
            String::from("B"),
            format!(
                "${:02X} [${:08X}]",
                displacement_8bit,
                Cpu::get_address_with_byte_displacement_sign_extended(
                    pc.get_address() + 2,
                    displacement_8bit
                )
            ),
        ), //,
    };

    DisassemblyResult::from_pc(
        pc,
        format!("B{}.{}", conditional_test, size_format),
        operands_format,
    )
}

#[cfg(test)]
mod tests {
    use crate::{cpu::instruction::DisassemblyResult, register::STATUS_REGISTER_MASK_CARRY};

    #[test]
    fn step_bcc_b_when_carry_clear() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;
                                      // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCC.B"),
                String::from("$02 [$00C00004]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_bcc_b_when_carry_set() {
        // arrange
        let code = [0x64, 0x02].to_vec(); // BCC.B 2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCC.B"),
                String::from("$02 [$00C00004]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
    }
}
