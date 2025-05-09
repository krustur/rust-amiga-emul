use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
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

    match ea_data.operation_size {
        OperationSize::Byte => ea_data.set_value_byte(pc, reg, mem, step_log, 0x00, true),
        OperationSize::Word => ea_data.set_value_word(pc, reg, mem, step_log, 0x0000, true),
        OperationSize::Long => ea_data.set_value_long(pc, reg, mem, step_log, 0x00000000, true),
    };

    let status_register_result = StatusRegisterResult {
        status_register: STATUS_REGISTER_MASK_ZERO,
        status_register_mask: STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_CARRY,
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
        mem,
        String::from(format!("CLR.{}", ea_data.operation_size.get_format())),
        ea_format.format,
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

    #[test]
    fn clr_byte_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0x2b, 0x00, 0x0a, /* DC */ 0x88].to_vec(); // CLR.B ($000A,A3)
                                                                     // DC.B $88
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xBFFFFA);
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
                0xC00004,
                String::from("CLR.B"),
                String::from("($000A,A3)"),
                vec![0x422b, 0x000a]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.mem.get_byte_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.reg_pc.get_address());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn clr_word_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0x6b, 0x00, 0x0a, /* DC */ 0x88, 0x77].to_vec(); // CLR.W ($000A,A3)
                                                                           // DC.W $8877
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xBFFFFA);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CLR.W"),
                String::from("($000A,A3)"),
                vec![0x426b, 0x000a]
            ),
            debug_result
        );
        // act
        mm.step();
        // asser00t
        assert_eq!(0x00, mm.mem.get_word_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.reg_pc.get_address());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn clr_long_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0xab, 0x00, 0x0a, /* DC */ 0x88, 0x77, 0x99, 0x66].to_vec(); // CLR.L ($000A,A3)
                                                                                       // DC.W $88779966
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xBFFFFA);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CLR.L"),
                String::from("($000A,A3)"),
                vec![0x42ab, 0x000a]
            ),
            debug_result
        );
        // act
        mm.step();
        // asser00t
        assert_eq!(0x00000000, mm.mem.get_long_no_log(0xC00004));
        assert_eq!(0xC00004, mm.cpu.register.reg_pc.get_address());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
