use crate::{
    cpu::{
        instruction::{InstructionDebugResult, PcResult},
        Cpu,
    },
    mem::Mem,
    register::Register,
};

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition = Cpu::evaluate_condition(reg, &conditional_test);

    let result = match condition {
        false => {
            let register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
            let reg_word =reg.reg_d[register].wrapping_sub(1) & 0xffff;
            reg.reg_d[register] = (reg.reg_d[register] & 0xffff0000) | reg_word;

            let x = match reg.reg_d[register] & 0xffff {
                0x0000ffff => {
                    println!("NOT branching {:08x}!", reg.reg_d[register] & 0xffff);
                    InstructionExecutionResult::Done {
                        // -1
                        pc_result: PcResult::Increment(4),
                    }
                }
                _ => {
                    println!("BRANCHING  {:08x}!", reg.reg_d[register] & 0xffff);
                    let displacement_16bit = mem.get_signed_word(instr_address + 2);
                    let branch_to =
                        Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, displacement_16bit);
                    InstructionExecutionResult::Done {
                        pc_result: PcResult::Set(branch_to),
                    }
                }
            };

            x
        }
        true => InstructionExecutionResult::Done {
            pc_result: PcResult::Increment(4),
        },
    };

    result
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> InstructionDebugResult {
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

    let displacement_16bit = mem.get_signed_word(instr_address + 2);

    let branch_to = Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, displacement_16bit);

    let result = InstructionDebugResult::Done {
        name: format!("DB{:?}", conditional_test),
        operands_format: format!(
            "D{},${:04X} [${:08X}]",
            register, displacement_16bit, branch_to
        ),
        next_instr_address: instr_address + 4,
    };

    result
}

#[cfg(test)]
mod tests {
    use crate::{cpu::instruction::InstructionDebugResult, register::STATUS_REGISTER_MASK_CARRY};

    // DBCC C set/reg gt 0 => decrease reg and branch (not -1)
    // DBCC C set/reg eq 0 => decrease reg and no branch (-1)
    // DBCC C clear => do nothing

    #[test]
    fn step_dbcc_when_carry_set_and_reg_greater_than_zero() {
        // arrange
        let code = [0x54, 0xc8, 0x00, 0x04].to_vec(); // DBCC D0,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0xffff0001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_instruction_debug();
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("DBCC"),
                operands_format: String::from("D0,$0004 [$00080006]"),
                next_instr_address: 0x080004
            },
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0000, cpu.register.reg_d[0]);
        assert_eq!(0x080006, cpu.register.reg_pc);
    }

    #[test]
    fn step_dbcc_when_carry_set_and_reg_equal_to_zero() {
        // arrange
        let code = [0x54, 0xc9, 0x00, 0x04].to_vec(); // DBCC D1,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[1] = 0xffff0000;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_instruction_debug();
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("DBCC"),
                operands_format: String::from("D1,$0004 [$00080006]"),
                next_instr_address: 0x080004
            },
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffffff, cpu.register.reg_d[1]);
        assert_eq!(0x080004, cpu.register.reg_pc);
    }

    #[test]
    fn step_dbcc_when_carry_clear() {
        // arrange
        let code = [0x54, 0xca, 0x00, 0x04].to_vec(); // DBCC D2,0x0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0xffff0001;
        cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;
                                      // act assert - debug
        let debug_result = cpu.get_next_instruction_debug();
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("DBCC"),
                operands_format: String::from("D2,$0004 [$00080006]"),
                next_instr_address: 0x080004
            },
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0001, cpu.register.reg_d[2]);
        assert_eq!(0x080004, cpu.register.reg_pc);
    }
}
