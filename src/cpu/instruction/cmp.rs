use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, RegisterType, STATUS_REGISTER_MASK_CARRY,
        STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let operation_mode = CmpOpMode::from_u16(instr_word);
            match operation_mode {
                Some(operation_mode) => match operation_mode {
                    CmpOpMode::CmpByte | CmpOpMode::CmpWord | CmpOpMode::CmpLong => {
                        crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word)
                    }

                    CmpOpMode::CmpaWord | CmpOpMode::CmpaLong => {
                        crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word)
                    }
                },
                _ => false,
            }
        }
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

impl CmpOpMode {
    fn from_u16(value: u16) -> Option<Self> {
        match (value >> 6) & 0b111 {
            0b000 => Some(CmpOpMode::CmpByte),
            0b001 => Some(CmpOpMode::CmpWord),
            0b010 => Some(CmpOpMode::CmpLong),
            0b011 => Some(CmpOpMode::CmpaWord),
            0b111 => Some(CmpOpMode::CmpaLong),
            _ => None,
        }
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let operation_mode = CmpOpMode::from_u16(instr_word).unwrap();
    let operation_size = match operation_mode {
        CmpOpMode::CmpByte => OperationSize::Byte,
        CmpOpMode::CmpWord => OperationSize::Word,
        CmpOpMode::CmpLong => OperationSize::Long,
        CmpOpMode::CmpaWord => OperationSize::Word,
        CmpOpMode::CmpaLong => OperationSize::Long,
    };
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |_| Ok(operation_size),
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let status_register = match operation_mode {
        CmpOpMode::CmpByte => {
            let source = ea_data.get_value_byte(pc, reg, mem, true);
            let dest = reg.get_d_reg_byte(register);

            let add_result = Cpu::sub_bytes(source, dest);

            add_result.status_register_result.status_register
        }
        CmpOpMode::CmpWord => {
            let source = ea_data.get_value_word(pc, reg, mem, true);
            let dest = reg.get_d_reg_word(register);

            let add_result = Cpu::sub_words(source, dest);

            add_result.status_register_result.status_register
        }
        CmpOpMode::CmpLong => {
            let source = ea_data.get_value_long(pc, reg, mem, true);
            let dest = reg.get_d_reg_long(register);

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result.status_register
        }
        CmpOpMode::CmpaWord => {
            let source = Cpu::sign_extend_word(ea_data.get_value_word(pc, reg, mem, true));
            let dest = reg.get_a_reg_long(register);

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result.status_register
        }
        CmpOpMode::CmpaLong => {
            let source = ea_data.get_value_long(pc, reg, mem, true);
            let dest = reg.get_a_reg_long(register);

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result.status_register
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
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let operation_mode = CmpOpMode::from_u16(instr_word).unwrap();
    let (instruction_name, operation_size, register_type) = match operation_mode {
        CmpOpMode::CmpByte => ("CMP", OperationSize::Byte, RegisterType::Data),
        CmpOpMode::CmpWord => ("CMP", OperationSize::Word, RegisterType::Data),
        CmpOpMode::CmpLong => ("CMP", OperationSize::Long, RegisterType::Data),
        CmpOpMode::CmpaWord => ("CMPA", OperationSize::Word, RegisterType::Address),
        CmpOpMode::CmpaLong => ("CMPA", OperationSize::Long, RegisterType::Address),
    };
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |_| Ok(operation_size),
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("{}.{}", instruction_name, operation_size.get_format()),
        format!("{},{}{}", ea_format, register_type.get_format(), register),
    ))
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

    // cmp byte

    #[test]
    fn cmp_byte_set_negative() {
        // arrange
        let code = [0xbe, 0x10, /* DC */ 0x50].to_vec(); // CMP.B (A0),D7
                                                         // DC.B $50
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xC00002);
        cpu.register.set_d_reg_long(7, 0x40);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.B"),
                String::from("(A0),D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x50, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x40, cpu.register.get_d_reg_long(7));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmp_byte_clear_negative() {
        // arrange
        let code = [0xbe, 0x10, /* DC */ 0x50].to_vec(); // CMP.B (A0),D7
                                                         // DC.B $50
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xC00002);
        cpu.register.set_d_reg_long(7, 0x60);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_EXTEND);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.B"),
                String::from("(A0),D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x50, cpu.memory.get_byte(0xC00002));
        assert_eq!(0x60, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    // cmp word

    #[test]
    fn cmp_word_set_zero() {
        // arrange
        let code = [0xbe, 0x50, /* DC */ 0x50, 0x00].to_vec(); // CMP.W (A0),D7
                                                               // DC.W $5000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xC00002);
        cpu.register.set_d_reg_long(7, 0x5000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.W"),
                String::from("(A0),D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5000, cpu.memory.get_word(0xC00002));
        assert_eq!(0x5000, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmp_word_clear_zero() {
        // arrange
        let code = [0xbe, 0x50, /* DC */ 0x50, 0x00].to_vec(); // CMP.W (A0),D7
                                                               // DC.W $5000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xC00002);
        cpu.register.set_d_reg_long(7, 0x5001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO | STATUS_REGISTER_MASK_EXTEND);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.W"),
                String::from("(A0),D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5000, cpu.memory.get_word(0xC00002));
        assert_eq!(0x5001, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    // cmp long

    #[test]
    fn cmp_long_set_overflow() {
        // arrange
        let code = [0xbc, 0xa6, /* DC */ 0x80, 0x00, 0x00, 0x00].to_vec(); // CMP.L -(A6),D6
                                                                           // DC.L $80000000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(6, 0xC00006);
        cpu.register.set_d_reg_long(6, 0x10000001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.L"),
                String::from("-(A6),D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80000000, cpu.memory.get_long(0xC00002));
        assert_eq!(0x10000001, cpu.register.get_d_reg_long(6));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmp_long_clear_overflow() {
        // arrange
        let code = [0xbc, 0xa6, /* DC */ 0x10, 0x00, 0x00, 0x00].to_vec(); // CMP.L -(A6),D6
                                                                           // DC.L 7FFFFFFF
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(6, 0xC00006);
        cpu.register.set_d_reg_long(6, 0x7fffffff);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_OVERFLOW);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMP.L"),
                String::from("-(A6),D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x10000000, cpu.memory.get_long(0xC00002));
        assert_eq!(0x7fffffff, cpu.register.get_d_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    // cmpa word

    #[test]
    fn cmpa_word_set_zero() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x5000);
        cpu.register.set_a_reg_long(6, 0x5000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_EXTEND);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x5000, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_word_clear_zero() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x5000);
        cpu.register.set_a_reg_long(6, 0x4fff);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x4fff, cpu.register.get_a_reg_long(6));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_word_set_negative_use_address_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00005000);
        cpu.register.set_a_reg_long(6, 0xffff6000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00005000, cpu.register.get_d_reg_long(0));
        assert_eq!(0xffff6000, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_word_clear_negative_use_address_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00005000);
        cpu.register.set_a_reg_long(6, 0x00014000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_EXTEND);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00005000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x00014000, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_word_clear_zero_dont_use_data_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0xffff6000);
        cpu.register.set_a_reg_long(6, 0x00005000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff6000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x00005000, cpu.register.get_a_reg_long(6));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_word_set_zero_dont_use_data_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x11114000);
        cpu.register.set_a_reg_long(6, 0x00004000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_EXTEND);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.W"),
                String::from("D0,A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x11114000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x00004000, cpu.register.get_a_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    // cmp long

    #[test]
    fn cmpa_long_set_carry() {
        // arrange
        let code = [0xbb, 0xc2].to_vec(); // CMP.L D2,A5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x90000000);
        cpu.register.set_a_reg_long(5, 0x20000000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.L"),
                String::from("D2,A5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x90000000, cpu.register.get_d_reg_long(2));
        assert_eq!(0x20000000, cpu.register.get_a_reg_long(5));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }

    #[test]
    fn cmpa_long_clear_overflow() {
        // arrange
        let code = [0xbb, 0xc2].to_vec(); // CMP.L D2,A5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x11223344);
        cpu.register.set_a_reg_long(5, 0x44442222);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("CMPA.L"),
                String::from("D2,A5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x11223344, cpu.register.get_d_reg_long(2));
        assert_eq!(0x44442222, cpu.register.get_a_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "extend");
    }
}
