use crate::{
    cpu::Cpu,
    memhandler::MemHandler,
    register::{ProgramCounter, Register, RegisterType},
};

use super::{
    GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError, StepResult,
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut MemHandler,
) -> Result<StepResult, StepError> {
    let instr_word = pc.fetch_next_word(mem);
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };
    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    let status_register_result = match register_type {
        RegisterType::Data => match operation_size {
            OperationSize::Byte => {
                let value_1 = Cpu::get_byte_from_long(reg.reg_d[source_register_index]);
                let value_2 = Cpu::get_byte_from_long(reg.reg_d[destination_register_index]);
                let result = Cpu::add_bytes_with_extend(value_1, value_2, reg.is_sr_carry_set());
                reg.reg_d[destination_register_index] =
                    Cpu::set_byte_in_long(result.result, reg.reg_d[destination_register_index]);
                result.status_register_result
            }
            OperationSize::Word => {
                let value_1 = Cpu::get_word_from_long(reg.reg_d[source_register_index]);
                let value_2 = Cpu::get_word_from_long(reg.reg_d[destination_register_index]);
                let result = Cpu::add_words_with_extend(value_1, value_2, reg.is_sr_carry_set());
                reg.reg_d[destination_register_index] =
                    Cpu::set_word_in_long(result.result, reg.reg_d[destination_register_index]);
                result.status_register_result
            }
            OperationSize::Long => {
                let value_1 = reg.reg_d[source_register_index];
                let value_2 = reg.reg_d[destination_register_index];
                let result = Cpu::add_longs_with_extend(value_1, value_2, reg.is_sr_carry_set());
                reg.reg_d[destination_register_index] = result.result;
                result.status_register_result
            }
        },
        RegisterType::Address => match operation_size {
            OperationSize::Byte => {
                reg.reg_a[source_register_index] -= operation_size.size_in_bytes();
                let value_1 = mem.get_byte(reg.reg_a[source_register_index]);

                reg.reg_a[destination_register_index] -= operation_size.size_in_bytes();
                let value_2 = mem.get_byte(reg.reg_a[destination_register_index]);

                let result = Cpu::add_bytes_with_extend(value_1, value_2, reg.is_sr_carry_set());

                mem.set_byte(reg.reg_a[destination_register_index], result.result);
                result.status_register_result
            }
            OperationSize::Word => {
                reg.reg_a[source_register_index] -= operation_size.size_in_bytes();
                let value_1 = mem.get_word(reg.reg_a[source_register_index]);

                reg.reg_a[destination_register_index] -= operation_size.size_in_bytes();
                let value_2 = mem.get_word(reg.reg_a[destination_register_index]);

                let result = Cpu::add_words_with_extend(value_1, value_2, reg.is_sr_carry_set());

                mem.set_word(reg.reg_a[destination_register_index], result.result);
                result.status_register_result
            }
            OperationSize::Long => {
                reg.reg_a[source_register_index] -= operation_size.size_in_bytes();
                let value_1 = mem.get_long(reg.reg_a[source_register_index]);

                reg.reg_a[destination_register_index] -= operation_size.size_in_bytes();
                let value_2 = mem.get_long(reg.reg_a[destination_register_index]);

                let result = Cpu::add_longs_with_extend(value_1, value_2, reg.is_sr_carry_set());

                mem.set_long(reg.reg_a[destination_register_index], result.result);
                result.status_register_result
            }
        },
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);
    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &MemHandler,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.fetch_next_word(mem);
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };
    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    match register_type {
        RegisterType::Data => Ok(GetDisassemblyResult::from_pc(
            pc,
            format!("ADDX.{}", operation_size.get_format(),),
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
            format!("ADDX.{}", operation_size.get_format(),),
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
    fn data_register_byte_with_carry_zero() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x00000020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x30, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_byte_with_carry_one() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x00000020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x31, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_byte_with_carry_one_set_carry_extend() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x000000f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01, cpu.register.reg_d[1]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_byte_with_carry_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x0000000f;
        cpu.register.reg_d[1] = 0x000000f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.register.reg_d[1]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_byte_with_carry_one_set_overflow_negative() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x0000007f;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x90, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_byte_with_carry_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x0000006f;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("D0,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    // Data register word

    #[test]
    fn data_register_word_with_carry_zero() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x00002020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x3030, cpu.register.reg_d[3]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_word_with_carry_one() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x00002020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x3031, cpu.register.reg_d[3]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_word_with_carry_one_set_carry_extend() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x0000f0f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0101, cpu.register.reg_d[3]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_word_with_carry_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x0000000f;
        cpu.register.reg_d[3] = 0x0000fff0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000, cpu.register.reg_d[3]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_word_with_carry_one_set_overflow_negative() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x00001000;
        cpu.register.reg_d[3] = 0x00007fff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x9000, cpu.register.reg_d[3]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_word_with_carry_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x00001000;
        cpu.register.reg_d[3] = 0x00006fff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("D2,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x8000, cpu.register.reg_d[3]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    // Data register long

    #[test]
    fn data_register_long_with_carry_zero() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0x20202020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x30303030, cpu.register.reg_d[5]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_long_with_carry_one() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0x20202020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x30303031, cpu.register.reg_d[5]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_long_with_carry_one_set_carry_extend() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0xf0f0f0f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01010101, cpu.register.reg_d[5]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_long_with_carry_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x0000000f;
        cpu.register.reg_d[5] = 0xfffffff0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.reg_d[5]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_long_with_carry_one_set_overflow_negative() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x10000000;
        cpu.register.reg_d[5] = 0x7fffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x90000000, cpu.register.reg_d[5]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn data_register_long_with_carry_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[4] = 0x10000000;
        cpu.register.reg_d[5] = 0x6fffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("D4,D5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80000000, cpu.register.reg_d[5]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    // Address register indirect with pre decrement byte

    #[test]
    fn address_register_byte_with_carry_zero() {
        // arrange
        let code = [0xd9, 0x0b, /* DC */ 0x10, 0x20].to_vec(); // ADDX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0x00C00003;
        cpu.register.reg_a[4] = 0x00C00004;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("-(A3),-(A4)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[3]);
        assert_eq!(0x00c00003, cpu.register.reg_a[4]);
        assert_eq!(0x30, cpu.memory.get_byte(0x00c00003));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_register_byte_with_carry_one() {
        // arrange
        let code = [0xd9, 0x0b, /* DC */ 0x10, 0x20].to_vec(); // ADDX.B -(A3),-(A4)
                                                               // DC.B $10, $20
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0x00C00003;
        cpu.register.reg_a[4] = 0x00C00004;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.B"),
                String::from("-(A3),-(A4)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[3]);
        assert_eq!(0x00c00003, cpu.register.reg_a[4]);
        assert_eq!(0x31, cpu.memory.get_byte(0x00c00003));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_register_word_with_carry_zero() {
        // arrange
        let code = [0xdd, 0x4d, /* DC */ 0x10, 0x10, 0x20, 0x20].to_vec(); // ADDX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[5] = 0x00C00004;
        cpu.register.reg_a[6] = 0x00C00006;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("-(A5),-(A6)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[5]);
        assert_eq!(0x00c00004, cpu.register.reg_a[6]);
        assert_eq!(0x3030, cpu.memory.get_word(0x00c00004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_register_word_with_carry_one() {
        // arrange
        let code = [0xdd, 0x4d, /* DC */ 0x10, 0x10, 0x20, 0x20].to_vec(); // ADDX.W -(A5),-(A6)
                                                                           // DC.W $1010, $2020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[5] = 0x00C00004;
        cpu.register.reg_a[6] = 0x00C00006;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.W"),
                String::from("-(A5),-(A6)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[5]);
        assert_eq!(0x00c00004, cpu.register.reg_a[6]);
        assert_eq!(0x3031, cpu.memory.get_word(0x00c00004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_register_long_with_carry_zero() {
        // arrange
        let code = [
            0xd1, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x20, 0x20, 0x20, 0x20,
        ]
        .to_vec(); // ADDX.W -(A7),-(A0)
                   // DC.L $10101010, $20202020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[7] = 0x00C00006;
        cpu.register.reg_a[0] = 0x00C0000a;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("-(A7),-(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[7]);
        assert_eq!(0x00c00006, cpu.register.reg_a[0]);
        assert_eq!(0x30303030, cpu.memory.get_long(0x00c00006));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_register_long_with_carry_one() {
        // arrange
        let code = [
            0xd1, 0x8f, /* DC */ 0x10, 0x10, 0x10, 0x10, 0x20, 0x20, 0x20, 0x20,
        ]
        .to_vec(); // ADDX.W -(A7),-(A0)
                   // DC.L $10101010, $20202020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[7] = 0x00C00006;
        cpu.register.reg_a[0] = 0x00C0000a;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ADDX.L"),
                String::from("-(A7),-(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00002, cpu.register.reg_a[7]);
        assert_eq!(0x00c00006, cpu.register.reg_a[0]);
        assert_eq!(0x30303031, cpu.memory.get_long(0x00c00006));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
