use super::{
    GetDisassemblyResult, GetDisassemblyResultError, InstructionError, OperationSize, StepError,
    StepResult,
};
use crate::{
    cpu::Cpu,
    memhandler::MemHandler,
    register::{ProgramCounter, Register, RegisterType},
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
    mem: &mut MemHandler,
) -> Result<StepResult, StepError> {
    let instr_word = pc.peek_next_word(mem);
    let operation_mode = Cpu::extract_op_mode_from_bit_pos_6_new::<CmpOpMode>(instr_word)?;
    let operation_size = match operation_mode {
        CmpOpMode::CmpByte => OperationSize::Byte,
        CmpOpMode::CmpWord => OperationSize::Word,
        CmpOpMode::CmpLong => OperationSize::Long,
        CmpOpMode::CmpaWord => OperationSize::Word,
        CmpOpMode::CmpaLong => OperationSize::Long,
    };
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        reg,
        mem,
        Some(operation_size),
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let status_register_result = match operation_mode {
        CmpOpMode::CmpByte => {
            let source = ea_data.get_value_byte(pc, reg, mem, true);
            let dest = Cpu::get_byte_from_long(reg.reg_d[register]);

            let add_result = Cpu::sub_bytes(source, dest);

            add_result.status_register_result
        }
        CmpOpMode::CmpWord => {
            let source = ea_data.get_value_word(pc, reg, mem, true);
            let dest = Cpu::get_word_from_long(reg.reg_d[register]);

            let add_result = Cpu::sub_words(source, dest);

            add_result.status_register_result
        }
        CmpOpMode::CmpLong => {
            let source = ea_data.get_value_long(pc, reg, mem, true);
            let dest = reg.reg_d[register];

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result
        }
        CmpOpMode::CmpaWord => {
            let source = Cpu::sign_extend_word(ea_data.get_value_word(pc, reg, mem, true));
            let dest = reg.reg_a[register];

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result
        }
        CmpOpMode::CmpaLong => {
            let source = ea_data.get_value_long(pc, reg, mem, true);
            let dest = reg.reg_a[register];

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result
        }
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);

    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &MemHandler,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
    let operation_mode = Cpu::extract_op_mode_from_bit_pos_6_new::<CmpOpMode>(instr_word)?;
    let (instruction_name, operation_size, register_type) = match operation_mode {
        CmpOpMode::CmpByte => ("CMP", OperationSize::Byte, RegisterType::Data),
        CmpOpMode::CmpWord => ("CMP", OperationSize::Word, RegisterType::Data),
        CmpOpMode::CmpLong => ("CMP", OperationSize::Long, RegisterType::Data),
        CmpOpMode::CmpaWord => ("CMPA", OperationSize::Word, RegisterType::Address),
        CmpOpMode::CmpaLong => ("CMPA", OperationSize::Long, RegisterType::Address),
    };
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        reg,
        mem,
        Some(operation_size),
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, reg, mem);

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
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
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
        cpu.register.reg_a[0] = 0xC00002;
        cpu.register.reg_d[7] = 0x40;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x40, cpu.register.reg_d[7]);
        assert_eq!(true, cpu.register.is_sr_negative_set());
    }

    #[test]
    fn cmp_byte_clear_negative() {
        // arrange
        let code = [0xbe, 0x10, /* DC */ 0x50].to_vec(); // CMP.B (A0),D7
                                                         // DC.B $50
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0xC00002;
        cpu.register.reg_d[7] = 0x60;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;

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
        assert_eq!(0x60, cpu.register.reg_d[7]);
        assert_eq!(false, cpu.register.is_sr_negative_set());
    }

    // cmp word

    #[test]
    fn cmp_word_set_zero() {
        // arrange
        let code = [0xbe, 0x50, /* DC */ 0x50, 0x00].to_vec(); // CMP.W (A0),D7
                                                               // DC.W $5000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0xC00002;
        cpu.register.reg_d[7] = 0x5000;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x5000, cpu.register.reg_d[7]);
        assert_eq!(true, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmp_word_clear_zero() {
        // arrange
        let code = [0xbe, 0x50, /* DC */ 0x50, 0x00].to_vec(); // CMP.W (A0),D7
                                                               // DC.W $5000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0xC00002;
        cpu.register.reg_d[7] = 0x5001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_ZERO;

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
        assert_eq!(0x5001, cpu.register.reg_d[7]);
        assert_eq!(false, cpu.register.is_sr_zero_set());
    }

    // cmp long

    #[test]
    fn cmp_long_set_overflow() {
        // arrange
        let code = [0xbc, 0xa6, /* DC */ 0x80, 0x00, 0x00, 0x00].to_vec(); // CMP.L -(A6),D6
                                                                           // DC.L $80000000
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[6] = 0xC00006;
        cpu.register.reg_d[6] = 0x10000001;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x10000001, cpu.register.reg_d[6]);
        assert_eq!(true, cpu.register.is_sr_overflow_set());
    }

    #[test]
    fn cmp_long_clear_overflow() {
        // arrange
        let code = [0xbc, 0xa6, /* DC */ 0x10, 0x00, 0x00, 0x00].to_vec(); // CMP.L -(A6),D6
                                                                           // DC.L 7FFFFFFF
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[6] = 0xC00006;
        cpu.register.reg_d[6] = 0x7fffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;

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
        assert_eq!(0x7fffffff, cpu.register.reg_d[6]);
        assert_eq!(false, cpu.register.is_sr_overflow_set());
    }

    // cmpa word

    #[test]
    fn cmpa_word_set_zero() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x5000;
        cpu.register.reg_a[6] = 0x5000;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x5000, cpu.register.reg_d[0]);
        assert_eq!(0x5000, cpu.register.reg_a[6]);
        assert_eq!(true, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmpa_word_clear_zero() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x5000;
        cpu.register.reg_a[6] = 0x4fff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_ZERO;

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
        assert_eq!(0x5000, cpu.register.reg_d[0]);
        assert_eq!(0x4fff, cpu.register.reg_a[6]);
        assert_eq!(false, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmpa_word_set_negative_use_address_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00005000;
        cpu.register.reg_a[6] = 0xffff6000;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x00005000, cpu.register.reg_d[0]);
        assert_eq!(0xffff6000, cpu.register.reg_a[6]);
        assert_eq!(true, cpu.register.is_sr_negative_set());
    }

    #[test]
    fn cmpa_word_clear_negative_use_address_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x00005000;
        cpu.register.reg_a[6] = 0x00014000;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;

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
        assert_eq!(0x00005000, cpu.register.reg_d[0]);
        assert_eq!(0x00014000, cpu.register.reg_a[6]);
        assert_eq!(false, cpu.register.is_sr_negative_set());
    }

    #[test]
    fn cmpa_word_clear_zero_dont_use_data_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0xffff6000;
        cpu.register.reg_a[6] = 0x00005000;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_ZERO;

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
        assert_eq!(0xffff6000, cpu.register.reg_d[0]);
        assert_eq!(0x00005000, cpu.register.reg_a[6]);
        assert_eq!(false, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmpa_word_set_zero_dont_use_data_reg_long() {
        // arrange
        let code = [0xbc, 0xc0].to_vec(); // CMPA.W D0,A6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x11114000;
        cpu.register.reg_a[6] = 0x00004000;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x11114000, cpu.register.reg_d[0]);
        assert_eq!(0x00004000, cpu.register.reg_a[6]);
        assert_eq!(true, cpu.register.is_sr_zero_set());
    }

    // cmp long

    #[test]
    fn cmpa_long_set_carry() {
        // arrange
        let code = [0xbb, 0xc2].to_vec(); // CMP.L D2,A5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x90000000;
        cpu.register.reg_a[5] = 0x20000000;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x90000000, cpu.register.reg_d[2]);
        assert_eq!(0x20000000, cpu.register.reg_a[5]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
    }

    #[test]
    fn cmpa_long_clear_overflow() {
        // arrange
        let code = [0xbb, 0xc2].to_vec(); // CMP.L D2,A5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0x11223344;
        cpu.register.reg_a[5] = 0x44442222;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;

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
        assert_eq!(0x11223344, cpu.register.reg_d[2]);
        assert_eq!(0x44442222, cpu.register.reg_a[5]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
    }
}
