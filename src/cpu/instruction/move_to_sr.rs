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
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_data_addressing_modes_pos_0(instr_word),
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
    match reg.reg_sr.is_sr_supervisor_set(step_log) {
        true => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(OperationSize::Word),
            )?;

            let value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            reg.reg_sr.set_value(value);

            Ok(())
        }
        false => Err(StepError::PriviliegeViolation),
    }
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
        |instr_word| Ok(OperationSize::Word),
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        String::from("MOVE.W"),
        format!("{},SR", ea_format.format),
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
    fn move_to_sr_to_data_register_direct_ff00() {
        // arrange
        let code = [0x46, 0xc0].to_vec(); // MOVE.W D0,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xffffff00);
        mm.cpu.register.reg_sr.set_value(0x2000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("D0,SR"),
                vec![0x46c0]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xff00, mm.cpu.register.reg_sr.get_value());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_to_sr_to_data_register_direct_020f() {
        // arrange
        let code = [0x46, 0xc7].to_vec(); // MOVE.W D7,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0xffff020f);
        mm.cpu.register.reg_sr.set_value(0x2000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("D7,SR"),
                vec![0x46c7]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x020f, mm.cpu.register.reg_sr.get_value());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_to_sr_privilege_violation() {
        // arrange
        let code = [0x46, 0xc7].to_vec(); // MOVE.W D7,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_value(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        mm.mem.set_long_no_log(0x00000020, 0x11223344);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("D7,SR"),
                vec![0x46c7]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x11223344, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x11223344, mm.cpu.register.reg_pc.get_address_next());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_supervisor_set_no_log());
    }
}
