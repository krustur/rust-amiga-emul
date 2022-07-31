use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};

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
            let immediate_data = pc.fetch_next_word(mem);

            let dest = reg.reg_sr.get_value();

            let result = Cpu::and_words(immediate_data, dest);

            reg.reg_sr.set_value(result.result);

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
    let immediate_data = pc.fetch_next_word(mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("ANDI.W"),
        format!("#${:04X},SR", immediate_data),
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
    fn andi_to_sr_word_0000() {
        // arrange
        let code = [0x02, 0x7c, 0x00, 0x00].to_vec(); // ANDI.W #$0000,SR
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0xffff);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ANDI.W"),
                String::from("#$0000,SR")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000, cpu.register.reg_sr.get_value());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_supervisor_set_no_log());
    }

    #[test]
    fn andi_to_sr_word_ffff() {
        // arrange
        let code = [0x02, 0x7c, 0xff, 0xff].to_vec(); // ANDI.W #$FFFF,SR
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0xffff);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ANDI.W"),
                String::from("#$FFFF,SR")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff, cpu.register.reg_sr.get_value());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_supervisor_set_no_log());
    }

    #[test]
    fn andi_to_sr_word_privilege_violation() {
        // arrange
        let code = [0x02, 0x7c, 0x00, 0x00].to_vec(); // ANDI.W #$0000,SR
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_value(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        cpu.memory.set_long_no_log(0x00000020, 0x11223344);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ANDI.W"),
                String::from("#$0000,SR")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();

        // assert
        assert_eq!(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_SUPERVISOR_STATE,
            cpu.register.reg_sr.get_value()
        );
        assert_eq!(0x11223344, cpu.register.reg_pc.get_address());
        assert_eq!(0x11223344, cpu.register.reg_pc.get_address_next());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_supervisor_set_no_log());
    }
}
