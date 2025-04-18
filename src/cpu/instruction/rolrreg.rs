use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
use crate::cpu::step_log::StepLog;
use crate::cpu::{Cpu, StatusRegisterResult};
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
    STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Tests!

enum RolrDirection {
    Right,
    Left,
}

impl RolrDirection {
    pub fn get_format(&self) -> char {
        match self {
            RolrDirection::Right => 'R',
            RolrDirection::Left => 'L',
        }
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let (rolr_direction, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (RolrDirection::Right, OperationSize::Byte),
        0b001 => (RolrDirection::Right, OperationSize::Word),
        0b010 => (RolrDirection::Right, OperationSize::Long),
        0b011 => (RolrDirection::Right, OperationSize::Word),
        0b100 => (RolrDirection::Left, OperationSize::Byte),
        0b101 => (RolrDirection::Left, OperationSize::Word),
        0b110 => (RolrDirection::Left, OperationSize::Long),
        _ => panic!(),
    };

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
    let status_register_result = match operation_size {
        OperationSize::Byte => {
            let value = reg.get_d_reg_byte(dest_register, step_log);

            println!("value: {}", value);
            println!("shift_count: {}", shift_count);
            let (result, carry) = match rolr_direction {
                RolrDirection::Left => {
                    println!("rolr_direction: left");
                    let result = value.rotate_left(shift_count);
                    (result, (result & 0x01) != 0)
                }
                RolrDirection::Right => {
                    println!("rolr_direction: right");
                    let result = value.rotate_right(shift_count);
                    (result, (result & 0x80) != 0)
                }
            };
            println!("result: {}", result);
            println!("carry: {}", carry);
            reg.set_d_reg_byte(step_log, dest_register, result);
            let mut status_register = 0x0000;
            match result {
                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }

            match carry {
                true => status_register |= STATUS_REGISTER_MASK_CARRY,
                false => (),
            }
            StatusRegisterResult {
                status_register,
                status_register_mask: get_status_register_mask(shift_count),
            }
        }
        OperationSize::Word => {
            let value = reg.get_d_reg_word(dest_register, step_log);
            // println!("value: {}", value);
            let (result, overflow) = match rolr_direction {
                RolrDirection::Left => {
                    // println!("rolr_direction: left");
                    let result = value.rotate_left(shift_count);
                    (result, (result & 0x0001) != 0)
                }
                RolrDirection::Right => {
                    // println!("rolr_direction: right");
                    let result = value.rotate_right(shift_count);
                    (result, (result & 0x8000) != 0)
                }
            };

            reg.set_d_reg_word(step_log, dest_register, result);
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
                true => status_register |= STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY,
                false => (),
            }
            StatusRegisterResult {
                status_register,
                status_register_mask: get_status_register_mask(shift_count),
            }
        }
        OperationSize::Long => {
            let value = reg.get_d_reg_long(dest_register, step_log);
            // println!("value: {}", value);
            let (result, overflow) = match rolr_direction {
                RolrDirection::Left => {
                    // println!("rolr_direction: left");
                    let result = value.rotate_left(shift_count);
                    (result, (result & 0x00000001) != 0)
                }
                RolrDirection::Right => {
                    // println!("rolr_direction: right");
                    let result = value.rotate_right(shift_count);
                    (result, (result & 0x80000000) != 0)
                }
            };

            reg.set_d_reg_long(step_log, dest_register, result);
            let mut status_register = 0x0000;
            match result {
                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            // println!("shift_count: {}", shift_count);
            // println!("result: {}", result);
            // println!("overflow: {}", overflow);
            match overflow {
                true => status_register |= STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY,
                false => (),
            }
            StatusRegisterResult {
                status_register,
                status_register_mask: get_status_register_mask(shift_count),
            }
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

fn get_status_register_mask(shift_count: u32) -> u16 {
    STATUS_REGISTER_MASK_NEGATIVE
        | STATUS_REGISTER_MASK_ZERO
        | STATUS_REGISTER_MASK_OVERFLOW
        | STATUS_REGISTER_MASK_CARRY
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let (rolr_direction, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (RolrDirection::Right, OperationSize::Byte),
        0b001 => (RolrDirection::Right, OperationSize::Word),
        0b010 => (RolrDirection::Right, OperationSize::Long),
        0b011 => (RolrDirection::Right, OperationSize::Word),
        0b100 => (RolrDirection::Left, OperationSize::Byte),
        0b101 => (RolrDirection::Left, OperationSize::Word),
        0b110 => (RolrDirection::Left, OperationSize::Long),
        _ => panic!(),
    };

    let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    match instr_word & 0x0020 {
        0x0020 => {
            let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                format!(
                    "RO{}.{}",
                    rolr_direction.get_format(),
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
                    "RO{}.{}",
                    rolr_direction.get_format(),
                    operation_size.get_format()
                ),
                format!("#${:02X},D{}", count, dest_register),
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

    // rol/ror register by immediate / XNZC / byte/word/long

    #[test]
    fn rol_register_by_immediate_byte() {
        // arrange
        let code = [0xed, 0x18].to_vec(); // ROL.B #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000011);
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
                String::from("ROL.B"),
                String::from("#$06,D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000044, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // #[test]
    // fn rol_register_by_immediate_byte_negative() {
    //     // arrange
    //     let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000003);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("#$06,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x000000c0, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_byte_zero() {
    //     // arrange
    //     let code = [0xed, 0x08].to_vec(); // LSL.B #6,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000008);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("#$06,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_byte_extend_carry() {
    //     // arrange
    //     let code = [0xe3, 0x0f].to_vec(); // LSL.B #1,D7
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000081);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("#$01,D7")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(7));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_word() {
    //     // arrange
    //     let code = [0xe3, 0x4e].to_vec(); // LSL.W #1,D6
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00002001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,D6")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00004002, mm.cpu.register.get_d_reg_long_no_log(6));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_word_negative() {
    //     // arrange
    //     let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000303);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$06,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x0000c0c0, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_word_zero() {
    //     // arrange
    //     let code = [0xed, 0x48].to_vec(); // LSL.W #6,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000800);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$06,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn rol_register_by_immediate_word_extend_carry() {
    //     // arrange
    //     let code = [0xe3, 0x4f].to_vec(); // LSL.W #1,D7
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00008001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );

    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,D7")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(7));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }

    #[test]
    fn rol_register_by_immediate_long() {
        // arrange
        let code = [0xe3, 0x9e].to_vec(); // ROL.L #1,D6
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x30002001);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ROL.L"),
                String::from("#$01,D6")
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
    fn rol_register_by_immediate_long_negative() {
        // arrange
        let code = [0xed, 0x98].to_vec(); // ROL.L #6,D0
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
                String::from("ROL.L"),
                String::from("#$06,D0")
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
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn rol_register_by_immediate_long_zero() {
        // arrange
        let code = [0xed, 0x98].to_vec(); // ROL.L #6,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000000);
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
                String::from("ROL.L"),
                String::from("#$06,D0")
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
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn rol_register_by_immediate_long_extend_carry() {
        // arrange
        let code = [0xe3, 0x9f].to_vec(); // ROL.L #1,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x80000001);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("ROL.L"),
                String::from("#$01,D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000003, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    // THE MESS OF THIS IS DEPRESSING :'(
    
    // #[test]
    // fn lsr_register_by_immediate_byte() {
    //     // arrange
    //     let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000040);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("#$02,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000010, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_byte_zero() {
    //     // arrange
    //     let code = [0xe4, 0x0a].to_vec(); // LSR.B #2,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("#$02,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_byte_extend_carry() {
    //     // arrange
    //     let code = [0xe2, 0x0a].to_vec(); // LSR.B #1,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("#$01,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_word() {
    //     // arrange
    //     let code = [0xe2, 0x4b].to_vec(); // LSR.W #1,D3
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(3, 0x00002002);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$01,D3")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00001001, mm.cpu.register.get_d_reg_long_no_log(3));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_word_zero() {
    //     // arrange
    //     let code = [0xe0, 0x4b].to_vec(); // LSR.W #8,D3
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(3, 0x000007f);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$08,D3")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(3));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_word_extend_carry() {
    //     // arrange
    //     let code = [0xe4, 0x4b].to_vec(); // LSR.W #2,D3
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(3, 0x0000fffe);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$02,D3")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00003fff, mm.cpu.register.get_d_reg_long_no_log(3));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_long() {
    //     // arrange
    //     let code = [0xe4, 0x8c].to_vec(); // LSR.L #2,D4
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(4, 0x30002001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("#$02,D4")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x0c000800, mm.cpu.register.get_d_reg_long_no_log(4));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_long_zero() {
    //     // arrange
    //     let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(4, 0x00000071);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("#$08,D4")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(4));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_immediate_long_extend_carry() {
    //     // arrange
    //     let code = [0xe0, 0x8c].to_vec(); // LSR.L #8,D4
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(4, 0x80000181);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("#$08,D4")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00800001, mm.cpu.register.get_d_reg_long_no_log(4));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // // rol/ror register by register / XNZC / zero => C clear/X unaffected / large shifts / byte/word/long
    //
    // #[test]
    // fn rol_register_by_register_byte() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000001);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_negative() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000003);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x000000c0, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_zero() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000010);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_extend_carry() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_byte_large_shift() {
    //     // arrange
    //     let code = [0xef, 0x28].to_vec(); // LSL.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000101);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00004040, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_negative() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000303);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x0000c0c0, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_zero() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00001000);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_extend_carry() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00008081);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000102, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_word_large_shift() {
    //     // arrange
    //     let code = [0xed, 0x69].to_vec(); // LSL.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x01000101);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x40004040, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_negative() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x03000303);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0xc000c0c0, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_zero() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x10000000);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_extend_carry() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x80008081);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00010102, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x55555555);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x55555555, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn rol_register_by_register_long_large_shift() {
    //     // arrange
    //     let code = [0xeb, 0xaa].to_vec(); // LSL.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSL.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000080);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000002, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte_zero() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000008);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte_extend_carry() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000040, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000055);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000055, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_byte_large_shift() {
    //     // arrange
    //     let code = [0xee, 0x28].to_vec(); // LSR.B D7,D0
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(0, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(7, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.B"),
    //             String::from("D7,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000101);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000004, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word_zero() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000008);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word_extend_carry() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00008081);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00004040, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_word_large_shift() {
    //     // arrange
    //     let code = [0xec, 0x69].to_vec(); // LSR.W D6,D1
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(1, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(6, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("D6,D1")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(1));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x01000101);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000006);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00040004, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long_zero() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000008);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000005);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long_extend_carry() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x80008081);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0x00000001);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x40004040, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long_shift_with_zero_extended_left_cleared() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00005555);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00005555, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long_shift_with_zero_extended_left_set() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x55555555);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 0);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x55555555, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
    //
    // #[test]
    // fn lsr_register_by_register_long_large_shift() {
    //     // arrange
    //     let code = [0xea, 0xaa].to_vec(); // LSR.L D5,D2
    //     let mut mm = crate::tests::instr_test_setup(code, None);
    //     mm.cpu.register.set_d_reg_long_no_log(2, 0x00000081);
    //     mm.cpu.register.set_d_reg_long_no_log(5, 63);
    //     mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //
    //     // act assert - debug
    //     let debug_result = mm.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("LSR.L"),
    //             String::from("D5,D2")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     mm.step();
    //     // assert
    //     assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(2));
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    // }
}
