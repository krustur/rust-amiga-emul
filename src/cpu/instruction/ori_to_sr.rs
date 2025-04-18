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

            let result = Cpu::or_words(immediate_data, dest);

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
    let immediate_data = format!("#${:04X}", pc.fetch_next_word(mem));

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        format!("ORI.W"),
        format!("{},SR", immediate_data),
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
    fn ori_to_sr_word_001f() {
        // arrange
        let code = [0x00, 0x7c, 0x00, 0x1F].to_vec(); // ORI.W #$001F,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ORI.W"),
                String::from("#$001F,SR")
            ),
            debug_result
        );
        // act
        println!("${:04X}", mm.cpu.register.reg_sr.get_value());

        mm.step();
        // assert
        assert_eq!(0x201F, mm.cpu.register.reg_sr.get_value());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_to_sr_word_2000() {
        // arrange
        let code = [0x00, 0x7c, 0x20, 0x00].to_vec(); // ORI.W #$2000,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x001f);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ORI.W"),
                String::from("#$2000,SR")
            ),
            debug_result
        );
        // act
        println!("${:04X}", mm.cpu.register.reg_sr.get_value());

        mm.step();
        // assert
        assert_eq!(0x201F, mm.cpu.register.reg_sr.get_value());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_to_sr_word_0010() {
        // arrange
        let code = [0x00, 0x7c, 0x00, 0x10].to_vec(); // ORI.W #$0010,SR
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ORI.W"),
                String::from("#$0010,SR")
            ),
            debug_result
        );
        // act
        println!("${:04X}", mm.cpu.register.reg_sr.get_value());

        mm.step();
        // assert
        assert_eq!(0x2010, mm.cpu.register.reg_sr.get_value());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn andi_to_sr_word_privilege_violation() {
        // arrange
        let code = [0x00, 0x7c, 0x20, 0x00].to_vec(); // ORI.W #$2000,SR
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
                0xC00004,
                String::from("ORI.W"),
                String::from("#$2000,SR")
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
