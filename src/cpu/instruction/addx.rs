use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register, RegisterType},
};

use super::{DisassemblyResult, InstructionExecutionResult, OperationSize};

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
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };
    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
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
        RegisterType::Address => {
            todo!()
        }
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);
    InstructionExecutionResult::Done {
        pc_result: PcResult::Increment,
    }
}

pub fn get_debug<'a>(pc: &mut ProgramCounter, reg: &Register, mem: &Mem) -> DisassemblyResult {
    let instr_word = pc.fetch_next_word(mem);
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };
    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    match register_type {
        RegisterType::Data => DisassemblyResult::from_pc(
            pc,
            format!("ADDX.{}", operation_size.get_format(),),
            format!(
                "{}{},{}{}",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        ),
        RegisterType::Address => DisassemblyResult::from_pc(
            pc,
            format!("ADDX.{}", operation_size.get_format(),),
            format!(
                "-({}{}),-({}{}) ",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::DisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    // Data register byte

    #[test]
    fn data_register_byte_with_extend_zero() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x00000020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_byte_with_extend_one() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x00000020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_byte_with_extend_one_set_carry_extend() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x000000f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_byte_with_extend_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x0000000f;
        cpu.register.reg_d[1] = 0x000000f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_byte_with_extend_one_set_overflow_negative() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x0000007f;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_byte_with_extend_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xd3, 0x00].to_vec(); // ADDX.B D0,D1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00000010;
        cpu.register.reg_d[1] = 0x0000006f;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_zero() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x00002020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_one() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x00002020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_one_set_carry_extend() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x00001010;
        cpu.register.reg_d[3] = 0x0000f0f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x0000000f;
        cpu.register.reg_d[3] = 0x0000fff0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_one_set_overflow_negative() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x00001000;
        cpu.register.reg_d[3] = 0x00007fff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_word_with_extend_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xd7, 0x42].to_vec(); // ADDX.W D2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[2] = 0x00001000;
        cpu.register.reg_d[3] = 0x00006fff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_zero() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0x20202020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_one() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0x20202020;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_one_set_carry_extend() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x10101010;
        cpu.register.reg_d[5] = 0xf0f0f0f0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_one_set_carry_extend_zero_test_both_carry() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x0000000f;
        cpu.register.reg_d[5] = 0xfffffff0;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_one_set_overflow_negative() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x10000000;
        cpu.register.reg_d[5] = 0x7fffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
    fn data_register_long_with_extend_one_set_overflow_negative_zero_test_both_overflow() {
        // arrange
        let code = [0xdb, 0x84].to_vec(); // ADDX.L D4,D5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[4] = 0x10000000;
        cpu.register.reg_d[5] = 0x6fffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
}
