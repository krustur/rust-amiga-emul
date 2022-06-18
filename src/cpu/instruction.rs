use std::fmt;

use crate::mem::Mem;
use crate::register::{ProgramCounter, Register, RegisterType};
use num_derive::FromPrimitive;

pub mod add;
pub mod addi;
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

// #[derive(Copy, Clone)]
// pub enum PcResult {
//     Increment,
//     Set(u32),
// }

#[derive(Copy, Clone)]
pub enum StepResult {
    Done,
    PassOn,
}

#[derive(Debug, PartialEq)]
pub struct GetDisassemblyResult {
    pub address: u32,
    pub address_next: u32,
    pub name: String,
    pub operands_format: String,
}

impl GetDisassemblyResult {
    pub fn from_pc(
        pc: &ProgramCounter,
        name: String,
        operands_format: String,
    ) -> GetDisassemblyResult {
        GetDisassemblyResult {
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
    ) -> GetDisassemblyResult {
        GetDisassemblyResult {
            address,
            address_next,
            name,
            operands_format,
        }
    }
}

#[derive(Copy, Clone, Debug, std::cmp::PartialEq)]
pub enum EffectiveAddressingMode {
    DRegDirect {
        //                                   0b000       Dn
        ea_register: usize,
    },
    ARegDirect {
        //                                   0b001       An
        ea_register: usize,
    },
    ARegIndirect {
        //                   0b010       (An)
        ea_register: usize,
        ea_address: u32,
    },
    ARegIndirectWithPostIncrement {
        //  0b011       (An)+
        operation_size: OperationSize,
        ea_register: usize,
        ea_address: u32,
    },
    ARegIndirectWithPreDecrement {
        //   0b100       (-An)
        operation_size: OperationSize,
        ea_register: usize,
        ea_address: u32,
    },
    ARegIndirectWithDisplacement {
        //   0b101       (d16,An)
        ea_register: usize,
        ea_address: u32,
        ea_displacement: u16,
    },
    ARegIndirectWithIndexOrMemoryIndirect {
        // 0b110       -
        ea_register: usize,
        ea_address: u32,
        extension_word: u16,
        displacement: u8,
        register_type: RegisterType,
        register: usize,
        index_size: OperationSize,
        scale_factor: ScaleFactor,
    },
    // TODO: 020+ CPU's below
    // ARegIndirectWithIndex8BitDisplacement{register: usize}, // 0b110       (d8, An, Xn.SIZE*SCALE)
    // ARegIndirectWithIndexBaseDisplacement{register: usize}, // 0b110       (bd, An, Xn.SIZE*SCALE)
    // MemoryIndirectPostIndexed{register: usize},             // 0b110       ([bd, An], Xn.SIZE*SCALE,od)
    // MemoryIndirectPreIndexed{register: usize},              // 0b110       ([bd, An, Xn.SIZE*SCALE],od)
    PcIndirectWithDisplacement {
        //                                0b111 0b010 (d16, PC)
        ea_address: u32,
        displacement: u16,
    },
    PcIndirectWithIndexOrPcMemoryIndirect {
        //                     0b110 0b011 -
        ea_register: usize,
        ea_address: u32,
        extension_word: u16,
        displacement: u8,
        register_type: RegisterType,
        register: usize,
        index_size: OperationSize,
        scale_factor: ScaleFactor,
    },
    // TODO: 020+ CPU's below
    // PcIndirectWithIndex8BitDisplacement{register: usize},   // 0b111 0b011 (d8, PC, Xn.SIZE*SCALE)
    // PcIndirectWithIndexBaseDisplacement{register: usize},   // 0b111 0b011 (bd, PC, Xn.SIZE*SCALE)
    // PcMemoryInderectPostIndexed{register: usize},           // 0b111 0b011 ([bd, PC], Xn.SIZE*SCALE,od)
    // PcMemoryInderectPreIndexed{register: usize},            // 0b111 0b011 ([bd, PC, Xn.SIZE*SCALE],od)
    AbsoluteShortAddressing {
        //                                   0b111 0b000 (xxx).W
        ea_address: u32,
        displacement: u16,
    },
    AbsolutLongAddressing {
        //                                   0b111 0b001 (xxx).L
        ea_address: u32,
    },
    ImmediateDataByte {
        //                                   0b111 0b100 #<xxx>
        data: u8,
    },
    ImmediateDataWord {
        //                                   0b111 0b100 #<xxx>
        data: u16,
    },
    ImmediateDataLong {
        //                                   0b111 0b100 #<xxx>
        data: u32,
    },
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq)]
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

    pub fn get_format(&self) -> char {
        match self {
            OperationSize::Byte => 'B',
            OperationSize::Word => 'W',
            OperationSize::Long => 'L',
        }
    }
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq)]
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
    pub step: fn(pc: &mut ProgramCounter, reg: &mut Register, mem: &mut Mem) -> StepResult,
    pub get_disassembly:
        fn(pc: &mut ProgramCounter, reg: &Register, mem: &Mem) -> GetDisassemblyResult,
}

impl Instruction {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        step: fn(pc: &mut ProgramCounter, reg: &mut Register, mem: &mut Mem) -> StepResult,
        get_disassembly: fn(
            pc: &mut ProgramCounter,
            reg: &Register,
            mem: &Mem,
        ) -> GetDisassemblyResult,
    ) -> Instruction {
        let instr = Instruction {
            name: name,
            mask: mask,
            opcode: opcode,
            step: step,
            get_disassembly,
        };
        instr
    }
}
