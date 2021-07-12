use crate::mem::Mem;
use crate::register::Register;

pub enum InstructionFormat<'a>{
    /// Instruction with uncommon format:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -   -   -   -   -   -   -   -   -   -   -   -   - 
    /// 
    Uncommon(
        fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> (String, String, u32)
    ),
    /// Instruction with common EA format and register:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -|   register|  -   -   -|ea mode    |ea register|
    /// 
    EffectiveAddressWithRegister{
        exec_func_absolute_short: Option<fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32) -> String>,
        exec_func_pc_indirect_with_displacement_mode: Option<fn(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>, register: usize, operand: u32) -> String>},
    /// Instruction with common EA format and opmode and register:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -|register   |opmode     |ea mode    |ea register|
    /// 
    EffectiveAddressWithOpmodeAndRegister(),
}

pub struct Instruction<'a> {
    pub name: String,
    pub mask: u16,
    pub opcode: u16,
    pub instruction_format: InstructionFormat<'a>,
}

impl<'a> Instruction<'a> {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        instruction_format: InstructionFormat<'a>,
    ) -> Instruction<'a> {
        let instr = Instruction {
            name: name,
            mask: mask,
            opcode: opcode,
            instruction_format: instruction_format,
        };
        instr
    }
}