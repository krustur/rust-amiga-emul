use super::{GetDisassemblyResult, GetDisassemblyResultError, Instruction, StepError};
use crate::{
    cpu::{instruction::OperationSize, step_log::StepLog, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{ProgramCounter, Register},
};

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
const WORD_WITH_AN_AS_DEST: usize = 0b011;
const LONG_WITH_AN_AS_DEST: usize = 0b111;

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
            match opmode {
                BYTE_WITH_DN_AS_DEST | WORD_WITH_DN_AS_DEST | LONG_WITH_DN_AS_DEST => {
                    crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word)
                }
                BYTE_WITH_EA_AS_DEST | WORD_WITH_EA_AS_DEST | LONG_WITH_EA_AS_DEST => {
                    crate::cpu::match_check_ea_only_memory_alterable_addressing_modes_pos_0(
                        instr_word,
                    )
                }
                WORD_WITH_AN_AS_DEST | LONG_WITH_AN_AS_DEST => {
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
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        WORD_WITH_AN_AS_DEST => OperationSize::Word,
        LONG_WITH_AN_AS_DEST => OperationSize::Long,
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
            let result = Cpu::sub_bytes(ea_value, reg_value);

            reg.set_d_reg_byte(step_log, register, result.result);
            result.status_register_result
        }
        WORD_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let result = Cpu::sub_words(ea_value, reg_value);

            reg.set_d_reg_word(step_log, register, result.result);
            result.status_register_result
        }
        LONG_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, true);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.set_d_reg_long(step_log, register, result.result);
            result.status_register_result
        }
        BYTE_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_byte(register, step_log);
            let result = Cpu::sub_bytes(reg_value, ea_value);
            ea_data.set_value_byte(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        WORD_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_word(register, step_log);
            let result = Cpu::sub_words(reg_value, ea_value);
            ea_data.set_value_word(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        LONG_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, false);
            let reg_value = reg.get_d_reg_long(register, step_log);
            let result = Cpu::sub_longs(reg_value, ea_value);

            ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        WORD_WITH_AN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, step_log, true);
            let ea_value = Cpu::sign_extend_word(ea_value);
            let reg_value = reg.get_a_reg_long(register, step_log);
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.set_a_reg_long(step_log, register, result.result);
            StatusRegisterResult::cleared()
        }
        LONG_WITH_AN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, step_log, true);
            let reg_value = reg.get_a_reg_long(register, step_log);
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.set_a_reg_long(step_log, register, result.result);
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
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        WORD_WITH_AN_AS_DEST => OperationSize::Word,
        LONG_WITH_AN_AS_DEST => OperationSize::Long,
        _ => panic!("Unrecognized opmode"),
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(operation_size),
    )?;
    let ea_mode = ea_data.ea_mode;
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, mem);
    match opmode {
        BYTE_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.B"),
            format!("{},D{}", ea_format, register),
        )),
        WORD_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.W"),
            format!("{},D{}", ea_format, register),
        )),
        LONG_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.L"),
            format!("{},D{}", ea_format, register),
        )),
        BYTE_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.B"),
            format!("D{},{}", register, ea_format),
        )),
        WORD_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.W"),
            format!("D{},{}", register, ea_format),
        )),
        LONG_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUB.L"),
            format!("D{},{}", register, ea_format),
        )),
        WORD_WITH_AN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUBA.W"),
            format!("{},A{}", ea_format, register),
        )),
        LONG_WITH_AN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            String::from("SUBA.L"),
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
    fn sub_address_register_indirect_to_data_register_direct_byte() {
        // arrange
        let code = [0x90, 0x10, /* DC */ 0x01].to_vec(); // SUB.B (A0),D0
                                                         // DC.B $01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
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
                String::from("SUB.B"),
                String::from("(A0),D0")
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
    fn sub_address_register_indirect_to_data_register_direct_byte_overflow() {
        // arrange
        let code = [0x90, 0x10, /* DC */ 0x01].to_vec(); // SUB.B (A0),D0
                                                         // DC.B $01
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
                String::from("SUB.B"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x7f, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_address_register_indirect_to_data_register_direct_byte_carry() {
        // arrange
        let code = [0x90, 0x10, /* DC */ 0x01].to_vec(); // SUB.B (A0),D0
                                                         // DC.B $01
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
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
                String::from("SUB.B"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xff, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_address_register_indirect_to_data_register_direct_word() {
        // arrange
        let code = [0x90, 0x50, /* DC */ 0x00, 0x01].to_vec(); // SUB.W (A0),D0
                                                               // DC.W $0001
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
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
                String::from("SUB.W"),
                String::from("(A0),D0")
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
    fn sub_address_register_indirect_to_data_register_direct_word_overflow() {
        // arrange
        let code = [0x90, 0x50, /* DC */ 0x00, 0x01].to_vec(); // SUB.W (A0),D0
                                                               // DC.W $0001
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x00008000);
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
                String::from("SUB.W"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x7fff, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_address_register_indirect_to_data_register_direct_word_carry() {
        // arrange
        let code = [0x90, 0x50, /* DC */ 0x00, 0x01].to_vec(); // SUB.W (A0),D0
                                                               // DC.W $0001
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00C00002);
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
                String::from("SUB.W"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffff, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_data_register_direct_to_data_register_direct_long() {
        // arrange
        let code = [0x9e, 0x80].to_vec(); // SUB.W D0,D7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x23451234);
        mm.cpu.register.set_d_reg_long_no_log(7, 0x77775555);
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
                String::from("SUB.L"),
                String::from("D0,D7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x54324321, mm.cpu.register.get_d_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_data_register_direct_to_address_register_indirect_long() {
        // arrange
        let code = [0x9f, 0x99, /* DC */ 0x23, 0x56, 0x89, 0xBC].to_vec(); // SUB.L D7,(A1)+
                                                                           // DC.L $235689BC
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
                String::from("SUB.L"),
                String::from("D7,(A1)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x11223344, mm.mem.get_long_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_data_register_direct_to_address_register_indirect_word() {
        // arrange
        let code = [0x9d, 0x5e, /* DC */ 0x35, 0xac].to_vec(); // SUB.W D6,(A6)+
                                                               // DC.W $35AC
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
                String::from("SUB.W"),
                String::from("D6,(A6)+")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x1144, mm.mem.get_word_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_data_register_direct_to_address_register_indirect_byte() {
        // arrange
        let code = [0x95, 0x20, /* DC */ 0x8a].to_vec(); // SUB.B D2,-(A0)
                                                         // DC.B $8A
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
                String::from("SUB.B"),
                String::from("D2,-(A0)")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x44, mm.mem.get_byte_no_log(0x00C00002));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_immediate_data_to_address_register_direct_word() {
        // arrange
        let code = [0x90, 0xfc, 0x44, 0x11].to_vec(); // SUBA.W #$4411,A0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(0, 0x00004311);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("SUBA.W"),
                String::from("#$4411,A0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffffff00, mm.cpu.register.get_a_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn sub_immediate_data_to_address_register_direct_long() {
        // arrange
        let code = [0x9f, 0xfc, 0x88, 0x88, 0x88, 0x88].to_vec(); // SUBA.L #$88888888,A7
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_a_reg_long_no_log(7, 0xaaaa8888);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("SUBA.L"),
                String::from("#$88888888,A7")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x22220000, mm.cpu.register.get_a_reg_long_no_log(7));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
