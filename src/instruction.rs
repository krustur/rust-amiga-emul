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

/// Instruction with common EA format:
/// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
///    -   -   -   -|   register|  -   -   -|ea mode    |ea register|
/// 
pub struct EaInstruction<'a> {
    pub name: String,
    pub mask: u16,
    pub opcode: u16,
    pub execute_absolute_short_func:
        Option<fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32) -> String>,
    pub execute_pc_indirect_with_displacement_mode_func:
        Option<fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32) -> String>,
}

impl<'a> EaInstruction<'a> {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        execute_absolute_short_func: Option<fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            register: usize,
            operand: u32,        
        ) -> String>,
        execute_pc_indirect_with_displacement_mode_func: Option<fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            register: usize,
            operand: u32,            
        ) -> String>,
    ) -> EaInstruction<'a> {
        let instr = EaInstruction {
            name: name,
            mask: mask,
            opcode: opcode,
            execute_absolute_short_func: execute_absolute_short_func,
            execute_pc_indirect_with_displacement_mode_func: execute_pc_indirect_with_displacement_mode_func,
        };
        instr
    }
}