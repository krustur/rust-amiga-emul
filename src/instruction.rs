use crate::mem::Mem;
use crate::register::Register;

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
    ) -> Instruction {
        let instr = Instruction {
            mask: mask,
            opcode: opcode,
            execute_func: execute_func,
        };
        instr
    }
}

pub struct EaInstruction<'a> {
    pub mask: u16,
    pub opcode: u16,
    pub execute_func:
        fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32,
}

impl<'a> EaInstruction<'a> {
    pub fn new(
        mask: u16,
        opcode: u16,
        execute_func: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
        ) -> u32,
    ) -> EaInstruction {
        let instr = EaInstruction {
            mask: mask,
            opcode: opcode,
            execute_func: execute_func,
        };
        instr
    }
}