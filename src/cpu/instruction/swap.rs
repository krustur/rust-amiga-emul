use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::{step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
};

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
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    let long = reg.get_d_reg_long(register, step_log);

    let result = ((long & 0xffff0000) >> 16) | ((long & 0x0000ffff) << 16);
    reg.set_d_reg_long(step_log, register, result);

    let mut status_register = STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW;
    match result {
        0 => status_register |= STATUS_REGISTER_MASK_ZERO,
        0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
        _ => (),
    }
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
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("SWAP"),
        format!("D{}", register),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn swap_d0() {
        // arrange
        let code = [0x48, 0x40].to_vec(); // SWAP D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long_no_log(0, 0xBBFF66AA);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_ZERO | STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SWAP"),
                String::from("D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x66AABBFF, cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn swap_d0_zero() {
        // arrange
        let code = [0x48, 0x40].to_vec(); // SWAP D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long_no_log(0, 0x00000000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SWAP"),
                String::from("D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn swap_d0_negative() {
        // arrange
        let code = [0x48, 0x40].to_vec(); // SWAP D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long_no_log(0, 0x12908000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("SWAP"),
                String::from("D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80001290, cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
