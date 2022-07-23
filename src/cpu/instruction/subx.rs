use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register, RegisterType},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
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
                let value_source = reg.get_d_reg_byte(source_register_index);
                let value_dest = reg.get_d_reg_byte(destination_register_index);
                println!("value_source: ${:08X}", value_source);
                println!("value_dest: ${:08X}", value_dest);
                let result = Cpu::sub_bytes_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                println!("result: ${:08X}", result.result);
                reg.set_d_reg_byte(destination_register_index, result.result);
                result.status_register_result
            }
            OperationSize::Word => {
                let value_source = reg.get_d_reg_word(source_register_index);
                let value_dest = reg.get_d_reg_word(destination_register_index);
                let result = Cpu::sub_words_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                reg.set_d_reg_word(destination_register_index, result.result);
                result.status_register_result
            }
            OperationSize::Long => {
                let value_source = reg.get_d_reg_long(source_register_index);
                let value_dest = reg.get_d_reg_long(destination_register_index);
                let result = Cpu::sub_longs_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );
                reg.set_d_reg_long(destination_register_index, result.result);
                result.status_register_result
            }
        },
        RegisterType::Address => match operation_size {
            OperationSize::Byte => {
                reg.decrement_a_reg(source_register_index, operation_size);
                let value_source = mem.get_byte(reg.get_a_reg_long(source_register_index));

                reg.decrement_a_reg(destination_register_index, operation_size);
                let value_dest = mem.get_byte(reg.get_a_reg_long(destination_register_index));

                let result = Cpu::sub_bytes_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_byte(
                    reg.get_a_reg_long(destination_register_index),
                    result.result,
                );
                result.status_register_result
            }
            OperationSize::Word => {
                reg.decrement_a_reg(source_register_index, operation_size);
                let value_source = mem.get_word(reg.get_a_reg_long(source_register_index));

                reg.decrement_a_reg(destination_register_index, operation_size);
                let value_dest = mem.get_word(reg.get_a_reg_long(destination_register_index));

                let result = Cpu::sub_words_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_word(
                    reg.get_a_reg_long(destination_register_index),
                    result.result,
                );
                result.status_register_result
            }
            OperationSize::Long => {
                reg.decrement_a_reg(source_register_index, operation_size);
                let value_source = mem.get_long(reg.get_a_reg_long(source_register_index));

                reg.decrement_a_reg(destination_register_index, operation_size);
                let value_dest = mem.get_long(reg.get_a_reg_long(destination_register_index));

                let result = Cpu::sub_longs_with_extend(
                    value_source,
                    value_dest,
                    reg.reg_sr.is_sr_carry_set(),
                );

                mem.set_long(
                    reg.get_a_reg_long(destination_register_index),
                    result.result,
                );
                result.status_register_result
            }
        },
    };

    reg.reg_sr.merge_status_register(status_register_result);
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000020);
        cpu.register.set_d_reg_long(1, 0x00000030);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x10, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000020);
        cpu.register.set_d_reg_long(1, 0x00000030);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0f, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_carry_extend_negative() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000010);
        cpu.register.set_d_reg_long(1, 0x00000010);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xff, cpu.register.get_d_reg_long(1));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_carry_extend_negative_test_both_carry() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000010);
        cpu.register.set_d_reg_long(1, 0x00000010);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xff, cpu.register.get_d_reg_long(1));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_overflow() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x0000001f);
        cpu.register.set_d_reg_long(1, 0x0000008f);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x6f, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_set_overflow_test_both_overflow() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000010);
        cpu.register.set_d_reg_long(1, 0x00000090);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x7f, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000010);
        cpu.register.set_d_reg_long(1, 0x00000011);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_byte_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x93, 0x00].to_vec(); // SUBX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000010);
        cpu.register.set_d_reg_long(1, 0x00000011);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.register.get_d_reg_long(1));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // Data register word

    #[test]
    fn subx_data_register_word_with_carry_clear() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001010);
        cpu.register.set_d_reg_long(3, 0x3030);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00002020, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001010);
        cpu.register.set_d_reg_long(3, 0x00003030);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000201f, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_carry_extend() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001010);
        cpu.register.set_d_reg_long(3, 0x00000101);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000f0f0, cpu.register.get_d_reg_long(3));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_carry_extend_test_both_carry() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x0000000f);
        cpu.register.set_d_reg_long(3, 0x00000000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000fff0, cpu.register.get_d_reg_long(3));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_overflow() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001000);
        cpu.register.set_d_reg_long(3, 0x00009000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00007fff, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_set_overflow_zero_test_both_overflow() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001000);
        cpu.register.set_d_reg_long(3, 0x00008000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00006fff, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001000);
        cpu.register.set_d_reg_long(3, 0x00001001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_word_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x97, 0x42].to_vec(); // SUBX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00001000);
        cpu.register.set_d_reg_long(3, 0x00001001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // Data register long

    #[test]
    fn subx_data_register_long_with_carry_clear() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10101010);
        cpu.register.set_d_reg_long(5, 0x30303030);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20202020, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10101010);
        cpu.register.set_d_reg_long(5, 0x30303031);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20202020, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_carry_extend_negative() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10101010);
        cpu.register.set_d_reg_long(5, 0x01010101);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xf0f0f0f0, cpu.register.get_d_reg_long(5));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_carry_extend_negative_test_both_carry() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x0000000f);
        cpu.register.set_d_reg_long(5, 0x00000000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xfffffff0, cpu.register.get_d_reg_long(5));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_overflow() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10000000);
        cpu.register.set_d_reg_long(5, 0x90000000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x7fffffff, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_set_overflow_zero_test_both_overflow() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10000000);
        cpu.register.set_d_reg_long(5, 0x80000000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x6fffffff, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_leave_zero_cleared() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10000000);
        cpu.register.set_d_reg_long(5, 0x10000001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_data_register_long_with_carry_set_leave_zero_set() {
        // arrange
        let code = [0x9b, 0x84].to_vec(); // SUBX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x10000000);
        cpu.register.set_d_reg_long(5, 0x10000001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // Address register indirect with pre decrement byte

    #[test]
    fn subx_address_register_byte_with_carry_clear() {
        // arrange
        let code = [0x99, 0x0b, /* DC */ 0x10, 0x30].to_vec(); // SUBX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(3, 0x00C00003);
        cpu.register.set_a_reg_long(4, 0x00C00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("-(A3),-(A4)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(3));
        assert_eq!(0x00c00003, cpu.register.get_a_reg_long(4));
        assert_eq!(0x20, cpu.memory.get_byte(0x00c00003));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_byte_with_carry_set() {
        // arrange
        let code = [0x99, 0x0b, /* DC */ 0x10, 0x30].to_vec(); // SUBX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(3, 0x00C00003);
        cpu.register.set_a_reg_long(4, 0x00C00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.B"),
                String::from("-(A3),-(A4)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(3));
        assert_eq!(0x00c00003, cpu.register.get_a_reg_long(4));
        assert_eq!(0x1f, cpu.memory.get_byte(0x00c00003));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_word_with_carry_clear() {
        // arrange
        let code = [0x9d, 0x4d, /* DC */ 0x10, 0x10, 0x30, 0x30].to_vec(); // SUBX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0x00C00004);
        cpu.register.set_a_reg_long(6, 0x00C00006);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("-(A5),-(A6)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(5));
        assert_eq!(0x00c00004, cpu.register.get_a_reg_long(6));
        assert_eq!(0x2020, cpu.memory.get_word(0x00c00004));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_word_with_carry_set() {
        // arrange
        let code = [0x9d, 0x4d, /* DC */ 0x10, 0x10, 0x30, 0x30].to_vec(); // SUBX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0x00C00004);
        cpu.register.set_a_reg_long(6, 0x00C00006);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.W"),
                String::from("-(A5),-(A6)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(5));
        assert_eq!(0x00c00004, cpu.register.get_a_reg_long(6));
        assert_eq!(0x201f, cpu.memory.get_word(0x00c00004));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_long_with_carry_clear() {
        // arrange
        let code = [
            0x91, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x30, 0x30, 0x30, 0x30,
        ]
        .to_vec(); // SUBX.W -(A7),-(A0)
                   // DC.L $10101010, $30303030
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0x00C00006);
        cpu.register.set_a_reg_long(0, 0x00C0000a);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("-(A7),-(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00c00006, cpu.register.get_a_reg_long(0));
        assert_eq!(0x20202020, cpu.memory.get_long(0x00c00006));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subx_address_register_long_with_carry_set() {
        // arrange
        let code = [
            0x91, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x30, 0x30, 0x30, 0x30,
        ]
        .to_vec(); // SUBX.W -(A7),-(A0)
                   // DC.L $10101010, $30303030
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0x00C00006);
        cpu.register.set_a_reg_long(0, 0x00C0000a);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBX.L"),
                String::from("-(A7),-(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00c00006, cpu.register.get_a_reg_long(0));
        assert_eq!(0x2020201f, cpu.memory.get_long(0x00c00006));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
