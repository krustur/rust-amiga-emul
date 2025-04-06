use std::fmt::{self, Display};

use crate::{
    mem::Mem,
    register::{ProgramCounter, Register, RegisterType},
};

use super::step_log::StepLog;

pub mod add;
pub mod addi;
pub mod addq;
pub mod addx;
pub mod and;
pub mod andi;
pub mod andi_to_sr;
pub mod bcc;
pub mod bclr;
pub mod bra;
pub mod bset;
pub mod bsr;
pub mod btst;
pub mod clr;
pub mod cmp;
pub mod cmpi;
pub mod cmpm;
pub mod dbcc;
pub mod exg;
pub mod jmp;
pub mod jsr;
pub mod lea;
pub mod link;
pub mod lslr;
pub mod mov;
pub mod move_from_sr;
pub mod move_to_sr;
pub mod move_usp;
pub mod movec;
pub mod movem;
pub mod moveq;
pub mod mulu;
pub mod neg;
pub mod nop;
pub mod not;
pub mod or;
pub mod ori;
pub mod ori_to_sr;
pub mod pea;
pub mod reset;
pub mod rolrmem;
pub mod rolrreg;
pub mod rte;
pub mod rts;
pub mod scc;
pub mod sub;
pub mod subi;
pub mod subq;
pub mod subx;
pub mod swap;
pub mod gen_tests;
pub mod tst;
pub mod unlk;

pub struct InstructionError {
    pub details: String,
}

#[allow(non_camel_case_types)]
pub enum StepError {
    AccessFault,
    AddressError,
    IllegalInstruction,
    IntegerDivideByZero,
    CHK_CHK2_Instruction,
    FRAPcc_TRAPcc_TRAPV_Instruction,
    PriviliegeViolation,
    // InstructionError isn't an actual hardware error. This error
    // is probably the result of an unimplemented instruction or an
    // instruction that is incorrectly implemented. And if not, this
    // is most likely a normal illegal instruction - which could be
    // the case when running a program that requires a cpu/fpu that
    // isn't connected
    InstructionError { details: String },
}

impl Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepError::AccessFault => write!(f, "AccessFault"),
            StepError::AddressError => write!(f, "AddressError"),
            StepError::IllegalInstruction => write!(f, "IllegalInstruction"),
            StepError::IntegerDivideByZero => write!(f, "IntegerDivideByZero"),
            StepError::CHK_CHK2_Instruction => write!(f, "CHK_CHK2_Instruction"),
            StepError::FRAPcc_TRAPcc_TRAPV_Instruction => {
                write!(f, "FRAPcc_TRAPcc_TRAPV_Instruction")
            }
            StepError::PriviliegeViolation => write!(f, "PriviliegeViolation"),
            StepError::InstructionError { details } => write!(f, "InstructionError: {}", details),
        }
    }
}
impl From<InstructionError> for StepError {
    fn from(error: InstructionError) -> Self {
        match error {
            InstructionError { details } => StepError::InstructionError { details },
        }
    }
}

pub struct GetDisassemblyResultError {
    pub details: String,
}

impl From<InstructionError> for GetDisassemblyResultError {
    fn from(error: InstructionError) -> Self {
        match error {
            InstructionError { details } => GetDisassemblyResultError { details },
        }
    }
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

#[derive(Copy, Clone, Debug, PartialEq)]
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
    },
    ARegIndirectWithPreDecrement {
        //   0b100       (-An)
        operation_size: OperationSize,
        ea_register: usize,
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

impl Display for EffectiveAddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EffectiveAddressingMode::DRegDirect { ea_register } => {
                write!(f, "EffectiveAddressingMode::DRegDirect D{}", ea_register)
            }
            EffectiveAddressingMode::ARegDirect { ea_register } => {
                write!(f, "EffectiveAddressingMode::ARegDirect A{}", ea_register)
            }
            EffectiveAddressingMode::ARegIndirect {
                ea_register,
                ea_address,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ARegIndirect (A{}) [${:08X}]",
                    ea_register, ea_address
                )
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                operation_size,
                ea_register,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ARegIndirectWithPostIncrement (A{})+ {}",
                    ea_register, operation_size
                )
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                operation_size,
                ea_register,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ARegIndirectWithPreDecrement -(A{}) {}",
                    ea_register, operation_size
                )
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement {
                ea_register,
                ea_address,
                ea_displacement,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ARegIndirectWithDisplacement ({},A{}) [${:08X}]",
                    ea_displacement, ea_register, ea_address
                )
            }
            EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect {
                ea_register,
                ea_address,
                extension_word,
                displacement,
                register_type,
                register,
                index_size,
                scale_factor,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect A{} TODO fmt::Display() [${:08X}]",
                    ea_register, ea_address
                )
            }
            EffectiveAddressingMode::PcIndirectWithDisplacement {
                ea_address,
                displacement,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::PcIndirectWithDisplacement ({},PC) [${:08X}]",
                    displacement, ea_address
                )
            }
            EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect {
                ea_register,
                ea_address,
                extension_word,
                displacement,
                register_type,
                register,
                index_size,
                scale_factor,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect PC A/D{} TODO fmt::Display() [${:08X}]",
                    ea_register, ea_address
                )
            }
            EffectiveAddressingMode::AbsoluteShortAddressing {
                ea_address,
                displacement,
            } => {
                write!(
                    f,
                    "EffectiveAddressingMode::AbsoluteShortAddressing (${:04X}) [${:08X}]",
                    displacement, ea_address
                )
            }

            EffectiveAddressingMode::AbsolutLongAddressing { ea_address } => {
                write!(
                    f,
                    "EffectiveAddressingMode::AbsolutLongAddressing [${:08X}]",
                    ea_address
                )
            }

            EffectiveAddressingMode::ImmediateDataByte { data } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ImmediateDataByte #${:02X}",
                    data
                )
            }
            EffectiveAddressingMode::ImmediateDataWord { data } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ImmediateDataWord #${:04X}",
                    data
                )
            }
            EffectiveAddressingMode::ImmediateDataLong { data } => {
                write!(
                    f,
                    "EffectiveAddressingMode::ImmediateDataLong #${:08X}",
                    data
                )
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperationSize {
    Byte,
    Word,
    Long,
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

impl Display for OperationSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OperationSize::Byte => {
                write!(f, "Byte")
            }
            OperationSize::Word => {
                write!(f, "Word")
            }
            OperationSize::Long => {
                write!(f, "Long")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScaleFactor {
    One,
    Two,
    Four,
    Eight,
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

impl Display for ScaleFactor {
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
    }
}

#[derive(Debug)]
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

impl Display for ConditionalTest {
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
            ConditionalTest::CS => {
                write!(f, "CS")
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
        }
    }
}

pub struct Instruction {
    pub name: String,
    pub mask: u16,
    pub opcode: u16,
    pub ex_mask: u16,
    pub ex_code: Vec<u16>,
    pub match_check: fn(instruction: &Instruction, instr_word: u16) -> bool,
    pub step: fn(
        instr_word: u16,
        pc: &mut ProgramCounter,
        reg: &mut Register,
        mem: &mut Mem,
        step_log: &mut StepLog,
    ) -> Result<(), StepError>,
    pub get_disassembly: fn(
        instr_word: u16,
        pc: &mut ProgramCounter,
        reg: &Register,
        mem: &Mem,
        step_log: &mut StepLog,
    ) -> Result<GetDisassemblyResult, GetDisassemblyResultError>,
}

impl Instruction {
    pub fn new(
        name: String,
        mask: u16,
        opcode: u16,
        match_check: fn(instruction: &Instruction, instr_word: u16) -> bool,
        step: fn(
            instr_word: u16,
            pc: &mut ProgramCounter,
            reg: &mut Register,
            mem: &mut Mem,
            step_log: &mut StepLog,
        ) -> Result<(), StepError>,
        get_disassembly: fn(
            instr_word: u16,
            pc: &mut ProgramCounter,
            reg: &Register,
            mem: &Mem,
            step_log: &mut StepLog,
        ) -> Result<GetDisassemblyResult, GetDisassemblyResultError>,
    ) -> Instruction {
        let instr = Instruction {
            name,
            mask,
            opcode,
            ex_mask: 0x0000,
            ex_code: vec![],
            match_check,
            step,
            get_disassembly,
        };
        instr
    }
}
