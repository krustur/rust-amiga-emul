use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, InstructionError, OperationSize,
    StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
};
use std::convert::TryFrom;

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_size000110_from_bit_pos_6(instr_word),
        false => false,
    }
}

enum CmpOpMode {
    CmpByte,
    CmpWord,
    CmpLong,
    CmpaWord,
    CmpaLong,
}

impl TryFrom<u16> for CmpOpMode {
    type Error = InstructionError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(CmpOpMode::CmpByte),
            0b001 => Ok(CmpOpMode::CmpWord),
            0b010 => Ok(CmpOpMode::CmpLong),
            0b011 => Ok(CmpOpMode::CmpaWord),
            0b111 => Ok(CmpOpMode::CmpaLong),
            _ => Err(InstructionError {
                details: format!("Failed to get CmpOpMode from u16 with value {}", value),
            }),
        }
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let source_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let dest_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();

    let status_register = match operation_size {
        OperationSize::Byte => {
            let mut source_address = reg.get_a_reg_long(source_register, step_log);
            let source = mem.get_byte(step_log, source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register, step_log);
            let dest = mem.get_byte(step_log, dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, dest_register, dest_address);

            let result = Cpu::sub_bytes(source, dest);
            result.status_register_result.status_register
        }
        OperationSize::Word => {
            let mut source_address = reg.get_a_reg_long(source_register, step_log);
            let source = mem.get_word(step_log, source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register, step_log);
            let dest = mem.get_word(step_log, dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, dest_register, dest_address);

            let result = Cpu::sub_words(source, dest);
            result.status_register_result.status_register
        }
        OperationSize::Long => {
            let mut source_address = reg.get_a_reg_long(source_register, step_log);
            let source = mem.get_long(step_log, source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register, step_log);
            let dest = mem.get_long(step_log, dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(step_log, dest_register, dest_address);

            let result = Cpu::sub_longs(source, dest);
            result.status_register_result.status_register
        }
    };

    let status_register_result = StatusRegisterResult {
        status_register,
        status_register_mask: STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE,
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
    let source_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let dest_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("CMPM.{}", operation_size.get_format()),
        format!("(A{})+,(A{})+", source_register, dest_register),
    ))
}

#[cfg(test)]
mod tests {
    use crate::cpu::instruction::GetDisassemblyResult;

    // cmpm byte

    #[test]
    fn cmpm_byte_equal_set_zero() {
        // arrange
        let code = [0xbd, 0x0f, /* DC */ 0x50, 0x50].to_vec(); // CMPM.B (A7)+,(A6)+
                                                               // DC.B $50, $50
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(6, 0xC00003);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.B"),
                String::from("(A7)+,(A6)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x50, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(0x50, mm.mem.get_byte_no_log(0xC00003));
        assert_eq!(0xC00003, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_not_equal_set_negative_carry() {
        // arrange
        let code = [0xbd, 0x0f, /* DC */ 0x60, 0x50].to_vec(); // CMPM.B (A7)+,(A6)+
                                                               // DC.B $60, $50
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(6, 0xC00003);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.B"),
                String::from("(A7)+,(A6)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x60, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(0x50, mm.mem.get_byte_no_log(0xC00003));
        assert_eq!(0xC00003, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_not_equal_set_overflow() {
        // arrange
        let code = [0xb1, 0x0d, /* DC */ 0x20, 0x90].to_vec(); // CMPM.B (A5)+,(A0)+
                                                               // DC.B $20, $90
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xC00003);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.B"),
                String::from("(A5)+,(A0)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(0x90, mm.mem.get_byte_no_log(0xC00003));
        assert_eq!(0xC00003, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_same_a_reg_different_address() {
        // arrange
        let code = [0xbb, 0x0d, /* DC */ 0x20, 0x30].to_vec(); // CMPM.B (A5)+,(A5)+
                                                               // DC.B $20, $20
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.B"),
                String::from("(A5)+,(A5)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(0x30, mm.mem.get_byte_no_log(0xC00003));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // cmpm word

    #[test]
    fn cmpm_word_equal_set_zero() {
        // arrange
        let code = [0xb1, 0x4d, /* DC */ 0x50, 0x22, 0x50, 0x22].to_vec(); // CMPM.W (A5)+,(A0)+
                                                                           // DC.W $5022, $5022
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xC00004);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.W"),
                String::from("(A5)+,(A0)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x5022, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(0x5022, mm.mem.get_word_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_not_equal_set_negative_carry() {
        // arrange
        let code = [0xbb, 0x48, /* DC */ 0x60, 0xff, 0x50, 0x00].to_vec(); // CMPM.W (A0)+,(A5)+
                                                                           // DC.W $60ff, $5000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00004);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.W"),
                String::from("(A0)+,(A5)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x60ff, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(0x5000, mm.mem.get_word_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_not_equal_set_overflow() {
        // arrange
        let code = [0xb9, 0x49, /* DC */ 0x20, 0x00, 0x90, 0xff].to_vec(); // CMPM.W (A1)+,(A4)+
                                                                           // DC.W $2000, $90ff
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(1, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(4, 0xC00004);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.W"),
                String::from("(A1)+,(A4)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x2000, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(0x90ff, mm.mem.get_word_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.get_a_reg_long_no_log(1));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(4));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_same_a_reg_different_address() {
        // arrange
        let code = [0xbd, 0x4e, /* DC */ 0x20, 0x20, 0x30, 0x00].to_vec(); // CMPM.W (A6)+,(A6)+
                                                                           // DC.B $2020, $3000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(6, 0xC00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.W"),
                String::from("(A6)+,(A6)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x2020, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(0x3000, mm.mem.get_word_no_log(0xC00004));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // cmpm long

    #[test]
    fn cmpm_long_equal_set_zero() {
        // arrange
        let code = [
            0xb7, 0x8a, /* DC */ 0x50, 0x22, 0x00, 0x11, 0x50, 0x22, 0x00, 0x11,
        ]
        .to_vec(); // CMPM.L (A2)+,(A3)+
                   // DC.L $50220011, $50220011
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(2, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.L"),
                String::from("(A2)+,(A3)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x50220011, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(0x50220011, mm.mem.get_long_no_log(0xC00006));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(2));
        assert_eq!(0xC0000a, mm.cpu.register.get_a_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_not_equal_set_negative_carry() {
        // arrange
        let code = [
            0xb7, 0x8a, /* DC */ 0x60, 0xff, 0xff, 0x00, 0x50, 0x00, 0x11, 0x11,
        ]
        .to_vec(); // CMPM.L (A2)+,(A3)+
                   // DC.L $60ffff00, $50001111
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(2, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.L"),
                String::from("(A2)+,(A3)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x60ffff00, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(0x50001111, mm.mem.get_long_no_log(0xC00006));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(2));
        assert_eq!(0xC0000a, mm.cpu.register.get_a_reg_long_no_log(3));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_not_equal_set_overflow() {
        // arrange
        let code = [
            0xb7, 0x8a, /* DC */ 0x20, 0x00, 0x11, 0x22, 0x90, 0xff, 0xee, 0xdd,
        ]
        .to_vec(); // CMPM.L (A2)+,(A3)+
                   // DC.L $20001122, $90ffeedd
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(2, 0xC00002);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.L"),
                String::from("(A2)+,(A3)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20001122, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(0x90ffeedd, mm.mem.get_long_no_log(0xC00006));
        assert_eq!(0xC00006, mm.cpu.register.get_a_reg_long_no_log(2));
        assert_eq!(0xC0000a, mm.cpu.register.get_a_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_same_a_reg_different_address() {
        // arrange
        let code = [
            0xbf, 0x8f, /* DC */ 0x20, 0x20, 0x40, 0x50, 0x30, 0x00, 0x01, 0x02,
        ]
        .to_vec(); // CMPM.L (A6)+,(A6)+
                   // DC.W $20204050, $30000102
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0xC00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPM.L"),
                String::from("(A7)+,(A7)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x20204050, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(0x30000102, mm.mem.get_long_no_log(0xC00006));
        assert_eq!(0xC0000a, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
