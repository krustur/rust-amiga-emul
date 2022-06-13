use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::Register;
use crate::{cpu::instruction::PcResult, register::ProgramCounter};
use std::panic;

use super::{DisassemblyResult, InstructionExecutionResult};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

const BYTE_WITH_DN_AS_DEST: usize = 0b000;
const WORD_WITH_DN_AS_DEST: usize = 0b001;
const LONG_WITH_DN_AS_DEST: usize = 0b010;
const BYTE_WITH_EA_AS_DEST: usize = 0b100;
const WORD_WITH_EA_AS_DEST: usize = 0b101;
const LONG_WITH_EA_AS_DEST: usize = 0b110;

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    // ea: u32,
) -> InstructionExecutionResult {
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;
    // let ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos_3_and_reg_pos_0(instr_word);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9);

    let opsize = match opmode {
        BYTE_WITH_DN_AS_DEST => 1,
        WORD_WITH_DN_AS_DEST => 2,
        LONG_WITH_DN_AS_DEST => 4,
        BYTE_WITH_EA_AS_DEST => 1,
        WORD_WITH_EA_AS_DEST => 2,
        LONG_WITH_EA_AS_DEST => 4,
        _ => panic!("What"),
    };

    match opmode {
        BYTE_WITH_DN_AS_DEST => {
            let ea_value = Cpu::get_ea_value_byte(ea_mode, pc, reg, mem);
            let reg_value = (reg.reg_d[register] & 0x000000ff) as u8;
            let add_result = Cpu::add_bytes(ea_value.value, reg_value);

            reg.reg_d[register] = (reg.reg_d[register] & 0xffffff00) | (add_result.result as u32);
            reg.reg_sr = add_result
                .status_register_result
                .merge_status_register(reg.reg_sr);

            return InstructionExecutionResult::Done {
                pc_result: PcResult::Increment,
            };
        }
        WORD_WITH_DN_AS_DEST => {
            let ea_value = Cpu::get_ea_value_word(ea_mode, pc, reg, mem);
            let reg_value = (reg.reg_d[register] & 0x0000ffff) as u16;
            let add_result = Cpu::add_words(ea_value.value, reg_value);

            reg.reg_d[register] = (reg.reg_d[register] & 0xffff0000) | (add_result.result as u32);
            reg.reg_sr = add_result
                .status_register_result
                .merge_status_register(reg.reg_sr);

            return InstructionExecutionResult::Done {
                pc_result: PcResult::Increment,
            };
        }
        LONG_WITH_DN_AS_DEST => {
            let ea_value = Cpu::get_ea_value_long(ea_mode, pc, reg, mem);
            let reg_value = reg.reg_d[register];
            let add_result = Cpu::add_longs(ea_value.value, reg_value);

            reg.reg_d[register] = add_result.result;
            reg.reg_sr = add_result
                .status_register_result
                .merge_status_register(reg.reg_sr);

            return InstructionExecutionResult::Done {
                pc_result: PcResult::Increment,
            };
        }
        _ => panic!("Unhandled ea_opmode"),
    }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9);
    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    match opmode {
        BYTE_WITH_DN_AS_DEST => DisassemblyResult::from_pc(
            pc,
            String::from("ADD.B"),
            format!("{},D{}", ea_format, register),
        ),
        WORD_WITH_DN_AS_DEST => DisassemblyResult::from_pc(
            pc,
            String::from("ADD.W"),
            format!("{},D{}", ea_format, register),
        ),
        LONG_WITH_DN_AS_DEST => DisassemblyResult::from_pc(
            pc,
            String::from("ADD.L"),
            format!("{},D{}", ea_format, register),
        ),
        _ => panic!("Unhandled ea_opmode"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::DisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn step_byte_to_d0() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B (A0),D0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x01, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_byte_to_d0_overflow() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B (A0),D0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x0000007f;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x80, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_byte_to_d0_carry() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B (A0),D0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x000000ff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x00, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x0001, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0_overflow() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x00007fff;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x8000, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0_carry() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W (A0),D0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00C00002;
        cpu.register.reg_d[0] = 0x0000ffff;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(0x0000, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }
}
