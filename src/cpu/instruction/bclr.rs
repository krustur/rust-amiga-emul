use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{ProgramCounter, Register, STATUS_REGISTER_MASK_ZERO},
};

// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word),
        false => false,
    }
}

pub fn step_dynamic<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        },
    )?;

    // Bit Number Dynamic, Specified in a Register
    let dreg = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let bit_number = match ea_data.operation_size {
        OperationSize::Long => reg.get_d_reg_byte(dreg, step_log) % 32,
        _ => reg.get_d_reg_byte(dreg, step_log) % 8,
    };

    let bit_set = match ea_data.operation_size {
        OperationSize::Long => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let bit_set = (value & bit_number_mask) != 0;
            let value = value & !bit_number_mask;
            ea_data.set_value_long(pc, reg, mem, step_log, value, true);
            bit_set
        }
        _ => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let bit_set = (value & bit_number_mask) != 0;
            let value = value & !bit_number_mask;
            ea_data.set_value_byte(pc, reg, mem, step_log, value, true);
            bit_set
        }
    };

    let zero_flag = !bit_set;
    let status_register_result = match zero_flag {
        false => StatusRegisterResult {
            status_register: 0x0000,
            status_register_mask: STATUS_REGISTER_MASK_ZERO,
        },
        true => StatusRegisterResult {
            status_register: STATUS_REGISTER_MASK_ZERO,
            status_register_mask: STATUS_REGISTER_MASK_ZERO,
        },
    };

    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

pub fn get_disassembly_dynamic<'a>(
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
        |instr_word| {
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        },
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(OperationSize::Byte), mem);

    // Bit Number Dynamic, Specified in a Register
    let dreg = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from(format!("BCLR.{}", ea_data.operation_size.get_format())),
        format!("D{},{}", dreg, ea_format.format),
    ))
}

pub fn step_static<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let operation_size = match instr_word & 0x0038 {
        0x0000 => {
            // DRegDirect
            OperationSize::Long
        }
        _ => {
            // other
            OperationSize::Byte
        }
    };
    // Bit Number Static, Specified as Immediate Data
    let bit_number = match operation_size {
        OperationSize::Long => Cpu::get_byte_from_word(pc.fetch_next_word(mem)) % 32,
        _ => Cpu::get_byte_from_word(pc.fetch_next_word(mem)) % 8,
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(operation_size),
    )?;

    let bit_set = match ea_data.operation_size {
        OperationSize::Long => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let bit_set = (value & bit_number_mask) != 0;
            let value = value & !bit_number_mask;
            ea_data.set_value_long(pc, reg, mem, step_log, value, true);
            bit_set
        }
        _ => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let bit_set = (value & bit_number_mask) != 0;
            let value = value & !bit_number_mask;
            ea_data.set_value_byte(pc, reg, mem, step_log, value, true);
            bit_set
        }
    };

    let zero_flag = !bit_set;
    let status_register_result = match zero_flag {
        false => StatusRegisterResult {
            status_register: 0x0000,
            status_register_mask: STATUS_REGISTER_MASK_ZERO,
        },
        true => StatusRegisterResult {
            status_register: STATUS_REGISTER_MASK_ZERO,
            status_register_mask: STATUS_REGISTER_MASK_ZERO,
        },
    };

    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

pub fn get_disassembly_static<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let operation_size = match instr_word & 0x0038 {
        0x0000 => {
            // DRegDirect
            OperationSize::Long
        }
        _ => {
            // other
            OperationSize::Byte
        }
    };

    // Bit Number Static, Specified as Immediate Data
    let bit_number = match operation_size {
        OperationSize::Long => pc.fetch_next_word(mem),
        _ => pc.fetch_next_word(mem),
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        },
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(OperationSize::Byte), mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from(format!("BCLR.{}", ea_data.operation_size.get_format())),
        format!("#${:02X},{}", bit_number, ea_format.format),
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

    // long (data register direct)

    #[test]
    fn bclr_long_bit_number_static_data_register_direct_bit_set() {
        // arrange
        let code = [0x08, 0x81, 0x00, 0x00].to_vec(); // BCLR.L #$00,D1
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(1, 0x00000001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BCLR.L"),
                String::from("#$00,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_long_bit_number_static_data_register_direct_bit_clear() {
        // arrange
        let code = [0x08, 0x82, 0x00, 0x21].to_vec(); // BCLR.L #$21,D2
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(2, 0x0000000d);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BCLR.L"),
                String::from("#$21,D2")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000000d, cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_long_bit_number_dynamic_data_register_direct_bit_set() {
        // arrange
        let code = [0x01, 0x83].to_vec(); // BCLR.L D0,D3
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(0, 0x00000002);
        cpu.register.set_d_reg_long_no_log(3, 0x0000fff7);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCLR.L"),
                String::from("D0,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000fff3, cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_long_bit_number_dynamic_data_data_register_direct_bit_clear() {
        // arrange
        let code = [0x0f, 0x86].to_vec(); // BCLR.L D7,D6
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(7, 0x00000003);
        cpu.register.set_d_reg_long_no_log(6, 0x0000fff7);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCLR.L"),
                String::from("D7,D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000fff7, cpu.register.get_d_reg_long_no_log(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // byte

    #[test]
    fn bclr_byte_bit_number_static_address_register_indirect_bit_set() {
        // arrange
        let code = [0x08, 0x90, 0x00, 0x08, /* DC  */ 0x01].to_vec(); // BCLR.B #$08,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(1, 0x00000001);
        cpu.register.set_a_reg_long_no_log(0, 0x00c00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BCLR.B"),
                String::from("#$08,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00c00004));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_byte_bit_number_static_address_register_indirect_bit_clear() {
        // arrange
        let code = [0x08, 0x90, 0x00, 0x09, /* DC  */ 0x01].to_vec(); // BCLR.B #$09,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(2, 0x0000fffd);
        cpu.register.set_a_reg_long_no_log(0, 0x00c00004);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BCLR.B"),
                String::from("#$09,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01, cpu.memory.get_byte_no_log(0x00c00004));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_byte_bit_number_static_address_register_indirect_with_displacement_bit_clear() {
        // arrange
        let code = [0x08, 0xa9, 0x00, 0x01, 0x00, 0x0e, /* DC  */ 0x7d].to_vec(); // BCLR.B #$01,($000E,A1)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_a_reg_long_no_log(1, 0x00BFFFF8);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BCLR.B"),
                String::from("#$01,($000E,A1) [14]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x7d, cpu.memory.get_byte_no_log(0x00c00006));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_byte_bit_number_static_address_register_indirect_with_displacement_bit_set() {
        // arrange
        let code = [0x08, 0xa9, 0x00, 0x01, 0x00, 0x0e, /* DC  */ 0x8f].to_vec(); // BCLR.B #$01,($000E,A1)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_a_reg_long_no_log(1, 0x00BFFFF8);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BCLR.B"),
                String::from("#$01,($000E,A1) [14]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x8d, cpu.memory.get_byte_no_log(0x00c00006));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_byte_bit_number_dynamic_address_register_indirect_bit_clear() {
        // arrange
        let code = [0x0b, 0x90, /* DC  */ 0x01].to_vec(); // BCLR.B D5,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(5, 0x00000001);
        cpu.register.set_a_reg_long_no_log(0, 0x00c00002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCLR.B"),
                String::from("D5,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01, cpu.memory.get_byte_no_log(0x00c00002));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn bclr_byte_bit_number_dynamic_address_register_indirect_bit_set() {
        // arrange
        let code = [0x0b, 0x90, /* DC  */ 0x01].to_vec(); // BCLR.B D5,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.set_d_reg_long_no_log(7, 0x00000000);
        cpu.register.set_a_reg_long_no_log(0, 0x00c00002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCLR.B"),
                String::from("D5,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00c00002));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
