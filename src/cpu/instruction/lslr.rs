use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
use crate::cpu::{Cpu, StatusRegisterResult};
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
    STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
};
use std::panic;

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

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

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let instr_word = pc.peek_next_word(mem);
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
            pc.fetch_next_word(mem);
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    let shift_count = reg.get_d_reg_long(source_register) % 64;
                    // Ok(GetDisassemblyResult::from_pc(
                    //     pc,
                    //     format!(
                    //         "LS{}.{}",
                    //         lslr_direction.get_format(),
                    //         operation_size.get_format()
                    //     ),
                    //     format!("D{},D{}", source_register, dest_register),
                    // ))
                    todo!("reg to reg")
                }
                _ => {
                    let shift_count = ((instr_word & 0x0e00) >> 9).into();
                    let shift_count = match shift_count {
                        1..=7 => shift_count,
                        _ => 8,
                    };
                    match operation_size {
                        OperationSize::Byte => {
                            let value = reg.get_d_reg_byte(dest_register) as u16;

                            // println!("value: {}", value);
                            let (result, overflow) = match lslr_direction {
                                LslrDirection::Left => {
                                    // println!("lslr_direction: left");
                                    let result = value << shift_count;
                                    let overflow = result & 0x100 != 0;
                                    let result = (result & 0xff) as u8;
                                    (result, overflow)
                                }
                                LslrDirection::Right => {
                                    // println!("lslr_direction: right");
                                    let result = (value << 1) >> shift_count;
                                    let overflow = result & 0x01 != 0;
                                    let result = ((result >> 1) & 0xff) as u8;
                                    (result, overflow)
                                }
                            };

                            reg.set_d_reg_byte(dest_register, result);
                            let mut status_register = 0x0000;
                            match result {
                                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                                0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                                _ => (),
                            }
                            // println!("shift_count: {}", shift_count);
                            // println!("result: {}", result);
                            // println!("overflow: {}", overflow);
                            match overflow {
                                true => {
                                    status_register |=
                                        STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY
                                }
                                false => (),
                            }
                            StatusRegisterResult {
                                status_register,
                                status_register_mask: get_status_register_mask(shift_count),
                            }
                        }
                        OperationSize::Word => {
                            let value = reg.get_d_reg_word(dest_register) as u32;
                            // println!("value: {}", value);
                            let (result, overflow) = match lslr_direction {
                                LslrDirection::Left => {
                                    // println!("lslr_direction: left");
                                    let result = value << shift_count;
                                    let overflow = result & 0x10000 != 0;
                                    let result = (result & 0xffff) as u16;
                                    (result, overflow)
                                }
                                LslrDirection::Right => {
                                    // println!("lslr_direction: right");
                                    let result = (value << 1) >> shift_count;
                                    let overflow = result & 0x0001 != 0;
                                    let result = ((result >> 1) & 0xffff) as u16;
                                    (result, overflow)
                                }
                            };

                            reg.set_d_reg_word(dest_register, result);
                            let mut status_register = 0x0000;
                            match result {
                                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                                0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                                _ => (),
                            }
                            // println!("shift_count: {}", shift_count);
                            // println!("result: {}", result);
                            // println!("overflow: {}", overflow);
                            match overflow {
                                true => {
                                    status_register |=
                                        STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY
                                }
                                false => (),
                            }
                            StatusRegisterResult {
                                status_register,
                                status_register_mask: get_status_register_mask(shift_count),
                            }
                        }
                        OperationSize::Long => {
                            let value = reg.get_d_reg_long(dest_register) as u64;
                            // println!("value: {}", value);
                            let (result, overflow) = match lslr_direction {
                                LslrDirection::Left => {
                                    // println!("lslr_direction: left");
                                    let result = value << shift_count;
                                    let overflow = result & 0x100000000 != 0;
                                    let result = (result & 0xffffffff) as u32;
                                    (result, overflow)
                                }
                                LslrDirection::Right => {
                                    // println!("lslr_direction: right");
                                    let result = (value << 1) >> shift_count;
                                    let overflow = result & 0x00000001 != 0;
                                    let result = ((result >> 1) & 0xffffffff) as u32;
                                    (result, overflow)
                                }
                            };

                            reg.set_d_reg_long(dest_register, result);
                            let mut status_register = 0x0000;
                            match result {
                                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                                0x80000000..=0xffffffff => {
                                    status_register |= STATUS_REGISTER_MASK_NEGATIVE
                                }
                                _ => (),
                            }
                            // println!("shift_count: {}", shift_count);
                            // println!("result: {}", result);
                            // println!("overflow: {}", overflow);
                            match overflow {
                                true => {
                                    status_register |=
                                        STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY
                                }
                                false => (),
                            }
                            StatusRegisterResult {
                                status_register,
                                status_register_mask: get_status_register_mask(shift_count),
                            }
                        }
                    }
                }
            }
        }
        LslrType::Memory => {
            let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                reg,
                mem,
                |instr_word| Ok(operation_size),
            )?;
            let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
            // Ok(GetDisassemblyResult::from_pc(
            //     pc,
            //     format!(
            //         "LS{}.{}",
            //         lslr_direction.get_format(),
            //         operation_size.get_format()
            //     ),
            //     format!("#$01,{}", ea_format),
            // ))
            todo!("memory")
        }
    };

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())

    // todo!("lslr")
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
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
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
            pc.fetch_next_word(mem);
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    Ok(GetDisassemblyResult::from_pc(
                        pc,
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
            let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                reg,
                mem,
                |instr_word| Ok(operation_size),
            )?;
            let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
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

    // register by immediate / XNZC / byte/word/long lsl/lsr

    #[test]
    fn lsl_register_by_immediate_byte() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.B"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000040, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_negative() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000003);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.B"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x000000c0, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_zero() {
        // arrange
        let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000008);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.B"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_byte_extend_carry() {
        // arrange
        let code = [0xe3, 0x0f].to_vec(); // LSL.B #1,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0x00000081);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.B"),
                String::from("#$01,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000002, cpu.register.get_d_reg_long(7));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word() {
        // arrange
        let code = [0xe3, 0x4e].to_vec(); // LSL.W #1,D6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(6, 0x00002001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.W"),
                String::from("#$01,D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00004002, cpu.register.get_d_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_negative() {
        // arrange
        let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000303);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.W"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000c0c0, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_zero() {
        // arrange
        let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x00000800);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.W"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_word_extend_carry() {
        // arrange
        let code = [0xe3, 0x4f].to_vec(); // LSL.W #1,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0x00008001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.W"),
                String::from("#$01,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000002, cpu.register.get_d_reg_long(7));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long() {
        // arrange
        let code = [0xe3, 0x8e].to_vec(); // LSL.L #1,D6
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(6, 0x30002001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.L"),
                String::from("#$01,D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x60004002, cpu.register.get_d_reg_long(6));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_negative() {
        // arrange
        let code = [0xed, 0x88].to_vec(); // LSL.L #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x03000303);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.L"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xc000c0c0, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_zero() {
        // arrange
        let code = [0xed, 0x88].to_vec(); // LSL.L #6,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x08000000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.L"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsl_register_by_immediate_long_extend_carry() {
        // arrange
        let code = [0xe3, 0x8f].to_vec(); // LSL.L #1,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0x80000001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSL.L"),
                String::from("#$01,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000002, cpu.register.get_d_reg_long(7));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte() {
        // arrange
        let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00000040);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.B"),
                String::from("#$02,D2")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000010, cpu.register.get_d_reg_long(2));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte_zero() {
        // arrange
        let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00000001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.B"),
                String::from("#$02,D2")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(2));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_byte_extend_carry() {
        // arrange
        let code = [0xe2, 0x0a].to_vec(); // LSR.B #1,D2
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(2, 0x00000081);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.B"),
                String::from("#$01,D2")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000040, cpu.register.get_d_reg_long(2));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word() {
        // arrange
        let code = [0xe2, 0x4b].to_vec(); // LSR.W #1,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(3, 0x00002002);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.W"),
                String::from("#$01,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00001001, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word_zero() {
        // arrange
        let code = [0xe0, 0x4b].to_vec(); // LSR.W #8,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(3, 0x000007f);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.W"),
                String::from("#$08,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(3));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_word_extend_carry() {
        // arrange
        let code = [0xe4, 0x4b].to_vec(); // LSR.W #2,D3
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(3, 0x0000fffe);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.W"),
                String::from("#$02,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00003fff, cpu.register.get_d_reg_long(3));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long() {
        // arrange
        let code = [0xe4, 0x8c].to_vec(); // LSR.L #2,D4
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x30002001);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.L"),
                String::from("#$02,D4")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0c000800, cpu.register.get_d_reg_long(4));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long_zero() {
        // arrange
        let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x00000071);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.L"),
                String::from("#$08,D4")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.register.get_d_reg_long(4));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn lsr_register_by_immediate_long_extend_carry() {
        // arrange
        let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(4, 0x80000181);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("LSR.L"),
                String::from("#$08,D4")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00800001, cpu.register.get_d_reg_long(4));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    // lsl/lsr register by register / XNZC / zero => C clear/X unaffected / byte/word/long

    // lsl/lsr memory(ea) XNZC word
}
