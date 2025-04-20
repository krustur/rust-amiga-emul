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
    match reg.reg_sr.is_sr_supervisor_set(step_log) {
        true => {
            let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

            match instr_word & 0x0008 {
                0x0008 => {
                    let usp = reg.get_usp_reg();
                    reg.set_a_reg_long(step_log, register, usp);
                }
                _ => {
                    let a_reg = reg.get_a_reg_long(register, step_log);
                    reg.set_usp_reg(a_reg);
                }
            };

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
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    match instr_word & 0x0008 {
        0x0008 => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("MOVE.L"),
            format!("USP,A{}", register),
        )),
        _ => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("MOVE.L"),
            format!("A{},USP", register),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_SUPERVISOR_STATE,
            STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn move_usp_to_address_register() {
        // arrange
        let code = [0x4e, 0x6e].to_vec(); // MOVE.L USP,A6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_usp_reg(0x11123334);
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
                String::from("MOVE.L"),
                String::from("USP,A6"),
                vec![0x4e6e]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x11123334, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_usp_from_address_register() {
        // arrange
        let code = [0x4e, 0x61].to_vec(); // MOVE.L A1,USP
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(1, 0x11123334);
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
                String::from("MOVE.L"),
                String::from("A1,USP"),
                vec![0x4e61]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x11123334, mm.cpu.register.get_usp_reg());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_usp_from_address_privilege_violation() {
        // arrange
        let code = [0x4e, 0x61].to_vec(); // MOVE.L A1,USP
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
                String::from("MOVE.L"),
                String::from("A1,USP"),
                vec![0x4e61]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_SUPERVISOR_STATE,
            mm.cpu.register.reg_sr.get_value()
        );
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
