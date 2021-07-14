use crate::mem::Mem;
use crate::register::Register;
// use std::fmt;
use num_derive::FromPrimitive;

pub enum InstructionFormat<'a> {
    /// Instruction with uncommon format:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -   -   -   -   -   -   -   -   -   -   -   -   -
    ///
    Uncommon(
        fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
        ) -> (String, String, u32),
    ),
    /// Instruction with common EA format and register:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -|   register|  -   -   -|ea mode    |ea register|
    ///
    EffectiveAddressWithRegister(
        fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            register: usize,
            ea: u32,
        ) -> String,
    ),
    /// Instruction with common EA format and opmode and register:
    /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
    ///    -   -   -   -|register   |opmode     |ea mode    |ea register|
    ///
    EffectiveAddressWithOpmodeAndRegister(
        fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem<'a>,
            ea_opmode: usize,
            register: usize,
            ea: u32,
        ) -> (String, OperationSize),
    ),
}

#[derive(FromPrimitive, Debug)]
pub enum EffectiveAddressingMode {
    DRegDirect = 0b000,
    ARegDirect = 0b001,
    ARegIndirect = 0b010,
    ARegIndirectWithPostIncrement = 0b011,
    ARegIndirectWithPreDecrement = 0b100,
    ARegIndirectWithDisplacement = 0b101,
    // TODO: Figure these out
    // ARegIndirectWithIndex           = 0b110,
    ARegIndirectWithIndex = 0b110,
    PcIndirectAndLotsMore = 0b111,
}

#[derive(FromPrimitive, Debug)]
pub enum OperationSize {
    Byte = 0b00,
    Word = 0b01,
    Long = 0b10,
}

impl OperationSize {
    pub fn size_in_bytes(&self) -> u32 {
        match self {
            OperationSize::Byte => 1,
            OperationSize::Word => 2,
            OperationSize::Long => 4,
        }
    }
}

#[derive(FromPrimitive, Debug)]
pub enum ConditionalTest {
    /// True
    T = 0b0000,
    /// False
    F = 0b0001,
    /// High
    HI = 0b0010,
    /// Low or Same
    LS = 0b0011,
    /// Carry Clear (CC HI)
    CC = 0b0100,
    /// Carry Set (CC LO)
    CS = 0b0101,
    /// Not Equal
    NE = 0b0110,
    /// Equal
    EQ = 0b0111,
    /// Overflow Clear
    VC = 0b1000,
    /// Overflow Set
    VS = 0b1001,
    /// Plus
    PL = 0b1010,
    /// Minus
    MI = 0b1011,
    /// Greater or Equal
    GE = 0b1100,
    /// Less Than
    LT = 0b1101,
    /// Greater Than
    GT = 0b1110,
    /// Less or Equal
    LE = 0b1111,
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
