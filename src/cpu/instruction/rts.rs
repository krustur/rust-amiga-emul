use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::step_log::StepLog,
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
    reg.stack_pop_pc(mem, pc, step_log);
    Ok(())
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
        String::from("RTS"),
        String::from(""),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        mem::rammemory::RamMemory,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn rts_dont_set_any_sr() {
        // arrange
        let code = [0x4e, 0x75].to_vec(); // RTS
        let mem_range = RamMemory::from_bytes(0x00F80000, [0x00, 0xc0, 0x12, 0x48].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);

        let mut mm = crate::tests::instr_test_setup(code, Some(mem_ranges));
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        mm.cpu.register.set_a_reg_long_no_log(7, 0x00F80000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("RTS"),
                String::from("")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC01248, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x00F80004, mm.cpu.register.get_a_reg_long_no_log(7));

        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn rts_dont_clear_any_sr() {
        // arrange
        let code = [0x4e, 0x75].to_vec(); // RTS
        let mem_range = RamMemory::from_bytes(0x00F80000, [0x00, 0xc0, 0x12, 0x48].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);

        let mut mm = crate::tests::instr_test_setup(code, Some(mem_ranges));
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        mm.cpu.register.set_a_reg_long_no_log(7, 0x00F80000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("RTS"),
                String::from("")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC01248, mm.cpu.register.reg_pc.get_address());
        assert_eq!(0x00F80004, mm.cpu.register.get_a_reg_long_no_log(7));

        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
