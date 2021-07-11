use crate::mem::Mem;
use crate::register::Register;

/// Instruction with uncommon layout:
/// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
///    -   -   -   -   -   -   -   -   -   -   -   -   -   -   -   - 
/// 

pub struct Instruction<'a> {
    pub mask: u16,
    pub opcode: u16,
    pub execute_func:
        fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32,
}

impl<'a> Instruction<'a> {
    pub fn new(
        mask: u16,
        opcode: u16,
        execute_func: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
        ) -> u32,
    ) -> Instruction<'a> {
        let instr = Instruction {
            mask: mask,
            opcode: opcode,
            execute_func: execute_func,
        };
        instr
    }
}

/// Instruction with common EA layout:
/// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
///    -   -   -   -|   register|  -   -   -|ea mode    |ea register|
/// 
pub struct EaInstruction<'a> {
    pub mask: u16,
    pub opcode: u16,
    pub execute_absolute_short_func:
        fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32, extension_word: i16) -> u32,
    pub execute_program_counter_indirect_with_displacement_mode_func:
        fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32, extension_word: i16) -> u32,
}

impl<'a> EaInstruction<'a> {
    pub fn new(
        mask: u16,
        opcode: u16,
        execute_absolute_short_func: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            register: usize,
            operand: u32,
            extension_word: i16
        ) -> u32,
        execute_program_counter_indirect_with_displacement_mode_func: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            register: usize,
            operand: u32,
            extension_word: i16
        ) -> u32,
    ) -> EaInstruction<'a> {
        let instr = EaInstruction {
            mask: mask,
            opcode: opcode,
            execute_absolute_short_func: execute_absolute_short_func,
            execute_program_counter_indirect_with_displacement_mode_func: execute_program_counter_indirect_with_displacement_mode_func,
        };
        instr
    }
}