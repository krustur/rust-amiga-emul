use std::fmt;

use crate::mem::Mem;
use crate::register::Register;
use num_derive::FromPrimitive;

pub mod add;
pub mod addq;
pub mod addx;
pub mod bcc;
pub mod cmp;
pub mod cmpi;
pub mod dbcc;
pub mod jmp;
pub mod lea;
pub mod mov;
pub mod moveq;
pub mod nop;
pub mod subq;

// pub mod instruction;


// pub enum InstructionFormat {
//     /// Instruction with uncommon format:
//     /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
//     ///    -   -   -   -   -   -   -   -   -   -   -   -   -   -   -   -
//     ///
//     Uncommon{
//         step: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &mut Register,
//             mem: &mut Mem,
//         ) -> InstructionExecutionResult,
//         get_debug: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &Register,
//             mem: &Mem
//         ) -> InstructionDebugResult
//     },
//     /// Instruction with common EA format and register:
//     /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
//     ///    -   -   -   -   -   -   -   -   -   -|ea mode    |ea register|
//     ///
//     EffectiveAddress{
//         common_step: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &mut Register,
//             mem: &mut Mem,
//             ea: u32,
//         ) -> InstructionExecutionResult,
//         common_get_debug: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &Register,
//             mem: &Mem,
//             ea_format: String,
//             ea: u32,
//         ) -> InstructionDebugResult,
//         areg_direct_step: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &mut Register,
//             mem: &mut Mem,
//             ea_register: usize
//         ) -> InstructionExecutionResult,
//         areg_direct_get_debug: fn(
//             instr_address: u32,
//             instr_word: u16,
//             reg: &Register,
//             mem: &Mem,
//             ea_register: usize
//         ) -> InstructionDebugResult,

//     },
//     // /// Instruction with common EA format and register:
//     // /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
//     // ///    -   -   -   -|   register|  -   -   -|ea mode    |ea register|
//     // ///
//     // EffectiveAddressWithRegister(
//     //     fn(
//     //         instr_address: u32,
//     //         instr_word: u16,
//     //         reg: &mut Register,
//     //         mem: &mut Mem,
//     //         ea_format: String,
//     //         register: usize,
//     //         ea: u32,
//     //     ) -> InstructionExecutionResult,
//     // ),
//     // /// Instruction with common EA format and opmode and register:
//     // /// | 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
//     // ///    -   -   -   -|register   |opmode     |ea mode    |ea register|
//     // ///
//     // EffectiveAddressWithOpmodeAndRegister(
//     //     fn(
//     //         instr_address: u32,
//     //         instr_word: u16,
//     //         reg: &mut Register,
//     //         mem: &mut Mem,
//     //         ea_format: String,
//     //         ea_opmode: usize,
//     //         register: usize,
//     //         ea: u32,
//     //     ) -> InstructionExecutionResult,
//     // ),
// }

#[derive(Copy, Clone)]
pub enum PcResult {
    Increment(u32),
    Set(u32),
}

#[derive(Copy, Clone)]
pub enum InstructionExecutionResult {
    Done { pc_result: PcResult },
    PassOn,
}

#[derive(Debug, PartialEq)]
pub enum DisassemblyResult {
    Done {
        name: String,
        operands_format: String,
        instr_address: u32,
        next_instr_address: u32,
    },
    PassOn,
}

#[derive(Copy, Clone, FromPrimitive, Debug, std::cmp::PartialEq)]
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

#[derive(FromPrimitive, Debug, Copy, Clone)]
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

impl fmt::Display for ConditionalTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConditionalTest::T => {
                write!(f, "T")
            }
            ConditionalTest::F => {
                write!(f, "F")
            }
            ConditionalTest::HI => {
                write!(f, "HI")
            }
            ConditionalTest::LS => {
                write!(f, "LS")
            }
            ConditionalTest::CC => {
                write!(f, "CC")
            }
            ConditionalTest::NE => {
                write!(f, "NE")
            }
            ConditionalTest::EQ => {
                write!(f, "EQ")
            }
            ConditionalTest::VC => {
                write!(f, "VC")
            }
            ConditionalTest::VS => {
                write!(f, "VS")
            }
            ConditionalTest::PL => {
                write!(f, "PL")
            }
            ConditionalTest::MI => {
                write!(f, "MI")
            }
            ConditionalTest::GE => {
                write!(f, "GE")
            }
            ConditionalTest::LT => {
                write!(f, "LT")
            }
            ConditionalTest::GT => {
                write!(f, "GT")
            }
            ConditionalTest::LE => {
                write!(f, "LE")
            }
            _ => {
                write!(f, "cc")
            }
        }
        // write!(f, "{}", self.format)
    }
}

pub struct Instruction {
    pub name: String,
    pub mask: u16,
    pub opcode: u16,
    // pub instruction_format: InstructionFormat,
    pub step: fn(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem,
    ) -> InstructionExecutionResult,
    pub get_debug: fn(
        instr_address: u32,
        instr_word: u16,
        reg: &Register,
        mem: &Mem,
    ) -> DisassemblyResult,
}

impl Instruction {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        step: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &mut Register,
            mem: &mut Mem,
        ) -> InstructionExecutionResult,
        get_debug: fn(
            instr_address: u32,
            instr_word: u16,
            reg: &Register,
            mem: &Mem,
        ) -> DisassemblyResult,
    ) -> Instruction {
        let instr = Instruction {
            name: name,
            mask: mask,
            opcode: opcode,
            step: step,
            get_debug: get_debug,
        };
        instr
    }
}
