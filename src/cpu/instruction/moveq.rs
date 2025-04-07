use super::{GetDisassemblyResultError, StepError};
use crate::cpu::instruction::GetDisassemblyResult;
use crate::cpu::step_log::StepLog;
use crate::cpu::StatusRegisterResult;
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_OVERFLOW,
    STATUS_REGISTER_MASK_ZERO,
};
use crate::{cpu::Cpu, register::STATUS_REGISTER_MASK_NEGATIVE};

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
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let data = Cpu::get_byte_from_word(instr_word);
    let mut status_register = 0x0000;
    match data {
        0 => status_register |= STATUS_REGISTER_MASK_ZERO,
        0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
        _ => (),
    }
    let data = Cpu::sign_extend_byte_to_long(data);
    let status_register_mask = 0xfff0;

    reg.set_d_reg_long(step_log, register, data);

    let status_register_result = StatusRegisterResult {
        status_register,
        status_register_mask: STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE,
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
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let data = Cpu::get_byte_from_word(instr_word);
    let data = Cpu::sign_extend_byte_to_long(data);
    let data_signed = Cpu::get_signed_long_from_long(data);
    let operands_format = format!("#{},D{}", data_signed, register);
    let status_register_mask = 0xfff0;

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("MOVEQ"),
        operands_format,
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
    fn moveq_positive_d0() {
        // arrange
        let code = [0x70, 0x1d].to_vec(); // MOVEQ #$1d,d0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act
        let debug_result = mm.get_next_disassembly_no_log();
        mm.step();
        // assert
        assert_eq!(0x1d, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#29,D0")
            ),
            debug_result
        );
    }

    #[test]
    fn moveq_negative_d0() {
        // arrange
        let code = [0x72, 0xff].to_vec(); // MOVEQ #-1,d1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act
        let debug_result = mm.get_next_disassembly_no_log();
        mm.step();
        // assert
        assert_eq!(0xffffffff, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#-1,D1")
            ),
            debug_result
        );
    }

    #[test]
    fn moveq_zero_d0() {
        // arrange
        let code = [0x74, 0x00].to_vec(); // MOVEQ #0,d0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act
        let debug_result = mm.get_next_disassembly_no_log();
        mm.step();
        // assert
        assert_eq!(0, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#0,D2")
            ),
            debug_result
        );
    }
}
