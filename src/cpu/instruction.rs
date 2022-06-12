use std::fmt;

use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};
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

#[derive(Copy, Clone)]
pub enum PcResult {
    Increment,
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
        address: u32,
        address_next: u32,
        name: String,
        operands_format: String,
        // pc: ProgramCounter,
        // next_pc: ProgramCounter,
    },
    PassOn,
}

impl DisassemblyResult {
    pub fn from_pc(
        pc: &ProgramCounter,
        name: String,
        operands_format: String,
    ) -> DisassemblyResult {
        DisassemblyResult::Done {
            address: pc.get_address(),
            address_next: pc.get_address_next(),
            name,
            operands_format,
        }
    }

    pub fn from_address_and_address_next(
        address: u32,
        address_next: u32,
        name: String,
        operands_format: String,
    ) -> DisassemblyResult {
        DisassemblyResult::Done {
            address,
            address_next,
            name,
            operands_format,
        }
    }
}

#[derive(Copy, Clone, Debug, std::cmp::PartialEq)]
pub enum EffectiveAddressingMode {
    DRegDirect { register: usize }, //                            0b000       Dn
    ARegDirect { register: usize }, //                            0b001       An
    ARegIndirect { register: usize }, //                          0b010       (An)
    ARegIndirectWithPostIncrement { register: usize }, //         0b011       (An)+
    ARegIndirectWithPreDecrement { register: usize }, //          0b100       (-An)
    ARegIndirectWithDisplacement { register: usize }, //          0b101       (d16,An)
    ARegIndirectWithIndexOrMemoryIndirect { register: usize }, // 0b110       -
    // TODO: 020+ CPU's below
    // ARegIndirectWithIndex8BitDisplacement{register: usize}, // 0b110       (d8, An, Xn.SIZE*SCALE)
    // ARegIndirectWithIndexBaseDisplacement{register: usize}, // 0b110       (bd, An, Xn.SIZE*SCALE)
    // MemoryIndirectPostIndexed{register: usize},             // 0b110       ([bd, An], Xn.SIZE*SCALE,od)
    // MemoryIndirectPreIndexed{register: usize},              // 0b110       ([bd, An, Xn.SIZE*SCALE],od)
    PcIndirectWithDisplacement, //                                0b111 0b010 (d16, PC)
    PcIndirectWithIndexOrPcMemoryIndirect, //                     0b110 0b011 -
    // TODO: 020+ CPU's below
    // PcIndirectWithIndex8BitDisplacement{register: usize},   // 0b111 0b011 (d8, PC, Xn.SIZE*SCALE)
    // PcIndirectWithIndexBaseDisplacement{register: usize},   // 0b111 0b011 (bd, PC, Xn.SIZE*SCALE)
    // PcMemoryInderectPostIndexed{register: usize},           // 0b111 0b011 ([bd, PC], Xn.SIZE*SCALE,od)
    // PcMemoryInderectPreIndexed{register: usize},            // 0b111 0b011 ([bd, PC, Xn.SIZE*SCALE],od)
    AbsoluteShortAddressing, //                                   0b111 0b000 (xxx).W
    AbsolutLongAddressing,   //                                   0b111 0b001 (xxx).L
    ImmediateData,           //                                   0b111 0b100 #<xxx>
}

pub struct EffectiveAddressingData {
    instr_word: u16,
    ea_mode: EffectiveAddressingMode,
}

impl EffectiveAddressingData {
    pub fn create(instr_word: u16, ea_mode: EffectiveAddressingMode) -> EffectiveAddressingData {
        EffectiveAddressingData {
            instr_word,
            ea_mode,
        }
    }
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

#[derive(FromPrimitive, Debug, Copy, Clone)]
pub enum ScaleFactor {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}

impl ScaleFactor {
    pub fn scale_as_int(&self) -> u32 {
        match self {
            ScaleFactor::One => 1,
            ScaleFactor::Two => 2,
            ScaleFactor::Four => 4,
            ScaleFactor::Eight => 8,
        }
    }
}

impl fmt::Display for ScaleFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ScaleFactor::One => {
                write!(f, "")
            }
            ScaleFactor::Two => {
                write!(f, "*2")
            }
            ScaleFactor::Four => {
                write!(f, "*4")
            }
            ScaleFactor::Eight => {
                write!(f, "*8")
            }
        }
        // write!(f, "{}", self.format)
    }
}

#[derive(FromPrimitive, Debug)]
pub enum ConditionalTest {
    /// True (1)
    T = 0b0000,
    /// False (0)
    F = 0b0001,
    /// High (!C & !Z)
    HI = 0b0010,
    /// Low or Same (C) | (Z)
    LS = 0b0011,
    /// Carry Clear (!C)
    CC = 0b0100,
    /// Carry Set (C)
    CS = 0b0101,
    /// Not Equal (!Z)
    NE = 0b0110,
    /// Equal (Z)
    EQ = 0b0111,
    /// Overflow Clear (!V)
    VC = 0b1000,
    /// Overflow Set (V)
    VS = 0b1001,
    /// Plus (!N)
    PL = 0b1010,
    /// Minus (N)
    MI = 0b1011,
    /// Greater or Equal (N & V) | (!N & !V)
    GE = 0b1100,
    /// Less Than (N & !V) | (!N & V)
    LT = 0b1101,
    /// Greater Than (N & V & !Z) | (!N & !V & !Z)
    GT = 0b1110,

    /// Less or Equal (Z) | (N & !V) | (!N & V)
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
    pub step: fn(
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &mut Mem,
    ) -> InstructionExecutionResult,
    pub get_debug: fn(pc: &mut ProgramCounter, reg: &Register, mem: &Mem) -> DisassemblyResult,
}

impl Instruction {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        step: fn(
            pc: &mut ProgramCounter,
            reg: &mut Register,
            mem: &mut Mem,
        ) -> InstructionExecutionResult,
        get_debug: fn(pc: &mut ProgramCounter, reg: &Register, mem: &Mem) -> DisassemblyResult,
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
