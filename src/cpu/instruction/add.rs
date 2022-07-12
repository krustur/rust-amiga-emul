use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
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

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let instr_word = pc.peek_next_word(mem);
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

    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(operation_size)
        })?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let status_register_result = match opmode {
        ADD_BYTE_DN_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, true);
            let reg_value = reg.get_d_reg_byte(register);
            let add_result = Cpu::add_bytes(ea_value, reg_value);

            reg.set_d_reg_byte(register, add_result.result);
            add_result.status_register_result
        }
        ADD_WORD_DN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, true);
            let reg_value = reg.get_d_reg_word(register);
            let add_result = Cpu::add_words(ea_value, reg_value);

            reg.set_d_reg_word(register, add_result.result);
            add_result.status_register_result
        }
        ADD_LONG_DN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, true);
            let reg_value = reg.get_d_reg_long(register);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_d_reg_long(register, add_result.result);
            add_result.status_register_result
        }
        ADD_BYTE_EA_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, false);
            let reg_value = reg.get_d_reg_byte(register);
            let add_result = Cpu::add_bytes(ea_value, reg_value);
            ea_data.set_value_byte(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        ADD_WORD_EA_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, false);
            let reg_value = reg.get_d_reg_word(register);
            let add_result = Cpu::add_words(ea_value, reg_value);
            ea_data.set_value_word(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        ADD_LONG_EA_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, false);
            let reg_value = reg.get_d_reg_long(register);
            let add_result = Cpu::add_longs(ea_value, reg_value);
            ea_data.set_value_long(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        ADDA_WORD => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, true);
            let ea_value = Cpu::sign_extend_word(ea_value);
            let reg_value = reg.get_a_reg_long(register);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_a_reg_long(register, add_result.result);
            StatusRegisterResult::cleared()
        }
        ADDA_LONG => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, true);
            let reg_value = reg.get_a_reg_long(register);
            let add_result = Cpu::add_longs(ea_value, reg_value);

            reg.set_a_reg_long(register, add_result.result);
            StatusRegisterResult::cleared()
        }
        _ => panic!("Unhandled ea_opmode"),
    };

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
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

    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(operation_size)
        })?;
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
    match opmode {
        ADD_BYTE_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.B"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_WORD_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.W"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_LONG_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.L"),
            format!("{},D{}", ea_format, register),
        )),
        ADD_BYTE_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.B"),
            format!("D{},{}", register, ea_format),
        )),
        ADD_WORD_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.W"),
            format!("D{},{}", register, ea_format),
        )),
        ADD_LONG_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADD.L"),
            format!("D{},{}", register, ea_format),
        )),
        ADDA_WORD => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("ADDA.W"),
            format!("{},A{}", ea_format, register),
        )),
        ADDA_LONG => Ok(GetDisassemblyResult::from_pc(
            pc,
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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(1, 0x00000001);
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
                String::from("ADD.B"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_byte_overflow() {
        // arrange
        let code = [0xd0, 0x10, /* DC */ 0x01].to_vec(); // ADD.B (A0),D0
                                                         // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(0, 0x0000007f);
        cpu.register.set_d_reg_long(1, 0x00000001);
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
                String::from("ADD.B"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_byte_carry() {
        // arrange
        let code = [0xd0, 0x10, /* DC */ 0x01].to_vec(); // ADD.B (A0),D0
                                                         // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(0, 0x000000ff);
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
                String::from("ADD.B"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.register.get_d_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(1, 0x00000001);
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
                String::from("ADD.W"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0001, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word_overflow() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(0, 0x00007fff);
        cpu.register.set_d_reg_long(1, 0x00000001);
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
                String::from("ADD.W"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x8000, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_register_indirect_to_data_register_direct_word_carry() {
        // arrange
        let code = [0xd0, 0x50, /* DC */ 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                               // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00002);
        cpu.register.set_d_reg_long(0, 0x0000ffff);
        cpu.register.set_d_reg_long(1, 0x00000001);
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
                String::from("ADD.W"),
                String::from("(A0),D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000, cpu.register.get_d_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_data_register_direct_long() {
        // arrange
        let code = [0xde, 0x80].to_vec(); // ADD.W D0,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x23451234);
        cpu.register.set_d_reg_long(7, 0x54324321);
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
                String::from("ADD.L"),
                String::from("D0,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x77775555, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_long() {
        // arrange
        let code = [0xdf, 0x99, /* DC */ 0x11, 0x22, 0x33, 0x44].to_vec(); // ADD.L D7,(A1)+
                                                                           // DC.L 0x11223344
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(1, 0x00C00002);
        cpu.register.set_d_reg_long(7, 0x12345678);
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
                String::from("ADD.L"),
                String::from("D7,(A1)+")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x235689BC, cpu.memory.get_long(0x00C00002));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_word() {
        // arrange
        let code = [0xdd, 0x5e, /* DC */ 0x11, 0x44].to_vec(); // ADD.W D6,(A6)+
                                                               // DC.W 0x1144
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(6, 0x00C00002);
        cpu.register.set_d_reg_long(6, 0x00002468);
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
                String::from("ADD.W"),
                String::from("D6,(A6)+")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x35AC, cpu.memory.get_word(0x00C00002));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn data_register_direct_to_address_register_indirect_byte() {
        // arrange
        let code = [0xd5, 0x20, /* DC */ 0x44].to_vec(); // ADD.B D2,-(A0)
                                                         // DC.B 0x44
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00C00003);
        cpu.register.set_d_reg_long(2, 0x00000046);
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
                String::from("ADD.B"),
                String::from("D2,-(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x8a, cpu.memory.get_byte(0x00C00002));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_address_register_direct_word() {
        // arrange
        let code = [0xd0, 0xfc, 0x44, 0x11].to_vec(); // ADDA.W #$4411,A0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0xffffff00);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("ADDA.W"),
                String::from("#$4411,A0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00004311, cpu.register.get_a_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_address_register_direct_long() {
        // arrange
        let code = [0xdf, 0xfc, 0x88, 0x88, 0x88, 0x88].to_vec(); // ADDA.L #$88888888,A7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(7, 0x22220000);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("ADDA.L"),
                String::from("#$88888888,A7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xaaaa8888, cpu.register.get_a_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
