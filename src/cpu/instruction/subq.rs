use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, Instruction,
    OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{ProgramCounter, Register},
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
        true => match crate::cpu::match_check_size000110_from_bit_pos_6(instr_word) {
            true => crate::cpu::match_check_ea_only_alterable_addressing_modes_pos_0(instr_word),
            false => false,
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
    let ea_mode = ea_data.ea_mode;

    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, mem);
    let data = Cpu::extract_3_bit_data_1_to_8_from_word_at_pos(ea_data.instr_word, 9);
    let status_register_result = match ea_data.operation_size {
        OperationSize::Byte => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let result = Cpu::sub_bytes(data, ea_value);
            ea_data.set_value_byte(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Word => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, false);
            if let EffectiveAddressingMode::ARegDirect { ea_register } = ea_data.ea_mode {
                let ea_value = Cpu::sign_extend_word(ea_value);
                let result = Cpu::sub_longs(data as u32, ea_value);
                ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
                StatusRegisterResult::cleared()
            } else {
                let add_result = Cpu::sub_words(data as u16, ea_value);
                ea_data.set_value_word(pc, reg, mem, step_log, add_result.result, true);
                add_result.status_register_result
            }
        }
        OperationSize::Long => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let result = Cpu::sub_longs(data as u32, ea_value);
            ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
            if let EffectiveAddressingMode::ARegDirect { ea_register } = ea_data.ea_mode {
                StatusRegisterResult::cleared()
            } else {
                result.status_register_result
            }
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
    let ea_mode = ea_data.ea_mode;
    let ea_format = Cpu::get_ea_format(ea_mode, pc, Some(ea_data.operation_size), mem);
    let data = Cpu::extract_3_bit_data_1_to_8_from_word_at_pos(ea_data.instr_word, 9);
    match ea_data.operation_size {
        OperationSize::Byte => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUBQ.B"),
            format!("#${:X},{}", data, ea_format),
        )),
        OperationSize::Word => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUBQ.W"),
            format!("#${:X},{}", data, ea_format),
        )),
        OperationSize::Long => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUBQ.L"),
            format!("#${:X},{}", data, ea_format),
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

    #[test]
    fn subq_data_to_data_register_direct_byte() {
        // arrange
        let code = [0x5b, 0x18, /* DC */ 0x1a].to_vec(); // SUBQ.B #$5,(A0)+
                                                         // DC.B $10
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xC00002);
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
                String::from("SUBQ.B"),
                String::from("#$5,(A0)+"),
                vec![0x5b18]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x15, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_byte_overflow() {
        // arrange
        let code = [0x5b, 0x18, /* DC */ 0x81].to_vec(); // SUBQ.B #$5,(A0)+
                                                         // DC.B $7e
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xC00002);
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
                String::from("SUBQ.B"),
                String::from("#$5,(A0)+"),
                vec![0x5b18]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x7c, mm.mem.get_byte_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_word() {
        // arrange
        let code = [0x51, 0x5b, /* DC */ 0x60, 0x30].to_vec(); // SUBQ.W #$8,(A3)+
                                                               // DC.W $6020
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00002);
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
                String::from("SUBQ.W"),
                String::from("#$8,(A3)+"),
                vec![0x515b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x6028, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_word_carry() {
        // arrange
        let code = [0x57, 0x5b, /* DC */ 0x00, 0x02].to_vec(); // SUBQ.W #$3,(A3)+
                                                               // DC.W $fffe
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00002);
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
                String::from("SUBQ.W"),
                String::from("#$3,(A3)+"),
                vec![0x575b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffff, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_word_negative() {
        // arrange
        let code = [0x57, 0x5b, /* DC */ 0xff, 0xf0].to_vec(); // SUBQ.W #$3,(A3)+
                                                               // DC.W $fffe
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(3, 0xC00002);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBQ.W"),
                String::from("#$3,(A3)+"),
                vec![0x575b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffed, mm.mem.get_word_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_long() {
        // arrange
        let code = [0x53, 0x9d, /* DC */ 0x60, 0x70, 0x80, 0x20].to_vec(); // SUBQ.L #$1,(A5)+
                                                                           // DC.W $60708020
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00002);
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
                String::from("SUBQ.L"),
                String::from("#$1,(A5)+"),
                vec![0x539d]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x6070801f, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_data_register_direct_long_zero() {
        // arrange
        let code = [0x51, 0x9d, /* DC */ 0x00, 0x00, 0x00, 0x08].to_vec(); // SUBQ.L #$8,(A5)+
                                                                           // DC.W $fffffff8
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xC00002);
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
                String::from("SUBQ.L"),
                String::from("#$8,(A5)+"),
                vec![0x519d]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.mem.get_long_no_log(0xC00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_address_register_direct_word() {
        // arrange
        let code = [0x51, 0x48].to_vec(); // SUBQ.W #$8,A0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00000006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBQ.W"),
                String::from("#$8,A0"),
                vec![0x5148]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xfffffffe, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subq_data_to_address_register_direct_long() {
        // arrange
        let code = [0x51, 0x89].to_vec(); // SUBQ.L #$8,A1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(1, 0x00000006);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SUBQ.L"),
                String::from("#$8,A1"),
                vec![0x5189]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xfffffffe, mm.cpu.register.get_a_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
