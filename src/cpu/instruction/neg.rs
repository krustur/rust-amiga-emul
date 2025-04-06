use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => match crate::cpu::match_check_size000110_from_bit_pos_6(instr_word) {
            false => false,
            true => {
                crate::cpu::match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word)
            }
        },
        false => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap()),
    )?;

    let status_register_result = match ea_data.operation_size {
        OperationSize::Byte => {
            let value = ea_data.get_value_byte(pc, reg, mem, step_log, false);

            let result = Cpu::neg_byte(value);
            ea_data.set_value_byte(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Word => {
            let value = ea_data.get_value_word(pc, reg, mem, step_log, false);

            let result = Cpu::neg_word(value);
            ea_data.set_value_word(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Long => {
            let value = ea_data.get_value_long(pc, reg, mem, step_log, false);

            let result = Cpu::neg_long(value);
            ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
    };

    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap()),
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from(format!("NEG.{}", ea_data.operation_size.get_format())),
        ea_format.format,
    ))
}

#[cfg(test)]
mod tests {

    // byte

    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn neg_byte_data_register_direct() {
        // arrange
        let code = [0x44, 0x07].to_vec(); // NEG.B D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0xffffff60);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.B"),
                String::from("D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffffffa0, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_byte_data_register_direct_zero() {
        // arrange
        let code = [0x44, 0x07].to_vec(); // NEG.B D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0xffffff00);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.B"),
                String::from("D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffffff00, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_byte_data_register_direct_overflow() {
        // arrange
        let code = [0x44, 0x07].to_vec(); // NEG.B D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0xffffff80);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.B"),
                String::from("D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffffff80, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // word

    #[test]
    fn neg_word_absolute_long_addressing_mode() {
        // arrange
        let code = [0x44, 0x79, 0x00, 0xc0, 0x00, 0x06, /* DC */ 0x55, 0x44].to_vec(); // NEG.W ($00C00006).L
                                                                                       // DC.W $5544
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("NEG.W"),
                String::from("($00C00006).L")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xaabc, mm.mem.get_word_no_log(0x00c00006));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_word_absolute_long_addressing_mode_zero() {
        // arrange
        let code = [0x44, 0x79, 0x00, 0xc0, 0x00, 0x06, /* DC */ 0x00, 0x00].to_vec(); // NEG.W ($00C00006).L
                                                                                       // DC.W $0000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("NEG.W"),
                String::from("($00C00006).L")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.mem.get_word_no_log(0x00c00006));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_word_absolute_long_addressing_mode_overflow() {
        // arrange
        let code = [0x44, 0x79, 0x00, 0xc0, 0x00, 0x06, /* DC */ 0x80, 00].to_vec(); // NEG.W ($00C00006).L
                                                                                     // DC.W $8000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("NEG.W"),
                String::from("($00C00006).L")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x8000, mm.mem.get_word_no_log(0x00c00006));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // long

    #[test]
    fn neg_long_address_register_indirect() {
        // arrange
        let code = [0x44, 0x90, /* DC */ 0x60, 0x07, 0x90, 0xaa].to_vec(); // NEG.L (A0)
                                                                           // DC.L $800790aa
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.L"),
                String::from("(A0)")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x9ff86f56, mm.mem.get_long_no_log(0x00c00002));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_long_address_register_indirect_zero() {
        // arrange
        let code = [0x44, 0x90, /* DC */ 0x00, 0x00, 0x00, 0x00].to_vec(); // NEG.L (A0)
                                                                           // DC.L $00000000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.L"),
                String::from("(A0)")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.mem.get_long_no_log(0x00c00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn neg_long_address_register_indirect_overflow() {
        // arrange
        let code = [0x44, 0x90, /* DC */ 0x80, 0x00, 0x00, 0x00].to_vec(); // NEG.L (A0)
                                                                           // DC.L $80000000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("NEG.L"),
                String::from("(A0)")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80000000, mm.mem.get_long_no_log(0x00c00002));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
