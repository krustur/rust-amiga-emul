use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::{Cpu, StatusRegisterResult};
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

const ADD_BYTE_DN_AS_DEST: usize = 0b000;
const ADD_WORD_DN_AS_DEST: usize = 0b001;
const ADD_LONG_DN_AS_DEST: usize = 0b010;
const ADD_BYTE_EA_AS_DEST: usize = 0b100;
const ADD_WORD_EA_AS_DEST: usize = 0b101;
const ADD_LONG_EA_AS_DEST: usize = 0b110;
const ADDA_WORD: usize = 0b011;
const ADDA_LONG: usize = 0b111;

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
            match opmode {
                ADD_BYTE_DN_AS_DEST | ADD_WORD_DN_AS_DEST | ADD_LONG_DN_AS_DEST => {
                    crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word)
                }
                ADD_BYTE_EA_AS_DEST | ADD_WORD_EA_AS_DEST | ADD_LONG_EA_AS_DEST => {
                    crate::cpu::match_check_ea_only_memory_alterable_addressing_modes_pos_0(
                        instr_word,
                    )
                }
                ADDA_WORD | ADDA_LONG => {
                    crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word)
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
        ADD_BYTE_DN_AS_DEST => OperationSize::Byte,
        ADD_WORD_DN_AS_DEST => OperationSize::Word,
        ADD_LONG_DN_AS_DEST => OperationSize::Long,
        ADD_BYTE_EA_AS_DEST => OperationSize::Byte,
        ADD_WORD_EA_AS_DEST => OperationSize::Word,
        ADD_LONG_EA_AS_DEST => OperationSize::Long,
        ADDA_WORD => OperationSize::Word,
        ADDA_LONG => OperationSize::Long,
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
        ADD_BYTE_DN_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_byte(register, step_log);
            let add_result = Cpu::add_bytes(ea_value, reg_value);

            reg.set_d_reg_byte(step_log, register, add_result.result);
            add_result.status_register_result
        }
        ADD_WORD_DN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let add_result = Cpu::add_words(ea_value, reg_value);

            reg.set_d_reg_word(step_log, register, add_result.result);
            add_result.status_register_result
        }
        ADD_LONG_DN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_d_reg_long(step_log, register, add_result.result);
            add_result.status_register_result
        }
        ADD_BYTE_EA_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_byte(register, step_log);
            let add_result = Cpu::add_bytes(ea_value, reg_value);
            ea_data.set_value_byte(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        ADD_WORD_EA_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let add_result = Cpu::add_words(ea_value, reg_value);
            ea_data.set_value_word(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        ADD_LONG_EA_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let add_result = Cpu::add_longs(ea_value, reg_value);
            ea_data.set_value_long(pc, reg, mem, step_log, add_result.result, true);
            add_result.status_register_result
        }
        ADDA_WORD => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            let ea_value = Cpu::sign_extend_word(ea_value);
            let reg_value = reg.get_a_reg_long(register, step_log);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_a_reg_long(step_log, register, add_result.result);
            StatusRegisterResult::cleared()
        }
        ADDA_LONG => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, true);
            let reg_value = reg.get_a_reg_long(register, step_log);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_a_reg_long(step_log, register, add_result.result);
            StatusRegisterResult::cleared()
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
        ADD_BYTE_DN_AS_DEST => OperationSize::Byte,
        ADD_WORD_DN_AS_DEST => OperationSize::Word,
        ADD_LONG_DN_AS_DEST => OperationSize::Long,
        ADD_BYTE_EA_AS_DEST => OperationSize::Byte,
        ADD_WORD_EA_AS_DEST => OperationSize::Word,
        ADD_LONG_EA_AS_DEST => OperationSize::Long,
        ADDA_WORD => OperationSize::Word,
        ADDA_LONG => OperationSize::Long,
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
        ADD_BYTE_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.B"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_WORD_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.W"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_LONG_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.L"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_BYTE_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.B"),
            format!("D{},{}", register, ea_format),
        )),
        ADD_WORD_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.W"),
            format!("D{},{}", register, ea_format),
        )),
        ADD_LONG_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADD.L"),
            format!("D{},{}", register, ea_format),
        )),
        ADDA_WORD => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADDA.W"),
            format!("{},A{}", ea_format, register),
        )),
        ADDA_LONG => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("ADDA.L"),
            format!("{},A{}", ea_format, register),
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
    fn address_register_indirect_to_data_register_direct_byte() {
        // arrange
        let code = [0xd0, 0x10, /* DC */ 0x01].to_vec(); // ADD.B (A0),D0
                                                         // DC.B 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000001);
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
                String::from("ADD.B"),
                String::from("(A0),D0"),
                vec![0xd010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x02, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_byte_overflow() {
        // arrange
        let code = [0xd0, 0x10, /* DC */ 0x01].to_vec(); // ADD.B (A0),D0
                                                         // DC.B 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x0000007f);
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
                String::from("ADD.B"),
                String::from("(A0),D0"),
                vec![0xd010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x80, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_byte_carry() {
        // arrange
        let code = [0xd0, 0x10, /* DC */ 0x01].to_vec(); // ADD.B (A0),D0
                                                         // DC.B 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x000000ff);
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
                String::from("ADD.B"),
                String::from("(A0),D0"),
                vec![0xd010]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00000001);
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
                String::from("ADD.W"),
                String::from("(A0),D0"),
                vec![0xd050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0002, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word_overflow() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00007fff);
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
                String::from("ADD.W"),
                String::from("(A0),D0"),
                vec![0xd050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x8000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word_carry() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x0000ffff);
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
                String::from("ADD.W"),
                String::from("(A0),D0"),
                vec![0xd050]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_data_register_direct_long() {
        // arrange
        let code = [0xde, 0x80].to_vec(); // ADD.L D0,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x23451234);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x54324321);
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
                String::from("ADD.L"),
                String::from("D0,D7"),
                vec![0xde80]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x77775555, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_long() {
        // arrange
        let code = [0xdf, 0x99, /* DC */ 0x11, 0x22, 0x33, 0x44].to_vec(); // ADD.L D7,(A1)+
                                                                           // DC.L 0x11223344
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(1, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x12345678);
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
                String::from("ADD.L"),
                String::from("D7,(A1)+"),
                vec![0xdf99]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x235689BC, mm.mem.get_long_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_word() {
        // arrange
        let code = [0xdd, 0x5e, /* DC */ 0x11, 0x44].to_vec(); // ADD.W D6,(A6)+
                                                               // DC.W 0x1144
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(6, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(6, 0x00002468);
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
                String::from("ADD.W"),
                String::from("D6,(A6)+"),
                vec![0xdd5e]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x35AC, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_byte() {
        // arrange
        let code = [0xd5, 0x20, /* DC */ 0x44].to_vec(); // ADD.B D2,-(A0)
                                                         // DC.B 0x44
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00003);
        mm.cpu.register.set_d_reg_long_no_log(2, 0x00000046);
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
                String::from("ADD.B"),
                String::from("D2,-(A0)"),
                vec![0xd520]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x8a, mm.mem.get_byte_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_address_register_direct_word() {
        // arrange
        let code = [0xd0, 0xfc, 0x44, 0x11].to_vec(); // ADDA.W #$4411,A0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0xffffff00);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ADDA.W"),
                String::from("#$4411,A0"),
                vec![0xd0fc, 0x4411]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00004311, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_address_register_direct_long() {
        // arrange
        let code = [0xdf, 0xfc, 0x88, 0x88, 0x88, 0x88].to_vec(); // ADDA.L #$88888888,A7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0x22220000);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("ADDA.L"),
                String::from("#$88888888,A7"),
                vec![0xdffc, 0x8888, 0x8888]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xaaaa8888, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
