use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::{Cpu, StatusRegisterResult};
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
    STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Adjust syntax for memory to: LSd <ea>

enum LslrType {
    Register,
    Memory,
}

enum LslrDirection {
    Right,
    Left,
}

impl LslrDirection {
    pub fn get_format(&self) -> char {
        match self {
            LslrDirection::Right => 'R',
            LslrDirection::Left => 'L',
        }
    }
}

pub fn match_check_register(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_size000110_from_bit_pos_6(instr_word),
        false => false,
    }
}

pub fn match_check_memory(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_memory_alterable_addressing_modes_pos_0(instr_word),
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
    let (lslr_direction, lslr_type, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Byte,
        ),
        0b001 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Word,
        ),
        0b010 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Long,
        ),
        0b011 => (LslrDirection::Right, LslrType::Memory, OperationSize::Word),
        0b100 => (LslrDirection::Left, LslrType::Register, OperationSize::Byte),
        0b101 => (LslrDirection::Left, LslrType::Register, OperationSize::Word),
        0b110 => (LslrDirection::Left, LslrType::Register, OperationSize::Long),
        _ => (LslrDirection::Left, LslrType::Memory, OperationSize::Word),
    };

    let status_register_result = match lslr_type {
        LslrType::Register => {
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            let shift_count = match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    let shift_count = reg.get_d_reg_long(source_register, step_log) % 64;
                    shift_count
                }
                _ => {
                    let shift_count = ((instr_word & 0x0e00) >> 9).into();
                    let shift_count = match shift_count {
                        1..=7 => shift_count,
                        _ => 8,
                    };
                    shift_count
                }
            };
            match operation_size {
                OperationSize::Byte => {
                    let value = reg.get_d_reg_byte(dest_register, step_log) as u16;
                    let (result, overflow) = match lslr_direction {
                        LslrDirection::Left => {
                            let result = value.checked_shl(shift_count).unwrap_or(0);
                            let overflow = result & 0x100 != 0;
                            let result = (result & 0xff) as u8;
                            (result, overflow)
                        }
                        LslrDirection::Right => {
                            let result = (value << 1).checked_shr(shift_count).unwrap_or(0);
                            let overflow = result & 0x01 != 0;
                            let result = ((result >> 1) & 0xff) as u8;
                            (result, overflow)
                        }
                    };
                    reg.set_d_reg_byte(step_log, dest_register, result);

                    let (is_zero, is_negative) = match result {
                        0 => (true, false),
                        0x80..=0xff => (false, true),
                        _ => (false, false),
                    };

                    get_status_register(shift_count, overflow, is_zero, is_negative)
                }
                OperationSize::Word => {
                    let value = reg.get_d_reg_word(dest_register, step_log) as u32;
                    let (result, overflow) = match lslr_direction {
                        LslrDirection::Left => {
                            let result = value.checked_shl(shift_count).unwrap_or(0);
                            let overflow = result & 0x10000 != 0;
                            let result = (result & 0xffff) as u16;
                            (result, overflow)
                        }
                        LslrDirection::Right => {
                            let result = (value << 1).checked_shr(shift_count).unwrap_or(0);
                            let overflow = result & 0x0001 != 0;
                            let result = ((result >> 1) & 0xffff) as u16;
                            (result, overflow)
                        }
                    };

                    reg.set_d_reg_word(step_log, dest_register, result);

                    let (is_zero, is_negative) = match result {
                        0 => (true, false),
                        0x8000..=0xffff => (false, true),
                        _ => (false, false),
                    };

                    get_status_register(shift_count, overflow, is_zero, is_negative)
                }
                OperationSize::Long => {
                    let value = reg.get_d_reg_long(dest_register, step_log) as u64;
                    let (result, overflow) = match lslr_direction {
                        LslrDirection::Left => {
                            // println!("lslr_direction: left");
                            let result = value.checked_shl(shift_count).unwrap_or(0);
                            let overflow = result & 0x100000000 != 0;
                            let result = (result & 0xffffffff) as u32;
                            (result, overflow)
                        }
                        LslrDirection::Right => {
                            // println!("lslr_direction: right");
                            let result = (value << 1).checked_shr(shift_count).unwrap_or(0);
                            let overflow = result & 0x00000001 != 0;
                            let result = ((result >> 1) & 0xffffffff) as u32;
                            (result, overflow)
                        }
                    };

                    reg.set_d_reg_long(step_log, dest_register, result);

                    let (is_zero, is_negative) = match result {
                        0 => (true, false),
                        0x80000000..=0xffffffff => (false, true),
                        _ => (false, false),
                    };

                    get_status_register(shift_count, overflow, is_zero, is_negative)
                }
            }
        }
        LslrType::Memory => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
            )?;

            let value = ea_data.get_value_word(pc, reg, mem, step_log, false) as u32;
            let (result, overflow) = match lslr_direction {
                LslrDirection::Left => {
                    let result = value.checked_shl(1).unwrap_or(0);
                    let overflow = result & 0x10000 != 0;
                    let result = (result & 0xffff) as u16;
                    (result, overflow)
                }
                LslrDirection::Right => {
                    let result = (value << 1).checked_shr(1).unwrap_or(0);
                    let overflow = result & 0x0001 != 0;
                    let result = ((result >> 1) & 0xffff) as u16;
                    (result, overflow)
                }
            };

            ea_data.set_value_word(pc, reg, mem, step_log, result, true);

            let (is_zero, is_negative) = match result {
                0 => (true, false),
                0x8000..=0xffff => (false, true),
                _ => (false, false),
            };

            get_status_register(1, overflow, is_zero, is_negative)
        }
    };

    // println!(
    //     "status_register_result: ${:04X}-${:04X}",
    //     status_register_result.status_register, status_register_result.status_register_mask
    // );
    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

fn get_status_register(shift_count: u32, overflow: bool, is_zero: bool, is_negative: bool) -> StatusRegisterResult {
    let mut status_register = 0x0000;
    if is_zero {
        status_register |= STATUS_REGISTER_MASK_ZERO
    }
    if is_negative {
        status_register |= STATUS_REGISTER_MASK_NEGATIVE
    }

    // NOTE: A shift_count of 0 will by nature never have overflow set, so no
    //       need to have special handling for that
    match overflow {
        true =>
            status_register |=
                STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY,
        false => (),
    }
    StatusRegisterResult {
        status_register,
        status_register_mask: get_status_register_mask(shift_count),
    }
}

fn get_status_register_mask(shift_count: u32) -> u16 {
    match shift_count == 0 {
        true => {
            STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
        }
        false => {
            STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
        }
    }
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let (lslr_direction, lslr_type, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Byte,
        ),
        0b001 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Word,
        ),
        0b010 => (
            LslrDirection::Right,
            LslrType::Register,
            OperationSize::Long,
        ),
        0b011 => (LslrDirection::Right, LslrType::Memory, OperationSize::Word),
        0b100 => (LslrDirection::Left, LslrType::Register, OperationSize::Byte),
        0b101 => (LslrDirection::Left, LslrType::Register, OperationSize::Word),
        0b110 => (LslrDirection::Left, LslrType::Register, OperationSize::Long),
        _ => (LslrDirection::Left, LslrType::Memory, OperationSize::Word),
    };

    match lslr_type {
        LslrType::Register => {
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    Ok(GetDisassemblyResult::from_pc(
                        pc,
                        mem,
                        format!(
                            "LS{}.{}",
                            lslr_direction.get_format(),
                            operation_size.get_format()
                        ),
                        format!("D{},D{}", source_register, dest_register),
                    ))
                }
                _ => {
                    let count = (instr_word & 0x0e00) >> 9;
                    let count = match count {
                        1..=7 => count,
                        _ => 8,
                    };
                    Ok(GetDisassemblyResult::from_pc(
                        pc,
                        mem,
                        format!(
                            "LS{}.{}",
                            lslr_direction.get_format(),
                            operation_size.get_format()
                        ),
                        format!("#${:02X},D{}", count, dest_register),
                    ))
                }
            }
        }
        LslrType::Memory => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
            )?;
            let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                format!(
                    "LS{}.{}",
                    lslr_direction.get_format(),
                    operation_size.get_format()
                ),
                format!("#$01,{}", ea_format),
            ))
        }
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

    // lsl/lsr register by immediate / XNZC / byte/word/long

    #[test]
    fn lsl_register_by_immediate_byte() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000001);
        // mm.cpu.register.reg_sr.set_value(0x0000);
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
                String::from("LSL.B"),
                String::from("#$06,D0"),
                vec![0xed08]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_negative() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000003);
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
                String::from("LSL.B"),
                String::from("#$06,D0"),
                vec![0xed08]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x000000c0, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_zero() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000008);
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
                String::from("LSL.B"),
                String::from("#$06,D0"),
                vec![0xed08]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_extend_carry() {
        // arrange
        let code = [0xe3, 0x0f].to_vec(); // LSL.B #1,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000081);
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
                String::from("LSL.B"),
                String::from("#$01,D7"),
                vec![0xe30f]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word() {
        // arrange
        let code = [0xe3, 0x4e].to_vec(); // LSL.W #1,D6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00002001);
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
                String::from("LSL.W"),
                String::from("#$01,D6"),
                vec![0xe34e]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00004002, mm.cpu.register.get_d_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_negative() {
        // arrange
        let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000303);
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
                String::from("LSL.W"),
                String::from("#$06,D0"),
                vec![0xed48]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000c0c0, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_zero() {
        // arrange
        let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000800);
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
                String::from("LSL.W"),
                String::from("#$06,D0"),
                vec![0xed48]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_extend_carry() {
        // arrange
        let code = [0xe3, 0x4f].to_vec(); // LSL.W #1,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00008001);
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
                String::from("LSL.W"),
                String::from("#$01,D7"),
                vec![0xe34f]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long() {
        // arrange
        let code = [0xe3, 0x8e].to_vec(); // LSL.L #1,D6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x30002001);
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
                String::from("LSL.L"),
                String::from("#$01,D6"),
                vec![0xe38e]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x60004002, mm.cpu.register.get_d_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_negative() {
        // arrange
        let code = [0xed, 0x88].to_vec(); // LSL.L #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x03000303);
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
                String::from("LSL.L"),
                String::from("#$06,D0"),
                vec![0xed88]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xc000c0c0, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_zero() {
        // arrange
        let code = [0xed, 0x88].to_vec(); // LSL.L #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x08000000);
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
                String::from("LSL.L"),
                String::from("#$06,D0"),
                vec![0xed88]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_extend_carry() {
        // arrange
        let code = [0xe3, 0x8f].to_vec(); // LSL.L #1,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x80000001);
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
                String::from("LSL.L"),
                String::from("#$01,D7"),
                vec![0xe38f]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte() {
        // arrange
        let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000040);
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
                String::from("LSR.B"),
                String::from("#$02,D2"),
                vec![0xe40a]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000010, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte_zero() {
        // arrange
        let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000001);
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
                String::from("LSR.B"),
                String::from("#$02,D2"),
                vec![0xe40a]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte_extend_carry() {
        // arrange
        let code = [0xe2, 0x0a].to_vec(); // LSR.B #1,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
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
                String::from("LSR.B"),
                String::from("#$01,D2"),
                vec![0xe20a]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word() {
        // arrange
        let code = [0xe2, 0x4b].to_vec(); // LSR.W #1,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x00002002);
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
                String::from("LSR.W"),
                String::from("#$01,D3"),
                vec![0xe24b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00001001, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word_zero() {
        // arrange
        let code = [0xe0, 0x4b].to_vec(); // LSR.W #8,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x000007f);
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
                String::from("LSR.W"),
                String::from("#$08,D3"),
                vec![0xe04b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word_extend_carry() {
        // arrange
        let code = [0xe4, 0x4b].to_vec(); // LSR.W #2,D3
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(3, 0x0000fffe);
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
                String::from("LSR.W"),
                String::from("#$02,D3"),
                vec![0xe44b]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00003fff, mm.cpu.register.get_d_reg_long_no_log(3));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long() {
        // arrange
        let code = [0xe4, 0x8c].to_vec(); // LSR.L #2,D4
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x30002001);
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
                String::from("LSR.L"),
                String::from("#$02,D4"),
                vec![0xe48c]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0c000800, mm.cpu.register.get_d_reg_long_no_log(4));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long_zero() {
        // arrange
        let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x00000071);
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
                String::from("LSR.L"),
                String::from("#$08,D4"),
                vec![0xe08c]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(4));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long_extend_carry() {
        // arrange
        let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(4, 0x80000181);
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
                String::from("LSR.L"),
                String::from("#$08,D4"),
                vec![0xe08c]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00800001, mm.cpu.register.get_d_reg_long_no_log(4));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // lsl/lsr register by register / XNZC / zero => C clear/X unaffected / large shifts / byte/word/long

    #[test]
    fn lsl_register_by_register_byte() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000001);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_negative() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000003);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x000000c0, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_zero() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000005);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_extend_carry() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000001);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
        mm.cpu.register.set_d_reg_long_no_log(7, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
        mm.cpu.register.set_d_reg_long_no_log(7, 0);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_large_shift() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(7, 63);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_byte_large_shift_modulo() {
        // arrange
        let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000021);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x42);
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
                String::from("LSL.B"),
                String::from("D7,D0"),
                vec![0xef28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000084, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000101);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00004040, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_negative() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000303);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000c0c0, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_zero() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00001000);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000005);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_extend_carry() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00008081);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000001);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000102, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(6, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(6, 0);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_large_shift() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(6, 63);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_word_large_shift_modulo() {
        // arrange
        let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x44);
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
                String::from("LSL.W"),
                String::from("D6,D1"),
                vec![0xed69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000810, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x01000101);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x40004040, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_negative() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x03000303);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xc000c0c0, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_zero() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x10000000);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000005);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_extend_carry() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x80008081);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000001);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00010102, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(5, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x55555555);
        mm.cpu.register.set_d_reg_long_no_log(5, 0);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x55555555, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_large_shift() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(5, 63);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_register_long_large_shift_modulo() {
        // arrange
        let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x7fffffda);
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
                String::from("LSL.L"),
                String::from("D5,D2"),
                vec![0xebaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x04000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000080);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_zero() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000008);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000005);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_extend_carry() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x00000001);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
        mm.cpu.register.set_d_reg_long_no_log(7, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
        mm.cpu.register.set_d_reg_long_no_log(7, 0);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_large_shift() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(7, 63);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_byte_large_shift_modulo() {
        // arrange
        let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(7, 65);
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
                String::from("LSR.B"),
                String::from("D7,D0"),
                vec![0xee28]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000101);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000004, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_zero() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000008);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000005);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_extend_carry() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00008081);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00000001);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00004040, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(6, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(6, 0);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_large_shift() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(6, 63);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_word_large_shift_modulo() {
        // arrange
        let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x00008081);
        mm.cpu.register.set_d_reg_long_no_log(6, 0xffffffca);
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
                String::from("LSR.W"),
                String::from("D6,D1"),
                vec![0xec69]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000020, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x01000101);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00040004, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_zero() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000008);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000005);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_extend_carry() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x80008081);
        mm.cpu.register.set_d_reg_long_no_log(5, 0x00000001);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x40004040, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_shift_with_zero_extended_left_cleared_carry_cleared() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00005555);
        mm.cpu.register.set_d_reg_long_no_log(5, 0);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_shift_with_zero_extended_left_set_carry_cleared() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x55555555);
        mm.cpu.register.set_d_reg_long_no_log(5, 0);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x55555555, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_large_shift() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
        mm.cpu.register.set_d_reg_long_no_log(5, 63);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_register_long_large_shift_modulo() {
        // arrange
        let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x4f018081);
        mm.cpu.register.set_d_reg_long_no_log(5, 0xffffffd0);
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
                String::from("LSR.L"),
                String::from("D5,D2"),
                vec![0xeaaa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00004f01, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // lsl/lsr memory(ea) by 1 / XNZC / word

    #[test]
    fn lsl_memory_word() {
        // arrange
        let code = [0xe3, 0xe0, /* DC */ 0x11, 0x11].to_vec(); // LSL.W #1,-(A0)
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00004);
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
                String::from("LSL.W"),
                String::from("#$01,-(A0)"),
                vec![0xe3e0]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x2222, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00002, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_memory_word_negative() {
        // arrange
        let code = [0xe3, 0xe0, /* DC */ 0x41, 0x41].to_vec(); // LSL.W #1,-(A0)
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00004);
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
                String::from("LSL.W"),
                String::from("#$01,-(A0)"),
                vec![0xe3e0]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x8282, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00002, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_memory_word_zero() {
        // arrange
        let code = [0xe3, 0xe0, /* DC */ 0x00, 0x00].to_vec(); // LSL.W #1,-(A0)
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00004);
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
                String::from("LSL.W"),
                String::from("#$01,-(A0)"),
                vec![0xe3e0]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00002, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_memory_word_extend_carry() {
        // arrange
        let code = [0xe3, 0xe0, /* DC */ 0x81, 0x00].to_vec(); // LSL.W #1,-(A0)
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00004);
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
                String::from("LSL.W"),
                String::from("#$01,-(A0)"),
                vec![0xe3e0]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0200, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00002, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_memory_word() {
        // arrange
        let code = [0xe2, 0xde, /* DC */ 0x11, 0x10].to_vec(); // LSR.W #1,(A6)+
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00002);
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
                String::from("LSR.W"),
                String::from("#$01,(A6)+"),
                vec![0xe2de]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0888, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_memory_word_zero() {
        // arrange
        let code = [0xe2, 0xde, /* DC */ 0x00, 0x00].to_vec(); // LSR.W #1,(A6)+
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00002);
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
                String::from("LSR.W"),
                String::from("#$01,(A6)+"),
                vec![0xe2de]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_memory_word_extend_carry() {
        // arrange
        let code = [0xe2, 0xde, /* DC */ 0x81, 0x01].to_vec(); // LSR.W #1,(A6)+
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00002);
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
                String::from("LSR.W"),
                String::from("#$01,(A6)+"),
                vec![0xe2de]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x4080, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(0x00C00004, mm.cpu.register.get_a_reg_long_no_log(6));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}