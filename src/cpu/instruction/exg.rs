use super::{GetDisassemblyResult, GetDisassemblyResultError, Instruction, StepError};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

enum ExgMode {
    DataRegisters,
    AddressRegisters,
    DataRegisterAndAddressRegister,
}

fn get_exg_mode(instr_word: u16) -> Option<ExgMode> {
    let opmode = (instr_word >> 3) & 0b11111;
    match opmode {
        0b01000 => Some(ExgMode::DataRegisters),
        0b01001 => Some(ExgMode::AddressRegisters),
        0b10001 => Some(ExgMode::DataRegisterAndAddressRegister),
        _ => None,
    }
}
pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let exgmode = get_exg_mode(instr_word);
            match exgmode {
                Some(_) => true,
                None => false,
            }
        }
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
    let exgmode = get_exg_mode(instr_word).unwrap();
    let register_x = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let register_y = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    match exgmode {
        ExgMode::AddressRegisters => {
            let tmp_x = reg.get_a_reg_long(register_x, step_log);
            let tmp_y = reg.get_a_reg_long(register_y, step_log);
            reg.set_a_reg_long(step_log, register_y, tmp_x);
            reg.set_a_reg_long(step_log, register_x, tmp_y);
        }
        ExgMode::DataRegisters => {
            let tmp_x = reg.get_d_reg_long(register_x, step_log);
            let tmp_y = reg.get_d_reg_long(register_y, step_log);
            reg.set_d_reg_long(step_log, register_y, tmp_x);
            reg.set_d_reg_long(step_log, register_x, tmp_y);
        }
        ExgMode::DataRegisterAndAddressRegister => {
            let tmp_x = reg.get_d_reg_long(register_x, step_log);
            let tmp_y = reg.get_a_reg_long(register_y, step_log);
            reg.set_a_reg_long(step_log, register_y, tmp_x);
            reg.set_d_reg_long(step_log, register_x, tmp_y);
        }
    }
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let exgmode = get_exg_mode(instr_word).unwrap();
    let register_x = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let register_y = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    match exgmode {
        ExgMode::AddressRegisters => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("EXG"),
            format!("A{},A{}", register_x, register_y),
        )),
        ExgMode::DataRegisters => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("EXG"),
            format!("D{},D{}", register_x, register_y),
        )),
        ExgMode::DataRegisterAndAddressRegister => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("EXG"),
            format!("D{},A{}", register_x, register_y),
        )),
    }
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
    fn exg_address_registers() {
        // arrange
        let code = [0xcb, 0x4e].to_vec(); // EXG A5,A6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(5, 0xa5123456);
        mm.cpu.register.set_a_reg_long_no_log(6, 0xa6789abc);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("EXG"),
                String::from("A5,A6")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xa6789abc, mm.cpu.register.get_a_reg_long_no_log(5));
        assert_eq!(0xa5123456, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn exg_data_registers() {
        // arrange
        let code = [0xc1, 0x47].to_vec(); // EXG D0,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xd0123456);
        mm.cpu.register.set_d_reg_long_no_log(7, 0xd7789abc);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("EXG"),
                String::from("D0,D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xd7789abc, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(0xd0123456, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn exg_data_and_address_registers() {
        // arrange
        let code = [0xc7, 0x8e].to_vec(); // EXG D3,A6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(3, 0xd3123456);
        mm.cpu.register.set_a_reg_long_no_log(6, 0xa6789abc);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("EXG"),
                String::from("D3,A6")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xa6789abc, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(0xd3123456, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
