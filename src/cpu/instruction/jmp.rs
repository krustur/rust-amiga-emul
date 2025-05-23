use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu},
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
        true => crate::cpu::match_check_ea_only_control_addressing_modes_pos_0(instr_word),
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
        |instr_word| Ok(OperationSize::Long),
    )?;
    let ea_address = ea_data.get_address(pc, reg, mem, step_log);
    pc.jump_long(ea_address);

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
        |instr_word| Ok(OperationSize::Long),
    )?;
    let ea_debug = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        String::from("JMP"),
        ea_debug.format,
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
    fn jmp_absolute_short_addressing_mode() {
        // arrange
        let code = [0x4e, 0xf8, 0x00, 0x04].to_vec(); // JMP ($0004).W
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("JMP"),
                String::from("($0004).W"),
                vec![0x4ef8, 0x0004]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000004, mm.cpu.register.reg_pc.get_address());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn jmp_address_register_indirect() {
        // arrange
        let code = [0x4e, 0xd4].to_vec(); // JMP (A4)
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(4, 0x11222210);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("JMP"),
                String::from("(A4)"),
                vec![0x4ed4]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x11222210, mm.cpu.register.reg_pc.get_address());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
