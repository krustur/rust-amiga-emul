use super::{
    GetDisassemblyResult, GetDisassemblyResultError, InstructionError, OperationSize, StepError,
};
use crate::{
    cpu::{Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, RegisterType, STATUS_REGISTER_MASK_CARRY,
        STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
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
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let instr_word = pc.fetch_next_word(mem);
    let source_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let dest_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;

    let status_register = match operation_size {
        OperationSize::Byte => {
            let mut source_address = reg.get_a_reg_long(source_register);
            let source = mem.get_byte(source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register);
            let dest = mem.get_byte(dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(dest_register, dest_address);

            let result = Cpu::sub_bytes(source, dest);
            result.status_register_result.status_register
        }
        OperationSize::Word => {
            let mut source_address = reg.get_a_reg_long(source_register);
            let source = mem.get_word(source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register);
            let dest = mem.get_word(dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(dest_register, dest_address);

            let result = Cpu::sub_words(source, dest);
            result.status_register_result.status_register
        }
        OperationSize::Long => {
            let mut source_address = reg.get_a_reg_long(source_register);
            let source = mem.get_long(source_address);
            source_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(source_register, source_address);

            let mut dest_address = reg.get_a_reg_long(dest_register);
            let dest = mem.get_long(dest_address);
            dest_address += operation_size.size_in_bytes();
            reg.set_a_reg_long(dest_register, dest_address);

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

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.fetch_next_word(mem);
    let source_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let dest_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;

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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0xC00002);
        cpu.register.set_a_reg_long(6, 0xC00003);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x50, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x50, cpu.memory.get_byte(0xC00003));
        assert_eq!(0xC00003, cpu.register.get_a_reg_long(7));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_not_equal_set_negative_carry() {
        // arrange
        let code = [0xbd, 0x0f, /* DC */ 0x60, 0x50].to_vec(); // CMPM.B (A7)+,(A6)+
                                                               // DC.B $60, $50
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0xC00002);
        cpu.register.set_a_reg_long(6, 0xC00003);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x60, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x50, cpu.memory.get_byte(0xC00003));
        assert_eq!(0xC00003, cpu.register.get_a_reg_long(7));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(6));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_not_equal_set_overflow() {
        // arrange
        let code = [0xb1, 0x0d, /* DC */ 0x20, 0x90].to_vec(); // CMPM.B (A5)+,(A0)+
                                                               // DC.B $20, $90
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0xC00002);
        cpu.register.set_a_reg_long(0, 0xC00003);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x90, cpu.memory.get_byte(0xC00003));
        assert_eq!(0xC00003, cpu.register.get_a_reg_long(5));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_byte_same_a_reg_different_address() {
        // arrange
        let code = [0xbb, 0x0d, /* DC */ 0x20, 0x30].to_vec(); // CMPM.B (A5)+,(A5)+
                                                               // DC.B $20, $20
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0xC00002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x30, cpu.memory.get_byte(0xC00003));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // cmpm word

    #[test]
    fn cmpm_word_equal_set_zero() {
        // arrange
        let code = [0xb1, 0x4d, /* DC */ 0x50, 0x22, 0x50, 0x22].to_vec(); // CMPM.W (A5)+,(A0)+
                                                                           // DC.W $5022, $5022
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0xC00002);
        cpu.register.set_a_reg_long(0, 0xC00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5022, cpu.memory.get_word(0xC00002));
        assert_eq!(0x5022, cpu.memory.get_word(0xC00004));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(5));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_not_equal_set_negative_carry() {
        // arrange
        let code = [0xbb, 0x48, /* DC */ 0x60, 0xff, 0x50, 0x00].to_vec(); // CMPM.W (A0)+,(A5)+
                                                                           // DC.W $60ff, $5000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xC00002);
        cpu.register.set_a_reg_long(5, 0xC00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x60ff, cpu.memory.get_word(0xC00002));
        assert_eq!(0x5000, cpu.memory.get_word(0xC00004));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(0));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(5));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_not_equal_set_overflow() {
        // arrange
        let code = [0xb9, 0x49, /* DC */ 0x20, 0x00, 0x90, 0xff].to_vec(); // CMPM.W (A1)+,(A4)+
                                                                           // DC.W $2000, $90ff
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(1, 0xC00002);
        cpu.register.set_a_reg_long(4, 0xC00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x2000, cpu.memory.get_word(0xC00002));
        assert_eq!(0x90ff, cpu.memory.get_word(0xC00004));
        assert_eq!(0xC00004, cpu.register.get_a_reg_long(1));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(4));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_word_same_a_reg_different_address() {
        // arrange
        let code = [0xbd, 0x4e, /* DC */ 0x20, 0x20, 0x30, 0x00].to_vec(); // CMPM.W (A6)+,(A6)+
                                                                           // DC.B $2020, $3000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(6, 0xC00002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x2020, cpu.memory.get_word(0xC00002));
        assert_eq!(0x3000, cpu.memory.get_word(0xC00004));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(2, 0xC00002);
        cpu.register.set_a_reg_long(3, 0xC00006);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x50220011, cpu.memory.get_long(0xC00002));
        assert_eq!(0x50220011, cpu.memory.get_long(0xC00006));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(2));
        assert_eq!(0xC0000a, cpu.register.get_a_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_not_equal_set_negative_carry() {
        // arrange
        let code = [
            0xb7, 0x8a, /* DC */ 0x60, 0xff, 0xff, 0x00, 0x50, 0x00, 0x11, 0x11,
        ]
        .to_vec(); // CMPM.L (A2)+,(A3)+
                   // DC.L $60ffff00, $50001111
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(2, 0xC00002);
        cpu.register.set_a_reg_long(3, 0xC00006);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x60ffff00, cpu.memory.get_long(0xC00002));
        assert_eq!(0x50001111, cpu.memory.get_long(0xC00006));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(2));
        assert_eq!(0xC0000a, cpu.register.get_a_reg_long(3));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_not_equal_set_overflow() {
        // arrange
        let code = [
            0xb7, 0x8a, /* DC */ 0x20, 0x00, 0x11, 0x22, 0x90, 0xff, 0xee, 0xdd,
        ]
        .to_vec(); // CMPM.L (A2)+,(A3)+
                   // DC.L $20001122, $90ffeedd
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(2, 0xC00002);
        cpu.register.set_a_reg_long(3, 0xC00006);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20001122, cpu.memory.get_long(0xC00002));
        assert_eq!(0x90ffeedd, cpu.memory.get_long(0xC00006));
        assert_eq!(0xC00006, cpu.register.get_a_reg_long(2));
        assert_eq!(0xC0000a, cpu.register.get_a_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn cmpm_long_same_a_reg_different_address() {
        // arrange
        let code = [
            0xbf, 0x8f, /* DC */ 0x20, 0x20, 0x40, 0x50, 0x30, 0x00, 0x01, 0x02,
        ]
        .to_vec(); // CMPM.L (A6)+,(A6)+
                   // DC.W $20204050, $30000102
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0xC00002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x20204050, cpu.memory.get_long(0xC00002));
        assert_eq!(0x30000102, cpu.memory.get_long(0xC00006));
        assert_eq!(0xC0000a, cpu.register.get_a_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
