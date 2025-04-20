use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::step_log::StepLog,
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

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    match reg.reg_sr.is_sr_supervisor_set(step_log) {
        true => {
            let sr = reg.stack_pop_word(mem, step_log);

            reg.stack_pop_pc(mem, pc, step_log);
            reg.reg_sr.set_value(sr);

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
    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        String::from("RTE"),
        String::from(""),
    ))
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
    fn rte() {
        // arrange
        let code = [0x4e, 0x73].to_vec(); // RTE
        let mut mm = crate::tests::instr_test_setup(code, None);

        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        mm.mem.set_word_no_log(
            0x010003FA,
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        mm.mem.set_long_no_log(0x010003FC, 0x00C01248);
        mm.cpu.register.set_a_reg_long_no_log(7, 0x010003FA);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("RTE"),
                String::from(""),
                vec![0x4e73]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00C01248, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x01000400, mm.cpu.register.get_ssp_reg());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_supervisor_set_no_log());
    }

    #[test]
    fn rte_privilege_viloation() {
        // arrange
        let code = [0x4e, 0x73].to_vec(); // RTE
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
                String::from("RTE"),
                String::from(""),
                vec![0x4e73]
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
