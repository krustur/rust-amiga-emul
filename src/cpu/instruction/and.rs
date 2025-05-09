use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};
use std::panic;

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

const BYTE_WITH_DN_AS_DEST: usize = 0b000;
const WORD_WITH_DN_AS_DEST: usize = 0b001;
const LONG_WITH_DN_AS_DEST: usize = 0b010;
const BYTE_WITH_EA_AS_DEST: usize = 0b100;
const WORD_WITH_EA_AS_DEST: usize = 0b101;
const LONG_WITH_EA_AS_DEST: usize = 0b110;

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
            match opmode {
                BYTE_WITH_DN_AS_DEST | WORD_WITH_DN_AS_DEST | LONG_WITH_DN_AS_DEST => {
                    crate::cpu::match_check_ea_only_data_addressing_modes_pos_0(instr_word)
                }
                BYTE_WITH_EA_AS_DEST | WORD_WITH_EA_AS_DEST | LONG_WITH_EA_AS_DEST => {
                    crate::cpu::match_check_ea_only_memory_alterable_addressing_modes_pos_0(
                        instr_word,
                    )
                }
                _ => false,
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
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let operation_size = match opmode {
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        _ => panic!("Unrecognized opmode"),
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(operation_size),
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let status_register_result = match opmode {
        BYTE_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_byte(register, step_log);
            let add_result = Cpu::and_bytes(ea_value, reg_value);

            reg.set_d_reg_byte(step_log, register, add_result.result);
            add_result.status_register_result
        }
        WORD_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let add_result = Cpu::and_words(ea_value, reg_value);

            reg.set_d_reg_word(step_log, register, add_result.result);
            add_result.status_register_result
        }
        LONG_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let add_result = Cpu::and_longs(ea_value, reg_value);

            reg.set_d_reg_long(step_log, register, add_result.result);
            add_result.status_register_result
        }
        BYTE_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_byte(register, step_log);
            let add_result = Cpu::and_bytes(ea_value, reg_value);
            ea_data.set_value_byte(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        WORD_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let add_result = Cpu::and_words(ea_value, reg_value);
            ea_data.set_value_word(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        LONG_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let add_result = Cpu::and_longs(ea_value, reg_value);
            ea_data.set_value_long(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        _ => panic!("Unhandled ea_opmode"),
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
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let operation_size = match opmode {
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        _ => panic!("Unrecognized opmode"),
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(operation_size),
    )?;
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
    match opmode {
        BYTE_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.B"),
            format!("{},D{}", ea_format, register),
        )),
        WORD_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.W"),
            format!("{},D{}", ea_format, register),
        )),
        LONG_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.L"),
            format!("{},D{}", ea_format, register),
        )),
        BYTE_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.B"),
            format!("D{},{}", register, ea_format),
        )),
        WORD_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.W"),
            format!("D{},{}", register, ea_format),
        )),
        LONG_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("AND.L"),
            format!("D{},{}", register, ea_format),
        )),
        _ => panic!("Unhandled ea_opmode: {}", opmode),
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
    fn and_byte_address_register_indirect_to_data_register_direct() {
        // arrange
        let code = [0xc0, 0x10, /* DC */ 0x33].to_vec(); // AND.B (A0),D0
                                                         // DC.B 0x33
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x000000f1);
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
                String::from("AND.B"),
                String::from("(A0),D0"),
                vec![0xc010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x31, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_byte_address_register_indirect_to_data_register_direct_negative() {
        // arrange
        let code = [0xc0, 0x10, /* DC */ 0x81].to_vec(); // AND.B (A0),D0
                                                         // DC.B 0x81
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000087);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.B"),
                String::from("(A0),D0"),
                vec![0xc010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x81, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_byte_address_register_indirect_to_data_register_direct_zero() {
        // arrange
        let code = [0xc0, 0x10, /* DC */ 0xf0].to_vec(); // AND.B (A0),D0
                                                         // DC.B 0xf0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x0000000f);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.B"),
                String::from("(A0),D0"),
                vec![0xc010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_address_register_indirect_to_data_register_direct() {
        // arrange
        let code = [0xc0, 0x50, /* DC */ 0x01, 0x0f].to_vec(); // AND.W (A0),D0
                                                               // DC.W $010F
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x000087ff);
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
                String::from("AND.W"),
                String::from("(A0),D0"),
                vec![0xc050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x010F, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_address_register_indirect_to_data_register_direct_negative() {
        // arrange
        let code = [0xc0, 0x50, /* DC */ 0xc0, 0xff].to_vec(); // AND.W (A0),D0
                                                               // DC.W $C0FF
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00008fff);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.W"),
                String::from("(A0),D0"),
                vec![0xc050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80ff, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_address_register_indirect_to_data_register_direct_zero() {
        // arrange
        let code = [0xc0, 0x50, /* DC */ 0xff, 0x0f].to_vec(); // AND.W (A0),D0
                                                               // DC.W 0xff0f
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000080);
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
                String::from("AND.W"),
                String::from("(A0),D0"),
                vec![0xc050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_address_register_indirect_to_data_register_direct() {
        // arrange
        let code = [0xc0, 0x90, /* DC */ 0x01, 0x0f, 0x01, 0x0f].to_vec(); // AND.L (A0),D0
                                                                           // DC.W $010F010F
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x87ff87ff);
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
                String::from("AND.L"),
                String::from("(A0),D0"),
                vec![0xc090]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x010F010F, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_address_register_indirect_to_data_register_direct_negative() {
        // arrange
        let code = [0xc0, 0x90, /* DC */ 0xc0, 0xff, 0xc0, 0xff].to_vec(); // AND.L (A0),D0
                                                                           // DC.W $C0FFC0FF
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x8fff8fff);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.L"),
                String::from("(A0),D0"),
                vec![0xc090]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80ff80ff, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_address_register_indirect_to_data_register_direct_zero() {
        // arrange
        let code = [0xc0, 0x90, /* DC */ 0xff, 0x0f, 0xff, 0x0f].to_vec(); // AND.L (A0),D0
                                                                           // DC.W 0xff0fff0f
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00800080);
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
                String::from("AND.L"),
                String::from("(A0),D0"),
                vec![0xc090]
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
    fn and_byte_data_register_direct_to_address_register_indirect() {
        // arrange
        let code = [0xc1, 0x10, /* DC */ 0x33].to_vec(); // AND.B D0,(A0)
                                                         // DC.B 0x33
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x000000f1);
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
                String::from("AND.B"),
                String::from("D0,(A0)"),
                vec![0xc110]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x31, mm.mem.get_byte_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_byte_data_register_direct_to_address_register_indirect_negative() {
        // arrange
        let code = [0xc1, 0x10, /* DC */ 0x81].to_vec(); // AND.B D0,(A0)
                                                         // DC.B 0x81
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000087);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.B"),
                String::from("D0,(A0)"),
                vec![0xc110]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x81, mm.mem.get_byte_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_byte_data_register_direct_to_address_register_indirect_zero() {
        // arrange
        let code = [0xc1, 0x10, /* DC */ 0xf0].to_vec(); // AND.B D0,(A0)
                                                         // DC.B 0xf0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x0000000f);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.B"),
                String::from("D0,(A0)"),
                vec![0xc110]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.mem.get_byte_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_data_register_direct_to_address_register_indirect() {
        // arrange
        let code = [0xc1, 0x50, /* DC */ 0x01, 0x0f].to_vec(); // AND.W D0,(A0)
                                                               // DC.W $010F
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x000087ff);
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
                String::from("AND.W"),
                String::from("D0,(A0)"),
                vec![0xc150]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x010F, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_data_register_direct_to_address_register_indirect_negative() {
        // arrange
        let code = [0xc1, 0x50, /* DC */ 0xc0, 0xff].to_vec(); // AND.W D0,(A0)
                                                               // DC.W $C0FF
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00008fff);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.W"),
                String::from("D0,(A0)"),
                vec![0xc150]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80ff, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_word_data_register_direct_to_address_register_indirect_zero() {
        // arrange
        let code = [0xc1, 0x50, /* DC */ 0xff, 0x0f].to_vec(); // AND.W D0,(A0)
                                                               // DC.W 0xff0f
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000080);
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
                String::from("AND.W"),
                String::from("D0,(A0)"),
                vec![0xc150]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_data_register_direct_to_address_register_indirect() {
        // arrange
        let code = [0xc1, 0x90, /* DC */ 0x01, 0x0f, 0x01, 0x0f].to_vec(); // AND.L D0,(A0)
                                                                           // DC.W $010F010F
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x87ff87ff);
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
                String::from("AND.L"),
                String::from("D0,(A0)"),
                vec![0xc190]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x010F010F, mm.mem.get_long_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_data_register_direct_to_address_register_indirect_negative() {
        // arrange
        let code = [0xc1, 0x90, /* DC */ 0xc0, 0xff, 0xc0, 0xff].to_vec(); // AND.L D0,(A0)
                                                                           // DC.W $C0FFC0FF
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x8fff8fff);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("AND.L"),
                String::from("D0,(A0)"),
                vec![0xc190]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80ff80ff, mm.mem.get_long_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn and_long_data_register_direct_to_address_register_indirect_zero() {
        // arrange
        let code = [0xc1, 0x90, /* DC */ 0xff, 0x0f, 0xff, 0x0f].to_vec(); // AND.L D0,(A0)
                                                                           // DC.W 0xff0fff0f
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00800080);
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
                String::from("AND.L"),
                String::from("D0,(A0)"),
                vec![0xc190]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.mem.get_long_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
