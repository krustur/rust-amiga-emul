use super::{GetDisassemblyResult, GetDisassemblyResultError, Instruction, StepError};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};

// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Tests!

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

            let opmode = (instr_word >> 3) & 0b11111;
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
) -> Result<(), StepError> {
    let exgmode = get_exg_mode(instr_word).unwrap();
    let register_x = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let register_y = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    match exgmode {
        ExgMode::AddressRegisters => {
            let tmp_x = reg.get_a_reg_long(register_x);
            let tmp_y = reg.get_a_reg_long(register_y);
            reg.set_a_reg_long(register_y, tmp_x);
            reg.set_a_reg_long(register_x, tmp_y);
        }
        ExgMode::DataRegisters => {
            let tmp_x = reg.get_d_reg_long(register_x);
            let tmp_y = reg.get_d_reg_long(register_y);
            reg.set_d_reg_long(register_y, tmp_x);
            reg.set_d_reg_long(register_x, tmp_y);
        }
        ExgMode::DataRegisterAndAddressRegister => {
            let tmp_x = reg.get_d_reg_long(register_x);
            let tmp_y = reg.get_a_reg_long(register_y);
            reg.set_a_reg_long(register_y, tmp_x);
            reg.set_d_reg_long(register_x, tmp_y);
        }
    }
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(5, 0xa5123456);
        cpu.register.set_a_reg_long(6, 0xa6789abc);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
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
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xa6789abc, cpu.register.get_a_reg_long(5));
        assert_eq!(0xa5123456, cpu.register.get_a_reg_long(6));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    // #[test]
    // fn swap_d0_zero() {
    //     // arrange
    //     let code = [0x48, 0x40].to_vec(); // SWAP D0
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_d_reg_long(0, 0x00000000);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("SWAP"),
    //             String::from("D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x00000000, cpu.register.get_d_reg_long(0));
    //     assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn swap_d0_negative() {
    //     // arrange
    //     let code = [0x48, 0x40].to_vec(); // SWAP D0
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_d_reg_long(0, 0x12908000);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("SWAP"),
    //             String::from("D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x80001290, cpu.register.get_d_reg_long(0));
    //     assert_eq!(0xC00002, cpu.register.reg_pc.get_address());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }
}
