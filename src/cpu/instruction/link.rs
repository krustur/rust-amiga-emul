use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let register_value = reg.get_a_reg_long(register, step_log);
    // println!(
    //     "link - A{}=${:08X} - SP=${:08x}",
    //     register,
    //     register_value,
    //     reg.get_a_reg_long(7)
    // );

    reg.stack_push_long(mem, step_log, register_value);
    let sp = reg.get_a_reg_long(7, step_log);
    reg.set_a_reg_long(step_log, register, sp);

    let displacement = Cpu::sign_extend_word(pc.fetch_next_word(mem));

    let new_sp = reg.get_a_reg_long(7, step_log).wrapping_add(displacement);
    reg.set_a_reg_long(step_log, 7, new_sp);

    Ok(())
}

pub fn step_long<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let register_value = reg.get_a_reg_long(register, step_log);

    reg.stack_push_long(mem, step_log, register_value);
    let sp = reg.get_a_reg_long(7, step_log);
    reg.set_a_reg_long(step_log, register, sp);

    let displacement = pc.fetch_next_long(mem);

    let new_sp = reg.get_a_reg_long(7, step_log).wrapping_add(displacement);
    reg.set_a_reg_long(step_log, 7, new_sp);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let displacement = pc.fetch_next_word(mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("LINK"),
        format!(
            "A{},#${:04X} [{}]",
            register,
            displacement,
            Cpu::get_signed_word_from_word(displacement)
        ),
    ))
}

pub fn get_disassembly_long<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let displacement = pc.fetch_next_long(mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("LINK"),
        format!(
            "A{},#${:08X} [{}]",
            register,
            displacement,
            Cpu::get_signed_long_from_long(displacement)
        ),
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
    fn link_word_a5_negative() {
        // arrange
        let code = [0x4e, 0x55, 0xff, 0xf2].to_vec(); // LINK A5,#$FFF2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long_no_log(5, 0xa5a5a5a5);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("LINK"),
                String::from("A5,#$FFF2 [-14]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00004, cpu.register.reg_pc.get_address());
        assert_eq!(0x10003ee, cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xa5a5a5a5, cpu.memory.get_long_no_log(0x10003fc));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn link_word_a5_positive() {
        // arrange
        let code = [0x4e, 0x55, 0x01, 0x02].to_vec(); // LINK A5,#$0102
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long_no_log(5, 0xa5a5a5a5);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("LINK"),
                String::from("A5,#$0102 [258]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00004, cpu.register.reg_pc.get_address());
        assert_eq!(0x10004fe, cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xa5a5a5a5, cpu.memory.get_long_no_log(0x10003fc));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn link_long_a1_negative() {
        // arrange
        let code = [0x48, 0x09, 0xff, 0xff, 0xff, 0xf2].to_vec(); // LINK A5,#$FFF2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long_no_log(1, 0xa1a1a1a1);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("LINK"),
                String::from("A1,#$FFFFFFF2 [-14]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00006, cpu.register.reg_pc.get_address());
        assert_eq!(0x10003ee, cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xa1a1a1a1, cpu.memory.get_long_no_log(0x10003fc));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn link_long_a1_positive() {
        // arrange
        let code = [0x48, 0x09, 0x00, 0x00, 0x01, 0x02].to_vec(); // LINK A5,#$0102
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long_no_log(1, 0xa1a1a1a1);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("LINK"),
                String::from("A1,#$00000102 [258]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00c00006, cpu.register.reg_pc.get_address());
        assert_eq!(0x10004fe, cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(0xa1a1a1a1, cpu.memory.get_long_no_log(0x10003fc));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }
}
