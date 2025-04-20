use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register, RegisterType},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// TODO: bug fix, use X instead of C !!!

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_size000110_from_bit_pos_6(instr_word),
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
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };
    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();
    let status_register_result = match register_type {
        RegisterType::Data => match operation_size {
            OperationSize::Byte => {
                let value_source = reg.get_d_reg_byte(source_register_index, step_log);
                let value_dest = reg.get_d_reg_byte(destination_register_index, step_log);
                println!("value_source: ${:08X}", value_source);
                println!("value_dest: ${:08X}", value_dest);
                let result = Cpu::sub_bytes_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                println!("result: ${:08X}", result.result);
                reg.set_d_reg_byte(step_log, destination_register_index, result.result);
                result.status_register_result
            }
            OperationSize::Word => {
                let value_source = reg.get_d_reg_word(source_register_index, step_log);
                let value_dest = reg.get_d_reg_word(destination_register_index, step_log);
                let result = Cpu::sub_words_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                reg.set_d_reg_word(step_log, destination_register_index, result.result);
                result.status_register_result
            }
            OperationSize::Long => {
                let value_source = reg.get_d_reg_long(source_register_index, step_log);
                let value_dest = reg.get_d_reg_long(destination_register_index, step_log);
                let result = Cpu::sub_longs_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                reg.set_d_reg_long(step_log, destination_register_index, result.result);
                result.status_register_result
            }
        },
        RegisterType::Address => match operation_size {
            OperationSize::Byte => {
                reg.decrement_a_reg(source_register_index, step_log, operation_size);
                let source_register_value = reg.get_a_reg_long(source_register_index, step_log);
                let value_source = mem.get_byte(step_log, source_register_value);

                reg.decrement_a_reg(destination_register_index, step_log, operation_size);
                let destination_register_value =
                    reg.get_a_reg_long(destination_register_index, step_log);
                let value_dest = mem.get_byte(step_log, destination_register_value);

                let result = Cpu::sub_bytes_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_byte(step_log, destination_register_value, result.result);
                result.status_register_result
            }
            OperationSize::Word => {
                reg.decrement_a_reg(source_register_index, step_log, operation_size);
                let source_register_value = reg.get_a_reg_long(source_register_index, step_log);
                let value_source = mem.get_word(step_log, source_register_value);

                reg.decrement_a_reg(destination_register_index, step_log, operation_size);
                let destination_register_value =
                    reg.get_a_reg_long(destination_register_index, step_log);
                let value_dest = mem.get_word(step_log, destination_register_value);

                let result = Cpu::sub_words_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_word(step_log, destination_register_value, result.result);
                result.status_register_result
            }
            OperationSize::Long => {
                reg.decrement_a_reg(source_register_index, step_log, operation_size);
                let source_register_value = reg.get_a_reg_long(source_register_index, step_log);
                let value_source = mem.get_long(step_log, source_register_value);

                reg.decrement_a_reg(destination_register_index, step_log, operation_size);
                let destination_register_value =
                    reg.get_a_reg_long(destination_register_index, step_log);
                let value_dest = mem.get_long(step_log, destination_register_value);

                let result = Cpu::sub_longs_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_long(step_log, destination_register_value, result.result);
                result.status_register_result
            }
        },
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
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };

    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();
    match register_type {
        RegisterType::Data => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            format!("SUBX.{}", operation_size.get_format(),),
            format!(
                "{}{},{}{}",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        )),
        RegisterType::Address => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            format!("SUBX.{}", operation_size.get_format(),),
            format!(
                "-({}{}),-({}{})",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    // Data register byte

    #[test]
    fn subx_data_register_byte_with_carry_clear() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000020);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000030);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x10, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000020);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000030);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0f, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_carry_extend_negative() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000010);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xff, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_carry_extend_negative_test_both_carry() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000010);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xff, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_overflow() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x0000001f);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x0000008f);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x6f, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_overflow_test_both_overflow() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000090);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x7f, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000011);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000011);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1"),
                vec![0x9300]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // Data register word

    #[test]
    fn subx_data_register_word_with_carry_clear() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001010);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x3030);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00002020, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001010);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00003030);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000201f, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_carry_extend() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001010);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00000101);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000f0f0, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_carry_extend_test_both_carry() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x0000000f);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00000000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000fff0, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_overflow() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001000);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00009000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00007fff, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_overflow_zero_test_both_overflow() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001000);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00008000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00006fff, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001000);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00001001);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00001000);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00001001);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3"),
                vec![0x9742]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // Data register long

    #[test]
    fn subx_data_register_long_with_carry_clear() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10101010);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x30303030);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20202020, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10101010);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x30303031);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20202020, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_carry_extend_negative() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10101010);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x01010101);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xf0f0f0f0, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_carry_extend_negative_test_both_carry() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x0000000f);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xfffffff0, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_overflow() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10000000);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x90000000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x7fffffff, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_overflow_zero_test_both_overflow() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10000000);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x80000000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x6fffffff, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10000000);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x10000001);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x10000000);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x10000001);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5"),
                vec![0x9b84]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // Address register indirect with pre decrement byte

    #[test]
    fn subx_address_register_byte_with_carry_clear() {
        // arrange
        let code = [0x99, 0x0b, /* DC */ 0x10, 0x30].to_vec(); // SUBX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0x00C00003);
        mm.cpu.register.set_a_reg_long_no_log(4, 0x00C00004);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.B"),
                String::from("-(A3),-(A4)"),
                vec![0x990b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(3));
        assert_eq!(0x00c00003, mm.cpu.register.get_a_reg_long_no_log(4));
        assert_eq!(0x20, mm.mem.get_byte_no_log(0x00c00003));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_byte_with_carry_set() {
        // arrange
        let code = [0x99, 0x0b, /* DC */ 0x10, 0x30].to_vec(); // SUBX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0x00C00003);
        mm.cpu.register.set_a_reg_long_no_log(4, 0x00C00004);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
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
                String::from("SUBX.B"),
                String::from("-(A3),-(A4)"),
                vec![0x990b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(3));
        assert_eq!(0x00c00003, mm.cpu.register.get_a_reg_long_no_log(4));
        assert_eq!(0x1f, mm.mem.get_byte_no_log(0x00c00003));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_word_with_carry_clear() {
        // arrange
        let code = [0x9d, 0x4d, /* DC */ 0x10, 0x10, 0x30, 0x30].to_vec(); // SUBX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0x00C00004);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.W"),
                String::from("-(A5),-(A6)"),
                vec![0x9d4d]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(0x00c00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(0x2020, mm.mem.get_word_no_log(0x00c00004));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_word_with_carry_set() {
        // arrange
        let code = [0x9d, 0x4d, /* DC */ 0x10, 0x10, 0x30, 0x30].to_vec(); // SUBX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0x00C00004);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
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
                String::from("SUBX.W"),
                String::from("-(A5),-(A6)"),
                vec![0x9d4d]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(0x00c00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(0x201f, mm.mem.get_word_no_log(0x00c00004));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_long_with_carry_clear() {
        // arrange
        let code = [
            0x91, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x30, 0x30, 0x30, 0x30,
        ]
        .to_vec(); // SUBX.W -(A7),-(A0)
                   // DC.L $10101010, $30303030
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0x00C00006);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C0000a);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
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
                String::from("SUBX.L"),
                String::from("-(A7),-(A0)"),
                vec![0x918f]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0x00c00006, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(0x20202020, mm.mem.get_long_no_log(0x00c00006));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_long_with_carry_set() {
        // arrange
        let code = [
            0x91, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x30, 0x30, 0x30, 0x30,
        ]
        .to_vec(); // SUBX.W -(A7),-(A0)
                   // DC.L $10101010, $30303030
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0x00C00006);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C0000a);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
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
                String::from("SUBX.L"),
                String::from("-(A7),-(A0)"),
                vec![0x918f]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00c00002, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0x00c00006, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(0x2020201f, mm.mem.get_long_no_log(0x00c00006));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
