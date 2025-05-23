use crate::register::*;
use crate::{cpu::instruction::*, mem::Mem};
use byteorder::{BigEndian, ReadBytesExt};
use core::panic;
use std::convert::TryInto;
use crate::aint::AInt;
use self::ea::EffectiveAddressDebug;
use self::step_log::StepLog;

pub mod ea;
pub mod instruction;
pub mod step_log;

#[derive(Debug, PartialEq)]
pub struct ResultWithStatusRegister<T> {
    pub result: T,
    pub status_register_result: StatusRegisterResult,
}

#[derive(Debug, PartialEq)]
pub struct StatusRegisterResult {
    pub status_register: u16,
    pub status_register_mask: u16,
}

impl StatusRegisterResult {
    pub fn cleared() -> StatusRegisterResult {
        StatusRegisterResult {
            status_register: 0x0000,
            status_register_mask: 0x0000,
        }
    }
}

pub enum RotateDirection {
    Right,
    Left,
}

impl RotateDirection {
    pub fn get_format(&self) -> char {
        match self {
            RotateDirection::Right => 'R',
            RotateDirection::Left => 'L',
        }
    }
}

#[allow(non_camel_case_types)]
pub enum CpuSpeed {
    PAL_7_093790_MHz,
    NTSC_7_159090_MHz,
}

impl CpuSpeed {
    pub fn get_hz(&self) -> u128 {
        match self {
            CpuSpeed::PAL_7_093790_MHz => 7_093_790,
            CpuSpeed::NTSC_7_159090_MHz => 7_159_090,
        }
    }
}

pub struct Cpu {
    pub cpu_speed: CpuSpeed,
    pub register: Register,
    pub stopped: bool,
    instructions: Vec<Instruction>,
}

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    (instr_word & instruction.mask) == instruction.opcode
    // && instruction
    //     .ex_code
    //     .contains(&(instr_word & instruction.ex_mask))
    //     == false
}

pub fn match_check_size000110_from_bit_pos_6(instr_word: u16) -> bool {
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    match operation_size {
        None => false,
        Some(_) => true,
    }
}

pub fn match_check_size011110_from_bit_pos(instr_word: u16, bit_pos: u8) -> bool {
    let operation_size = Cpu::extract_size011110_from_bit_pos(instr_word, bit_pos);
    match operation_size {
        None => false,
        Some(_) => true,
    }
}

pub fn match_check_ea_all_addressing_modes_pos_0(instr_word: u16) -> bool {
    // All addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        _ => true,
    }
}

pub fn match_check_ea_only_data_addressing_modes_pos_0(instr_word: u16) -> bool {
    // All addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_001_000..=0b_001_111 => false, // An
        _ => true,
    }
}

pub fn match_check_ea_only_alterable_addressing_modes_pos_0(instr_word: u16) -> bool {
    // Only alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_111_100 => false, // #data
        0b_111_010 => false, // (d16,PC)
        0b_111_011 => false, // (d8,PC,Xn)
        _ => true,
    }
}

pub fn match_check_ea_only_data_alterable_addressing_modes_and_areg_direct_pos(
    instr_word: u16,
    bit_pos: u8,
    reg_bit_pos: u8,
) -> bool {
    // Only data alterable addressing modes
    let mode = (instr_word >> bit_pos) & 0b_111;
    let reg = (instr_word >> reg_bit_pos) & 0b_111;
    match (mode, reg) {
        (0b_111, 0b_101) => false, // These ea modes don't exist
        (0b_111, 0b_110) => false,
        (0b_111, 0b_111) => false,
        (0b_111, 0b_100) => false, // #data
        (0b_111, 0b_010) => false, // (d16,PC)
        (0b_111, 0b_011) => false, // (d8,PC,Xn)
        _ => true,
    }
}

pub fn match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word: u16) -> bool {
    // Only data alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_001_000..=0b_001_111 => false, // An
        0b_111_100 => false,              // #data
        0b_111_010 => false,              // (d16,PC)
        0b_111_011 => false,              // (d8,PC,Xn)
        _ => true,
    }
}

pub fn match_check_ea_only_memory_alterable_addressing_modes_pos_0(instr_word: u16) -> bool {
    // Only memory alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_000_000..=0b_001_111 => false, // Dn / An
        0b_111_100 => false,              // #data
        0b_111_010 => false,              // (d16,PC)
        0b_111_011 => false,              // (d8,PC,Xn)
        _ => true,
    }
}

pub fn match_check_ea_only_control_addressing_modes_pos_0(instr_word: u16) -> bool {
    // Only memory alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_000_000..=0b_001_111 => false, // Dn / An
        0b_011_000..=0b_100_111 => false, // (An)+ / -(An)
        0b_111_100 => false,              // #data
        _ => true,
    }
}

pub fn match_check_ea_only_control_alterable_or_predecrement_addressing_modes_pos_0(
    instr_word: u16,
) -> bool {
    // Only memory alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_000_000..=0b_001_111 => false, // Dn / An
        0b_011_000..=0b_011_111 => false, // (An)+
        0b_111_100 => false,              // #data
        _ => true,
    }
}

pub fn match_check_ea_only_control_or_postincrement_addressing_modes_pos_0(
    instr_word: u16,
) -> bool {
    // Only memory alterable addressing modes
    match instr_word & 0b_111_111 {
        0b_111_101 => false, // These ea modes don't exist
        0b_111_110 => false,
        0b_111_111 => false,
        0b_000_000..=0b_001_111 => false, // Dn / An
        0b_100_000..=0b_100_111 => false, // -(An)
        0b_111_100 => false,              // #data
        _ => true,
    }
}

pub fn match_check_ea_1011111__110_00_ea(
    instr_word: u16,
    mode_bit_pos: u8,
    reg_bit_pos: u8,
) -> bool {
    let mode = (instr_word >> mode_bit_pos) & 0b_111;
    let reg = (instr_word >> reg_bit_pos) & 0b_111;
    match (mode, reg) {
        (0b_000, 0b_000..=0b_111) => true, // Dn
        (0b_001, 0b_000..=0b_111) => false, // An
        (0b_010, 0b_000..=0b_111) => true, // (An)
        (0b_011, 0b_000..=0b_111) => true, // (An)+
        (0b_100, 0b_000..=0b_111) => true, // -(An)
        (0b_101, 0b_000..=0b_111) => true, // (d16,An)
        (0b_110, 0b_000..=0b_111) => true, // (d8,An,Xn)

        (0b_111, 0b_000) => true, // (xxx).W
        (0b_111, 0b_001) => true, // (xxx).L
        (0b_111, 0b_100) => false, // #<data>

        (0b_111, 0b_010) => false, // (d16,PC)
        (0b_111, 0b_011) => false, // (d8,PC,Xn)

        (0b_111, 0b_101) => false, // These 3 ea modes don't exist
        (0b_111, 0b_110) => false,
        (0b_111, 0b_111) => false,

        // All cases are covered above, but we're dealing with 2 x u8, so the compiler can't
        // know that.
        _ => false,
    }
}

pub fn match_check_ea_1011111__111_11_ea(
    instr_word: u16,
    mode_bit_pos: u8,
    reg_bit_pos: u8,
) -> bool {
    let mode = (instr_word >> mode_bit_pos) & 0b_111;
    let reg = (instr_word >> reg_bit_pos) & 0b_111;
    match (mode, reg) {
        (0b_000, 0b_000..=0b_111) => true, // Dn
        (0b_001, 0b_000..=0b_111) => false, // An
        (0b_010, 0b_000..=0b_111) => true, // (An)
        (0b_011, 0b_000..=0b_111) => true, // (An)+
        (0b_100, 0b_000..=0b_111) => true, // -(An)
        (0b_101, 0b_000..=0b_111) => true, // (d16,An)
        (0b_110, 0b_000..=0b_111) => true, // (d8,An,Xn)

        (0b_111, 0b_000) => true, // (xxx).W
        (0b_111, 0b_001) => true, // (xxx).L
        (0b_111, 0b_100) => true, // #<data>
        (0b_111, 0b_010) => true, // (d16,PC)
        (0b_111, 0b_011) => true, // (d8,PC,Xn)

        (0b_111, 0b_101) => false, // These 3 ea modes don't exist
        (0b_111, 0b_110) => false,
        (0b_111, 0b_111) => false,

        // All cases are covered above, but we're dealing with 2 x u8, so the compiler can't
        // know that.
        _ => false,
    }
}

impl Cpu {
    pub fn new(cpu_speed: CpuSpeed, ssp_address: u32,  pc_address: u32) -> Cpu {
        let instructions = vec![
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                instruction::add::match_check,
                instruction::add::step,
                instruction::add::get_disassembly,
            ),
            Instruction::new(
                String::from("ADDI"),
                0xff00,
                0x0600,
                instruction::addi::match_check,
                instruction::addi::step,
                instruction::addi::get_disassembly,
            ),
            Instruction::new(
                String::from("ADDQ"),
                0xf100,
                0x5000,
                instruction::addq::match_check,
                instruction::addq::step,
                instruction::addq::get_disassembly,
            ),
            Instruction::new(
                String::from("ADDX"),
                0xf130,
                0xd100,
                instruction::addx::match_check,
                instruction::addx::step,
                instruction::addx::get_disassembly,
            ),
            Instruction::new(
                String::from("AND"),
                0xf000,
                0xc000,
                instruction::and::match_check,
                instruction::and::step,
                instruction::and::get_disassembly,
            ),
            Instruction::new(
                String::from("ANDI"),
                0xff00,
                0x0200,
                instruction::andi::match_check,
                instruction::andi::step,
                instruction::andi::get_disassembly,
            ),
            Instruction::new(
                String::from("ANDI to SR"),
                0xffff,
                0x027c,
                crate::cpu::match_check,
                instruction::andi_to_sr::step,
                instruction::andi_to_sr::get_disassembly,
            ),
            Instruction::new(
                String::from("ASLR"),
                0xf018,
                0xe000,
                instruction::aslr::match_check_register,
                instruction::aslr::step,
                instruction::aslr::get_disassembly,
            ),
            Instruction::new(
                String::from("ASLR"),
                0xfec0,
                0xe0c0,
                instruction::aslr::match_check_memory,
                instruction::aslr::step,
                instruction::aslr::get_disassembly,
            ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                instruction::bcc::match_check,
                instruction::bcc::step,
                instruction::bcc::get_disassembly,
            ),
            Instruction::new(
                String::from("BCLR"), // Bit Number Dynamic
                0xf1c0,
                0x0180,
                instruction::bclr::match_check,
                instruction::bclr::step_dynamic,
                instruction::bclr::get_disassembly_dynamic,
            ),
            Instruction::new(
                String::from("BCLR"), // Bit Number Static
                0xffc0,
                0x0880,
                instruction::bclr::match_check,
                instruction::bclr::step_static,
                instruction::bclr::get_disassembly_static,
            ),
            Instruction::new(
                String::from("BRA"),
                0xff00,
                0x6000,
                crate::cpu::match_check,
                instruction::bra::step,
                instruction::bra::get_disassembly,
            ),
            Instruction::new(
                String::from("BSET"), // Bit Number Dynamic
                0xf1c0,
                0x01c0,
                instruction::bset::match_check,
                instruction::bset::step_dynamic,
                instruction::bset::get_disassembly_dynamic,
            ),
            Instruction::new(
                String::from("BSET"), // Bit Number Static
                0xffc0,
                0x08c0,
                instruction::bset::match_check,
                instruction::bset::step_static,
                instruction::bset::get_disassembly_static,
            ),
            Instruction::new(
                String::from("BSR"),
                0xff00,
                0x6100,
                crate::cpu::match_check,
                instruction::bsr::step,
                instruction::bsr::get_disassembly,
            ),
            Instruction::new(
                String::from("BTST"), // Bit Number Dynamic
                0xf1c0,
                0x0100,
                instruction::btst::match_check,
                instruction::btst::step_dynamic,
                instruction::btst::get_disassembly_dynamic,
            ),
            Instruction::new(
                String::from("BTST"), // Bit Number Static
                0xffc0,
                0x0800,
                instruction::btst::match_check,
                instruction::btst::step_static,
                instruction::btst::get_disassembly_static,
            ),
            Instruction::new(
                String::from("CLR"),
                0xff00,
                0x4200,
                instruction::clr::match_check,
                instruction::clr::step,
                instruction::clr::get_disassembly,
            ),
            Instruction::new(
                String::from("CMPM"),
                0xf138,
                0xb108,
                instruction::cmpm::match_check,
                instruction::cmpm::step,
                instruction::cmpm::get_disassembly,
            ),
            Instruction::new(
                String::from("CMP"),
                0xb000,
                0xb000,
                // TODO: match_check
                instruction::cmp::match_check,
                instruction::cmp::step,
                instruction::cmp::get_disassembly,
            ),
            Instruction::new(
                String::from("CMPI"),
                0xff00,
                0x0c00,
                instruction::cmpi::match_check,
                instruction::cmpi::step,
                instruction::cmpi::get_disassembly,
            ),
            Instruction::new(
                String::from("DBcc"),
                0xf0f8,
                0x50c8,
                crate::cpu::match_check,
                instruction::dbcc::step,
                instruction::dbcc::get_disassembly,
            ),
            Instruction::new(
                String::from("DIVS"),
                0xf1c0,
                0x81c0,
                instruction::divs::match_check,
                instruction::divs::step,
                instruction::divs::get_disassembly,
            ),
            Instruction::new(
                String::from("DIVU"),
                0xf1c0,
                0x80c0,
                instruction::divu::match_check,
                instruction::divu::step,
                instruction::divu::get_disassembly,
            ),
            Instruction::new(
                String::from("EOR"),
                0xf000,
                0xb000,
                instruction::eor::match_check,
                instruction::eor::step,
                instruction::eor::get_disassembly,
            ),
            Instruction::new(
                String::from("EXG"),
                0xf100,
                0xc100,
                instruction::exg::match_check,
                instruction::exg::step,
                instruction::exg::get_disassembly,
            ),
            Instruction::new(
                String::from("EXT"),
                0xfe38,
                0x4800,
                instruction::ext::match_check,
                instruction::ext::step,
                instruction::ext::get_disassembly,
            ),
            Instruction::new(
                String::from("JMP"),
                0xffc0,
                0x4ec0,
                instruction::jmp::match_check,
                instruction::jmp::step,
                instruction::jmp::get_disassembly,
            ),
            Instruction::new(
                String::from("JSR"),
                0xffc0,
                0x4e80,
                instruction::jsr::match_check,
                instruction::jsr::step,
                instruction::jsr::get_disassembly,
            ),
            Instruction::new(
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                instruction::lea::match_check,
                instruction::lea::step,
                instruction::lea::get_disassembly,
            ),
            Instruction::new(
                String::from("LINK"), // word
                0xfff8,
                0x4e50,
                crate::cpu::match_check,
                instruction::link::step,
                instruction::link::get_disassembly,
            ),
            Instruction::new(
                String::from("LINK"), // long
                0xfff8,
                0x4808,
                crate::cpu::match_check,
                instruction::link::step_long,
                instruction::link::get_disassembly_long,
            ),
            Instruction::new(
                String::from("LSLR"), // register
                0xf018,
                0xe008,
                instruction::lslr::match_check_register,
                instruction::lslr::step,
                instruction::lslr::get_disassembly,
            ),
            Instruction::new(
                String::from("LSLR"), // memory
                0xfec0,
                0xe2c0,
                instruction::lslr::match_check_memory,
                instruction::lslr::step,
                instruction::lslr::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVE"),
                0xc000,
                0x0000,
                instruction::mov::match_check,
                instruction::mov::step,
                instruction::mov::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVE from SR"),
                0xffc0,
                0x40c0,
                instruction::move_from_sr::match_check,
                instruction::move_from_sr::step,
                instruction::move_from_sr::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEC"),
                0xfffe,
                0x4e7a,
                crate::cpu::match_check,
                instruction::movec::step,
                instruction::movec::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEM"),
                0xfb80,
                0x4880,
                instruction::movem::match_check,
                instruction::movem::step,
                instruction::movem::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                crate::cpu::match_check,
                instruction::moveq::step,
                instruction::moveq::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVE to SR"),
                0xffc0,
                0x46c0,
                instruction::move_to_sr::match_check,
                instruction::move_to_sr::step,
                instruction::move_to_sr::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVE USP"),
                0xfff0,
                0x4e60,
                crate::cpu::match_check,
                instruction::move_usp::step,
                instruction::move_usp::get_disassembly,
            ),
            Instruction::new(
                String::from("MULU"),
                0xf1c0,
                0xc0c0,
                instruction::mulu::match_check,
                instruction::mulu::step,
                instruction::mulu::get_disassembly,
            ),
            Instruction::new(
                String::from("NEG"),
                0xff00,
                0x4400,
                instruction::neg::match_check,
                instruction::neg::step,
                instruction::neg::get_disassembly,
            ),
            Instruction::new(
                String::from("NOP"),
                0xffff,
                0x4e71,
                crate::cpu::match_check,
                instruction::nop::step,
                instruction::nop::get_disassembly,
            ),
            Instruction::new(
                String::from("NOT"),
                0xff00,
                0x4600,
                instruction::not::match_check,
                instruction::not::step,
                instruction::not::get_disassembly,
            ),
            Instruction::new(
                String::from("OR"),
                0xf000,
                0x8000,
                instruction::or::match_check,
                instruction::or::step,
                instruction::or::get_disassembly,
            ),
            Instruction::new(
                String::from("ORI"),
                0xff00,
                0x0000,
                instruction::ori::match_check,
                instruction::ori::step,
                instruction::ori::get_disassembly,
            ),
            Instruction::new(
                String::from("ORI to SR"),
                0xffff,
                0x007c,
                crate::cpu::match_check,
                instruction::ori_to_sr::step,
                instruction::ori_to_sr::get_disassembly,
            ),
            Instruction::new(
                String::from("PEA"),
                0xffc0,
                0x4840,
                instruction::pea::match_check,
                instruction::pea::step,
                instruction::pea::get_disassembly,
            ),
            Instruction::new(
                String::from("RESET"),
                0xffff,
                0x4e70,
                crate::cpu::match_check,
                instruction::reset::step,
                instruction::reset::get_disassembly,
            ),
            Instruction::new(
                String::from("ROLR"), // register
                0xf018,
                0xe018,
                // TODO: match_check
                crate::cpu::match_check,
                instruction::rolrreg::step,
                instruction::rolrreg::get_disassembly,
            ),
            Instruction::new(
                String::from("ROLR"), // memory
                0xfec0,
                0xe6c0,
                // TODO: match_check
                crate::cpu::match_check,
                instruction::rolrmem::step,
                instruction::rolrmem::get_disassembly,
            ),
            Instruction::new(
                String::from("RTS"),
                0xffff,
                0x4e75,
                crate::cpu::match_check,
                instruction::rts::step,
                instruction::rts::get_disassembly,
            ),
            Instruction::new(
                String::from("RTE"),
                0xffff,
                0x4e73,
                crate::cpu::match_check,
                instruction::rte::step,
                instruction::rte::get_disassembly,
            ),
            Instruction::new(
                String::from("Scc"),
                0xf0c0,
                0x50c0,
                instruction::scc::match_check,
                instruction::scc::step,
                instruction::scc::get_disassembly,
            ),
            Instruction::new(
                String::from("STOP"),
                0xffff,
                0x4e72,
                crate::cpu::match_check,
                instruction::stop::step,
                instruction::stop::get_disassembly,
            ),
            Instruction::new(
                String::from("SUB"),
                0xf000,
                0x9000,
                instruction::sub::match_check,
                instruction::sub::step,
                instruction::sub::get_disassembly,
            ),
            Instruction::new(
                String::from("SUBI"),
                0xff00,
                0x0400,
                instruction::subi::match_check,
                instruction::subi::step,
                instruction::subi::get_disassembly,
            ),
            Instruction::new(
                String::from("SUBQ"),
                0xf100,
                0x5100,
                instruction::subq::match_check,
                instruction::subq::step,
                instruction::subq::get_disassembly,
            ),
            Instruction::new(
                String::from("SUBX"),
                0xf130,
                0x9100,
                instruction::subx::match_check,
                instruction::subx::step,
                instruction::subx::get_disassembly,
            ),
            Instruction::new(
                String::from("SWAP"),
                0xfff8,
                0x4840,
                crate::cpu::match_check,
                instruction::swap::step,
                instruction::swap::get_disassembly,
            ),
            Instruction::new(
                String::from("TST"),
                0xff00,
                0x4a00,
                instruction::tst::match_check,
                instruction::tst::step,
                instruction::tst::get_disassembly,
            ),
            Instruction::new(
                String::from("UNLK"),
                0xfff8,
                0x4e58,
                crate::cpu::match_check,
                instruction::unlk::step,
                instruction::unlk::get_disassembly,
            ),
        ];
        let reg_pc = ProgramCounter::from_address(pc_address);
        let mut register = Register::new();
        register.set_ssp_reg(ssp_address);
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            cpu_speed,
            register,
            stopped: false,
            instructions,
        };
        cpu
    }

    pub fn sign_extend_byte(value: u8) -> u16 {
        // TODO: Any better way to do this?
        let address_bytes = value.to_be_bytes();
        // if address < 0
        let fixed_bytes: [u8; 2] = if value >= 0x80 {
            [0xff, address_bytes[0]]
        } else {
            [0x00, address_bytes[0]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..2];
        let res = fixed_bytes_slice.read_u16::<BigEndian>().unwrap();
        res
    }

    pub fn sign_extend_byte_to_long(value: u8) -> u32 {
        // TODO: Any better way to do this?
        let address_bytes = value.to_be_bytes();
        // if address < 0
        let fixed_bytes: [u8; 4] = if value >= 0x80 {
            [0xff, 0xff, 0xff, address_bytes[0]]
        } else {
            [0x00, 0x00, 0x00, address_bytes[0]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    pub fn sign_extend_word(value: u16) -> u32 {
        // // TODO: Any better way to do this?
        let address_bytes = value.to_be_bytes();
        // if address < 0
        let fixed_bytes: [u8; 4] = if value >= 0x8000 {
            [0xff, 0xff, address_bytes[0], address_bytes[1]]
        } else {
            [0x00, 0x00, address_bytes[0], address_bytes[1]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    pub fn get_address_with_byte_displacement_sign_extended(address: u32, displacement: u8) -> u32 {
        let displacement = Cpu::sign_extend_byte_to_long(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    pub fn get_address_with_word_displacement_sign_extended(
        address: u32,
        displacement: u16,
    ) -> u32 {
        let displacement = Cpu::sign_extend_word(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    pub fn get_address_with_long_displacement(address: u32, displacement: u32) -> u32 {
        let address = address.wrapping_add(displacement);

        address
    }

    fn extract_conditional_test_pos_8(word: u16) -> ConditionalTest {
        let conditional_test = (word >> 8) & 0x000f;
        let conditional_test = match conditional_test {
            0b0000 => ConditionalTest::T,
            0b0001 => ConditionalTest::F,
            0b0010 => ConditionalTest::HI,
            0b0011 => ConditionalTest::LS,
            0b0100 => ConditionalTest::CC,
            0b0101 => ConditionalTest::CS,
            0b0110 => ConditionalTest::NE,
            0b0111 => ConditionalTest::EQ,
            0b1000 => ConditionalTest::VC,
            0b1001 => ConditionalTest::VS,
            0b1010 => ConditionalTest::PL,
            0b1011 => ConditionalTest::MI,
            0b1100 => ConditionalTest::GE,
            0b1101 => ConditionalTest::LT,
            0b1110 => ConditionalTest::GT,
            _ => ConditionalTest::LE,
        };
        conditional_test
    }

    pub fn extract_scale_factor_from_bit_pos(word: u16, bit_pos: u8) -> ScaleFactor {
        let scale_factor = (word >> bit_pos) & 0b0011;
        match scale_factor {
            0b00 => ScaleFactor::One,
            0b01 => ScaleFactor::Two,
            0b10 => ScaleFactor::Four,
            _ => ScaleFactor::Eight,
        }
    }

    fn extract_op_mode_from_bit_pos_6(word: u16) -> usize {
        let op_mode = (word >> 6) & 0x0007;
        let op_mode = match op_mode {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            _ => 7,
        };
        op_mode
    }

    pub fn extract_register_index_from_bit_pos(
        word: u16,
        bit_pos: u8,
    ) -> Result<usize, InstructionError> {
        let register = (word >> bit_pos) & 0x0007;
        let result = match register.try_into() {
            Ok(register_index) => {
                if register_index <= 7 {
                    Ok(register_index)
                } else {
                    Err(InstructionError {
                        details: format!(
                            "Failed to extract register index from bit pos {}, got index: {}",
                            bit_pos, register_index
                        ),
                    })
                }
            }
            Err(a) => Err(InstructionError {
                details: format!("Failed to extract register index from bit pos {}", bit_pos),
            }),
        };

        result
    }

    pub fn extract_register_index_from_bit_pos_0(word: u16) -> Result<usize, InstructionError> {
        let register = word & 0x0007;
        let result = match register.try_into() {
            Ok(register_index) => {
                if register_index <= 7 {
                    Ok(register_index)
                } else {
                    Err(InstructionError {
                        details: format!(
                            "Failed to extract register index from bit pos 0, got index: {}",
                            register_index
                        ),
                    })
                }
            }
            Err(e) => Err(InstructionError {
                details: format!("Failed to extract register index from bit pos 0"),
            }),
        };
        result
    }

    pub fn extract_size000110_from_bit_pos_6(word: u16) -> Option<OperationSize> {
        let size = (word >> 6) & 0x0003;
        match size {
            0b00 => Some(OperationSize::Byte),
            0b01 => Some(OperationSize::Word),
            0b10 => Some(OperationSize::Long),
            _ => None,
        }
    }

    pub fn extract_size011110_from_bit_pos(word: u16, bit_pos: u8) -> Option<OperationSize> {
        let size = (word >> bit_pos) & 0x0003;
        match size {
            0b01 => Some(OperationSize::Byte),
            0b11 => Some(OperationSize::Word),
            0b10 => Some(OperationSize::Long),
            _ => None,
        }
    }

    pub fn extract_3_bit_data_1_to_8_from_word_at_pos(word: u16, bit_pos: u8) -> u8 {
        let data = ((word >> bit_pos) & 0b111) as u8;
        match data {
            0 => 8,
            _ => data,
        }
    }

    pub fn get_byte_from_long(long: u32) -> u8 {
        (long & 0x000000ff) as u8
    }

    pub fn get_byte_from_word(word: u16) -> u8 {
        (word & 0x00ff) as u8
    }

    pub fn get_word_from_long(long: u32) -> u16 {
        (long & 0x0000ffff) as u16
    }

    pub fn get_signed_byte_from_byte(byte: u8) -> i8 {
        byte as i8
    }

    pub fn get_signed_word_from_word(long: u16) -> i16 {
        long as i16
    }

    pub fn get_signed_long_from_long(long: u32) -> i32 {
        long as i32
    }

    pub fn set_byte_in_long(value: u8, long: u32) -> u32 {
        (long & 0xffffff00) | (value as u32)
    }

    pub fn set_word_in_long(value: u16, long: u32) -> u32 {
        (long & 0xffff0000) | (value as u32)
    }

    pub fn add_bytes(source: u8, dest: u8) -> ResultWithStatusRegister<u8> {
        let source_signed = Cpu::get_signed_byte_from_byte(source);
        let dest_signed = Cpu::get_signed_byte_from_byte(dest);

        let (result, carry) = source.overflowing_add(dest);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn add_bytes_with_extend(
        source: u8,
        dest: u8,
        extend: bool,
    ) -> ResultWithStatusRegister<u8> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_byte_from_byte(source);
        let dest_signed = Cpu::get_signed_byte_from_byte(dest);
        let extend_signed = extend as i8;

        let (result, carry) = source.overflowing_add(dest);
        let (result, carry2) = result.overflowing_add(extend);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);
        let (result_signed, overflow2) = result_signed.overflowing_add(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn add_words(source: u16, dest: u16) -> ResultWithStatusRegister<u16> {
        let source_signed = Cpu::get_signed_word_from_word(source);
        let dest_signed = Cpu::get_signed_word_from_word(dest);

        let (result, carry) = source.overflowing_add(dest);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn add_words_with_extend(
        source: u16,
        dest: u16,
        extend: bool,
    ) -> ResultWithStatusRegister<u16> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_word_from_word(source);
        let dest_signed = Cpu::get_signed_word_from_word(dest);
        let extend_signed = extend as i16;

        let (result, carry) = source.overflowing_add(dest);
        let (result, carry2) = result.overflowing_add(extend);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);
        let (result_signed, overflow2) = result_signed.overflowing_add(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn add_longs(source: u32, dest: u32) -> ResultWithStatusRegister<u32> {
        let source_signed = Cpu::get_signed_long_from_long(source);
        let dest_signed = Cpu::get_signed_long_from_long(dest);

        let (result, carry) = source.overflowing_add(dest);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i32::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn add_longs_with_extend(
        source: u32,
        dest: u32,
        extend: bool,
    ) -> ResultWithStatusRegister<u32> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_long_from_long(source);
        let dest_signed = Cpu::get_signed_long_from_long(dest);
        let extend_signed = extend as i32;

        let (result, carry) = source.overflowing_add(dest);
        let (result, carry2) = result.overflowing_add(extend);
        let (result_signed, overflow) = source_signed.overflowing_add(dest_signed);
        let (result_signed, overflow2) = result_signed.overflowing_add(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i32::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn and_bytes(source: u8, dest: u8) -> ResultWithStatusRegister<u8> {
        let result = source & dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn eor_bytes(source: u8, dest: u8) -> ResultWithStatusRegister<u8> {
        let result = source ^ dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn eor_words(source: u16, dest: u16) -> ResultWithStatusRegister<u16> {
        let result = source ^ dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn eor_longs(source: u32, dest: u32) -> ResultWithStatusRegister<u32> {
        let result = source ^ dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn or_bytes(source: u8, dest: u8) -> ResultWithStatusRegister<u8> {
        let result = source | dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn and_words(source: u16, dest: u16) -> ResultWithStatusRegister<u16> {
        let result = source & dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn or_words(source: u16, dest: u16) -> ResultWithStatusRegister<u16> {
        let result = source | dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn and_longs(source: u32, dest: u32) -> ResultWithStatusRegister<u32> {
        let result = source & dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn or_longs(source: u32, dest: u32) -> ResultWithStatusRegister<u32> {
        let result = source | dest;

        let mut status_register = 0x0000;
        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn join_words_to_long(hi: u16, low: u16) -> u32 {
        ((hi as u32) << 16) | (low as u32)
    }

    pub fn sub_bytes(source: u8, dest: u8) -> ResultWithStatusRegister<u8> {
        let source_signed = Cpu::get_signed_byte_from_byte(source);
        let dest_signed = Cpu::get_signed_byte_from_byte(dest);

        let (result, carry) = dest.overflowing_sub(source);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn sub_bytes_with_extend(
        source: u8,
        dest: u8,
        extend: bool,
    ) -> ResultWithStatusRegister<u8> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_byte_from_byte(source);
        let dest_signed = Cpu::get_signed_byte_from_byte(dest);
        let extend_signed = extend as i8;

        let (result, carry) = dest.overflowing_sub(source);
        let (result, carry2) = result.overflowing_sub(extend);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);
        let (result_signed, overflow2) = result_signed.overflowing_sub(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn sub_words(source: u16, dest: u16) -> ResultWithStatusRegister<u16> {
        let source_signed = Cpu::get_signed_word_from_word(source);
        let dest_signed = Cpu::get_signed_word_from_word(dest);

        let (result, carry) = dest.overflowing_sub(source);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn sub_words_with_extend(
        source: u16,
        dest: u16,
        extend: bool,
    ) -> ResultWithStatusRegister<u16> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_word_from_word(source);
        let dest_signed = Cpu::get_signed_word_from_word(dest);
        let extend_signed = extend as i16;

        let (result, carry) = dest.overflowing_sub(source);
        let (result, carry2) = result.overflowing_sub(extend);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);
        let (result_signed, overflow2) = result_signed.overflowing_sub(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn sub_longs(source: u32, dest: u32) -> ResultWithStatusRegister<u32> {
        let source_signed = Cpu::get_signed_long_from_long(source);
        let dest_signed = Cpu::get_signed_long_from_long(dest);

        let (result, carry) = dest.overflowing_sub(source);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);

        let mut status_register = 0x0000;
        match carry {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i32::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn sub_longs_with_extend(
        source: u32,
        dest: u32,
        extend: bool,
    ) -> ResultWithStatusRegister<u32> {
        let extend = match extend {
            true => 1,
            false => 0,
        };
        let source_signed = Cpu::get_signed_long_from_long(source);
        let dest_signed = Cpu::get_signed_long_from_long(dest);
        let extend_signed = extend as i32;

        let (result, carry) = dest.overflowing_sub(source);
        let (result, carry2) = result.overflowing_sub(extend);
        let (result_signed, overflow) = dest_signed.overflowing_sub(source_signed);
        let (result_signed, overflow2) = result_signed.overflowing_sub(extend_signed);

        let mut status_register_mask = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        let mut status_register = 0x0000;
        match carry | carry2 {
            true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            false => (),
        }
        match overflow | overflow2 {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => {
                status_register_mask &= STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_NEGATIVE
            }
            i32::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask,
            },
        }
    }

    pub fn neg_byte(dest: u8) -> ResultWithStatusRegister<u8> {
        let dest_signed = Cpu::get_signed_byte_from_byte(dest);

        let (result, _) = u8::overflowing_sub(0, dest);
        let (result_signed, overflow) = i8::overflowing_sub(0, dest_signed);

        let mut status_register = 0x0000;
        // match carry {
        //     true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        //     false => (),
        // }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => {
                status_register |= STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
            }
            _ => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn neg_word(dest: u16) -> ResultWithStatusRegister<u16> {
        let dest_signed = Cpu::get_signed_word_from_word(dest);

        let (result, _) = u16::overflowing_sub(0, dest);
        let (result_signed, overflow) = i16::overflowing_sub(0, dest_signed);

        let mut status_register = 0x0000;
        // match carry {
        //     true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        //     false => (),
        // }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i16::MIN..=-1 => {
                status_register |= STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
            }
            _ => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn neg_long(dest: u32) -> ResultWithStatusRegister<u32> {
        let dest_signed = Cpu::get_signed_long_from_long(dest);

        let (result, _) = u32::overflowing_sub(0, dest);
        let (result_signed, overflow) = i32::overflowing_sub(0, dest_signed);

        let mut status_register = 0x0000;
        // match carry {
        //     true => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        //     false => (),
        // }
        match overflow {
            true => status_register |= STATUS_REGISTER_MASK_OVERFLOW,
            false => (),
        }
        match result_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i32::MIN..=-1 => {
                status_register |= STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
            }
            _ => status_register |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_EXTEND
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn not_byte(dest: u8) -> ResultWithStatusRegister<u8> {
        let result = !dest;

        let mut status_register = 0x0000;

        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80..=0xff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn not_word(dest: u16) -> ResultWithStatusRegister<u16> {
        let result = !dest;

        let mut status_register = 0x0000;

        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn not_long(dest: u32) -> ResultWithStatusRegister<u32> {
        let result = !dest;

        let mut status_register = 0x0000;

        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn shift_arithmetic<T: AInt>(
        value: T,
        direction: RotateDirection,
        shift_count: u32,
    ) -> (T, StatusRegisterResult) {
        if shift_count > 63 {
            std::panic!("arithmetic_shift_byte: shift count is larger than 63");
        }

        // X -- Set according to the last bit shifted out of the operand; unaffacted for a count of zero
        // N -- Set if the most significant bit of the result is set; cleared otherwise
        // Z -- Set if the reuslt is zero; cleared otherwise
        // V -- Set if the most significant bit is changed at any time during the shift operation; cleared otherwise
        // C -- Set according to the last bit shifted out of the operand; cleared for a count of zero
        let mut status_register = 0x0000;
        let mut status_register_mask = STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_CARRY;
        let result = match shift_count {
            0 => value,
            shift_count => {
                status_register_mask |= STATUS_REGISTER_MASK_EXTEND;
                let mut result = value;
                match direction {
                    RotateDirection::Left => {
                        for _ in 0..shift_count {
                            // X -- Set according to the last bit shifted out of the operand; unaffacted for a count of zero
                            // C -- Set according to the last bit shifted out of the operand; cleared for a count of zero
                            if result.is_msb_set() {
                                status_register = (status_register
                                    & (STATUS_REGISTER_MASK_OVERFLOW
                                        | STATUS_REGISTER_MASK_ZERO
                                        | STATUS_REGISTER_MASK_NEGATIVE))
                                    | STATUS_REGISTER_MASK_EXTEND
                                    | STATUS_REGISTER_MASK_CARRY;
                            } else {
                                status_register = status_register
                                    & (STATUS_REGISTER_MASK_OVERFLOW
                                        | STATUS_REGISTER_MASK_ZERO
                                        | STATUS_REGISTER_MASK_NEGATIVE);
                            }
                            // Shift left
                            let new_result = result.checked_shift_left(1).unwrap_or(T::zero());
                            // V -- Set if the most significant bit is changed at any time during the shift operation; cleared otherwise
                            if new_result.is_msb_set() != result.is_msb_set() {
                                status_register |= STATUS_REGISTER_MASK_OVERFLOW;
                            }
                            result = new_result;
                        }
                    }
                    RotateDirection::Right => {
                        for _ in 0..shift_count {
                            // X -- Set according to the last bit shifted out of the operand; unaffacted for a count of zero
                            // C -- Set according to the last bit shifted out of the operand; cleared for a count of zero
                            if result.is_lsb_set() {
                                status_register = (status_register
                                    & (STATUS_REGISTER_MASK_OVERFLOW
                                        | STATUS_REGISTER_MASK_ZERO
                                        | STATUS_REGISTER_MASK_NEGATIVE))
                                    | STATUS_REGISTER_MASK_EXTEND
                                    | STATUS_REGISTER_MASK_CARRY;
                            } else {
                                status_register = status_register
                                    & (STATUS_REGISTER_MASK_OVERFLOW
                                        | STATUS_REGISTER_MASK_ZERO
                                        | STATUS_REGISTER_MASK_NEGATIVE);
                            }
                            // Shift right
                            let mut new_result = result.checked_shift_right(1).unwrap_or(T::zero());
                            // Keep original sign
                            new_result = new_result.bit_or(value.get_msb_mask());
                            // V -- Set if the most significant bit is changed at any time during the shift operation; cleared otherwise
                            //      Will never change when shifting right as MSB is retained through an arithmetic shift
                            result = new_result;
                        }
                    }
                }
                result
            }
        };
        if result.is_zero() {
            status_register |= STATUS_REGISTER_MASK_ZERO;
        } else if result.is_msb_set() {
            status_register |= STATUS_REGISTER_MASK_NEGATIVE;
        }

        let status_register_result = StatusRegisterResult {
            status_register,
            status_register_mask,
        };
        (result, status_register_result)
    }

    pub fn divs_long_by_word(source: u16, dest: u32) -> ResultWithStatusRegister<u32> {
        let source = Cpu::sign_extend_word(source) as i32;
        let dest_signed = dest as i32;
        let quotient_i32 = (dest_signed / source) as i32;
        let quotient_u16 = quotient_i32 as u16;
        let remainder = (dest_signed % source) as u16;

        let mut status_register = 0x0000;

        let result;
        if quotient_i32 < 0x00010000 {
            match quotient_u16 {
                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            result = Self::join_words_to_long(remainder, quotient_u16);
        } else {
            status_register |= STATUS_REGISTER_MASK_OVERFLOW;
            result = dest;
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn divu_long_by_word(source: u16, dest: u32) -> ResultWithStatusRegister<u32> {
        let source = source as u32;
        let quotient_u32 = dest / source;
        let quotient_u16 = quotient_u32 as u16;
        let remainder = (dest % source) as u16;

        let mut status_register = 0x0000;

        let result;
        if quotient_u32 < 0x00010000 {
            match quotient_u16 {
                0 => status_register |= STATUS_REGISTER_MASK_ZERO,
                0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            result = Self::join_words_to_long(remainder, quotient_u16);
        } else {
            status_register |= STATUS_REGISTER_MASK_OVERFLOW;
            result = dest;
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn mulu_words(source: u16, dest: u16) -> ResultWithStatusRegister<u32> {
        let source = source as u32;
        let dest = dest as u32;

        let result = source * dest;

        let mut status_register = 0x0000;

        match result {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            0x80000000..=0xffffffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        ResultWithStatusRegister {
            result,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn get_ea_format(
        ea_mode: EffectiveAddressingMode,
        pc: &mut ProgramCounter,
        operation_size: Option<OperationSize>,
        mem: &Mem,
    ) -> EffectiveAddressDebug {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect {
                ea_register: register,
            } => {
                // Dn
                let format = format!("D{}", register);
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                let format = format!("A{}", register);
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ARegIndirect {
                ea_register: register,
                ea_address: address,
            } => {
                // (An)
                let format = format!("(A{})", register);
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                operation_size,
                ea_register,
            } => {
                // (An)+
                let format = format!("(A{})+", ea_register);
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                operation_size,
                ea_register,
            } => {
                // (-An)
                let format = format!("-(A{})", ea_register);
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement {
                ea_register: register,
                ea_address: address,
                ea_displacement: displacement,
            } => {
                // (d16,An)
                let format = format!(
                    "(${:04X},A{})",
                    displacement,
                    register,
                    // Cpu::get_signed_long_from_long(Cpu::sign_extend_word(displacement))
                );
                EffectiveAddressDebug { format }
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
                // ARegIndirectWithIndex8BitDisplacement (d8, An, Xn.SIZE*SCALE)
                // ARegIndirectWithIndexBaseDisplacement (bd, An, Xn.SIZE*SCALE)
                // MemoryIndirectPostIndexed             ([bd, An], Xn.SIZE*SCALE,od)
                // MemoryIndirectPreIndexed              ([bd, An, Xn.SIZE*SCALE],od)
                let register_type = match register_type {
                    RegisterType::Address => 'A',
                    RegisterType::Data => 'D',
                };
                let index_size = index_size.get_format();

                let displacement = Cpu::get_signed_byte_from_byte(displacement);

                let format = format!(
                    "(${:02X},A{},{}{}.{}{})",
                    displacement,
                    ea_register,
                    register_type,
                    register,
                    index_size,
                    scale_factor,
                    // displacement
                );
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::PcIndirectWithDisplacement {
                ea_address,
                displacement,
            } => {
                // (d16,PC)
                let format = format!("(${:04X},PC)", displacement);

                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::AbsoluteShortAddressing {
                ea_address,
                displacement,
            } => {
                // (xxx).W
                let format = match displacement > 0x8000 {
                    false => format!("(${:04X}).W", displacement),
                    true => format!("(${:04X}).W", displacement),
                };
                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::AbsolutLongAddressing { ea_address } => {
                // (xxx).L
                let format = format!("(${:08X}).L", ea_address);
                EffectiveAddressDebug { format }
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
                // PcIndirectWithIndex8BitDisplacement (d8, PC, Xn.SIZE*SCALE)
                // PcIndirectWithIndexBaseDisplacement (bd, PC, Xn.SIZE*SCALE)
                // PcMemoryInderectPostIndexed         ([bd, PC], Xn.SIZE*SCALE,od)
                // PcMemoryInderectPreIndexed          ([bd, PC, Xn.SIZE*SCALE],od)
                let index_size_format = index_size.get_format();

                let register_type_format = match register_type {
                    RegisterType::Data => 'D',
                    RegisterType::Address => 'A',
                };

                let format = format!(
                    "(${:02X},PC,{}{}.{}{})",
                    displacement, register_type_format, register, index_size_format, scale_factor
                );

                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::ImmediateDataByte { data } => {
                // #<xxx>
                EffectiveAddressDebug {
                    format: format!("#${:02X}", data),
                }
            }
            EffectiveAddressingMode::ImmediateDataWord { data } => EffectiveAddressDebug {
                format: format!("#${:04X}", data),
            },
            EffectiveAddressingMode::ImmediateDataLong { data } => EffectiveAddressDebug {
                format: format!("#${:08X}", data),
            },
        }
    }

    pub fn exception(
        &mut self,
        pc: &mut ProgramCounter,
        mem: &mut Mem,
        step_log: &mut StepLog,
        vector: u32,
    ) {
        // TODO: This could probably be a StepLogEntry
        println!("Exception occured!");
        println!("vector: {} [${:02X}]", vector, vector);
        let sr_to_push = self.register.reg_sr.get_value();
        self.register.reg_sr.set_supervisor();
        // TODO: T bit clear
        // TODO: Do we push PC or PC+2?
        self.register.stack_push_pc(mem, step_log);
        self.register.stack_push_word(mem, step_log, sr_to_push);

        let vector_offset = (vector) * 4;
        println!("vector_offset: ${:08X}", vector_offset);
        let vector_address = mem.get_long(step_log, vector_offset);
        println!("vector_address: ${:08X}", vector_address);
        pc.set_long(vector_address);
    }

    pub fn execute_next_instruction(self: &mut Cpu, mem: &mut Mem) {
        self.execute_next_instruction_step_log(mem, &mut StepLog::none())
    }

    pub fn execute_next_instruction_step_log(
        self: &mut Cpu,
        mem: &mut Mem,
        step_log: &mut StepLog,
    ) {
        if self.stopped == true {
            return;
        }
        let mut pc = self.register.reg_pc.clone();
        let instr_word = pc.fetch_next_word(mem);

        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| match_check(x, instr_word) && (x.match_check)(x, instr_word));
        match instruction_pos {
            None => {
                panic!(
                    "{:#010X} Unidentified instruction {:#06X}",
                    pc.get_address(),
                    instr_word
                );
                // self.exception(&mut pc, 4);
                // self.register.reg_pc = pc.get_step_next_pc();
            }
            Some(instruction_pos) => {
                let instruction = &self.instructions[instruction_pos];
                let step_result =
                    (instruction.step)(instr_word, &mut pc, &mut self.register, mem, step_log);
                match step_result {
                    Ok(step_result) => self.register.reg_pc = pc.get_step_next_pc(),
                    Err(step_error) => match step_error {
                        StepError::IllegalInstruction => {
                            self.exception(&mut pc, mem, step_log, 4);
                            self.register.reg_pc = pc.get_step_next_pc();
                        }
                        StepError::PriviliegeViolation => {
                            self.exception(&mut pc, mem, step_log, 8);
                            self.register.reg_pc = pc.get_step_next_pc();
                        }
                        StepError::Stop => {
                            println!("STOP:ing CPU instruction excecution");
                            self.stopped = true;
                            self.register.reg_pc = pc.get_step_next_pc();
                        }
                        _ => {
                            println!("Runtime error occured when running instruction.");
                            println!(
                                " Instruction word: ${:04X} ({}) at address ${:08X}",
                                instr_word,
                                instruction.name,
                                pc.get_address()
                            );
                            println!(" Error: {}", step_error);
                            self.register.print_registers();
                            panic!();
                        }
                    },
                }
            }
        };
    }

    pub fn get_next_disassembly_no_log(self: &mut Cpu, mem: &mut Mem) -> GetDisassemblyResult {
        self.get_next_disassembly(mem, &mut StepLog::none())
    }

    pub fn get_next_disassembly(
        self: &mut Cpu,
        mem: &mut Mem,
        step_log: &mut StepLog,
    ) -> GetDisassemblyResult {
        let get_disassembly_result =
            self.get_disassembly(&mut self.register.reg_pc.clone(), mem, step_log);
        get_disassembly_result
    }

    pub fn get_disassembly(
        self: &mut Cpu,
        pc: &mut ProgramCounter,
        mem: &mut Mem,
        step_log: &mut StepLog,
    ) -> GetDisassemblyResult {
        let instr_word = pc.fetch_next_word(mem);

        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| match_check(x, instr_word) && (x.match_check)(x, instr_word));

        let result = match instruction_pos {
            Some(instruction_pos) => {
                let instruction = &self.instructions[instruction_pos];

                let get_disassembly_result = (instruction.get_disassembly)(
                    instr_word,
                    pc,
                    &mut self.register,
                    mem,
                    step_log,
                );

                match get_disassembly_result {
                    Ok(result) => result,
                    Err(error) => GetDisassemblyResult::from_pc(
                        pc,
                        mem,
                        String::from("DC.W"),
                        format!(
                            "#${:04X} ; Error when getting disassembly from instruction: {}",
                            instr_word, error.details
                        ),
                    ),
                }
            }
            None => GetDisassemblyResult::from_pc(
                pc,
                mem,
                String::from("DC.W"),
                format!("#${:04X}", instr_word),
            ),
        };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_byte_positive() {
        let res = Cpu::sign_extend_byte_to_long(45);
        assert_eq!(45, res);
    }

    #[test]
    fn sign_extend_byte_negative() {
        let res = Cpu::sign_extend_byte_to_long(0xd3); // -45
        assert_eq!(0xFFFFFFD3, res);
    }

    #[test]
    fn sign_extend_byte_negative2() {
        let res = Cpu::sign_extend_byte_to_long(0xff); // -1
        assert_eq!(0xFFFFFFFF, res);
    }

    #[test]
    fn sign_extend_word_positive() {
        let res = Cpu::sign_extend_word(345);
        assert_eq!(345, res);
    }

    #[test]
    fn sign_extend_word_negative() {
        let res = Cpu::sign_extend_word(0xFEA7); // -345
        assert_eq!(0xFFFFFEA7, res);
    }

    #[test]
    fn sign_extend_word_negative2() {
        let res = Cpu::sign_extend_word(0xffff); // -1
        assert_eq!(0xFFFFFFFF, res);
    }

    #[test]
    fn get_address_with_byte_displacement_sign_extended() {
        let res = Cpu::get_address_with_byte_displacement_sign_extended(0x00100000, 0x7f); // i8::MAX
        assert_eq!(0x0010007f, res);
    }

    #[test]
    fn get_address_with_byte_displacement_sign_extended_negative() {
        let res = Cpu::get_address_with_byte_displacement_sign_extended(0x00100000, 0x80); // i8::MIN
        assert_eq!(0x000fff80, res);
    }

    #[test]
    fn get_address_with_byte_displacement_sign_extended_overflow() {
        let res = Cpu::get_address_with_byte_displacement_sign_extended(0xffffffff, 0x7f); // i8::MAX
        assert_eq!(0x0000007e, res);
    }

    #[test]
    fn get_address_with_byte_displacement_sign_extended_overflow_negative() {
        let res = Cpu::get_address_with_byte_displacement_sign_extended(0x00000000, 0x80); // i8::MIN
        assert_eq!(0xffffff80, res);
    }

    #[test]
    fn get_address_with_word_displacement_sign_extended() {
        let res = Cpu::get_address_with_word_displacement_sign_extended(0x00100000, 0x7fff); // i16::MAX
        assert_eq!(0x00107fff, res);
    }

    #[test]
    fn get_address_with_word_displacement_sign_extended_negative() {
        let res = Cpu::get_address_with_word_displacement_sign_extended(0x00100000, 0x8000); // i16::MIN
        assert_eq!(0x000f8000, res);
    }

    #[test]
    fn get_address_with_word_displacement_sign_extended_overflow() {
        let res = Cpu::get_address_with_word_displacement_sign_extended(0xffffffff, 0x7fff); // i16::MAX
        assert_eq!(0x00007ffe, res);
    }

    #[test]
    fn get_address_with_word_displacement_sign_extended_overflow_neg() {
        let res = Cpu::get_address_with_word_displacement_sign_extended(0x00000000, 0x8000); // i16::MIN
        assert_eq!(0xffff8000, res);
    }

    #[test]
    fn get_byte_from_long_x78() {
        let res = Cpu::get_byte_from_long(0x12345678);
        assert_eq!(0x78, res);
    }

    #[test]
    fn get_byte_from_long_xff() {
        let res = Cpu::get_byte_from_long(0xffffffff);
        assert_eq!(0xff, res);
    }

    #[test]
    fn get_byte_from_long_x00() {
        let res = Cpu::get_byte_from_long(0x88888800);
        assert_eq!(0x00, res);
    }

    #[test]
    fn get_byte_from_word_x78() {
        let res = Cpu::get_byte_from_word(0x5678);
        assert_eq!(0x78, res);
    }

    #[test]
    fn get_byte_from_word_xff() {
        let res = Cpu::get_byte_from_word(0xffff);
        assert_eq!(0xff, res);
    }

    #[test]
    fn get_byte_from_word_x00() {
        let res = Cpu::get_byte_from_word(0x8800);
        assert_eq!(0x00, res);
    }

    #[test]
    fn get_word_from_long_x5678() {
        let res = Cpu::get_word_from_long(0x12345678);
        assert_eq!(0x5678, res);
    }

    #[test]
    fn get_word_from_long_xffff() {
        let res = Cpu::get_word_from_long(0xffffffff);
        assert_eq!(0xffff, res);
    }

    #[test]
    fn get_word_from_long_x0000() {
        let res = Cpu::get_word_from_long(0x88880000);
        assert_eq!(0x0000, res);
    }

    #[test]
    fn get_signed_byte_from_byte_x78() {
        let res = Cpu::get_signed_byte_from_byte(0x78);
        assert_eq!(0x78, res);
    }

    #[test]
    fn get_signed_byte_from_byte_xff() {
        let res = Cpu::get_signed_byte_from_byte(0xff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_byte_from_byte_x80() {
        let res = Cpu::get_signed_byte_from_byte(0x80);
        assert_eq!(-128, res);
    }

    #[test]
    fn get_signed_byte_from_byte_x00() {
        let res = Cpu::get_signed_byte_from_byte(0x00);
        assert_eq!(0x00000000, res);
    }

    #[test]
    fn get_signed_word_from_word_x5678() {
        let res = Cpu::get_signed_word_from_word(0x5678);
        assert_eq!(0x5678, res);
    }

    #[test]
    fn get_signed_word_from_word_xffff() {
        let res = Cpu::get_signed_word_from_word(0xffff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_word_from_word_x8000() {
        let res = Cpu::get_signed_word_from_word(0x8000);
        assert_eq!(-32768, res);
    }

    #[test]
    fn get_signed_word_from_word_x0000() {
        let res = Cpu::get_signed_word_from_word(0x0000);
        assert_eq!(0x0000, res);
    }

    #[test]
    fn get_signed_long_from_long_x12345678() {
        let res = Cpu::get_signed_long_from_long(0x12345678);
        assert_eq!(0x12345678, res);
    }

    #[test]
    fn get_signed_long_from_long_xffffffff() {
        let res = Cpu::get_signed_long_from_long(0xffffffff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_long_from_long_x80000000() {
        let res = Cpu::get_signed_long_from_long(0x80000000);
        assert_eq!(-2147483648, res);
    }

    #[test]
    fn get_signed_long_from_long_x00000000() {
        let res = Cpu::get_signed_long_from_long(0x00000000);
        assert_eq!(0x00000000, res);
    }

    #[test]
    fn add_bytes_unsigned_overflow_set_carry_and_extend() {
        let result = Cpu::add_bytes(0xf0, 0x20);
        assert_eq!(
            ResultWithStatusRegister {
                result: 0x10,
                status_register_result: StatusRegisterResult {
                    status_register: STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
                    status_register_mask: STATUS_REGISTER_MASK_CARRY
                        | STATUS_REGISTER_MASK_EXTEND
                        | STATUS_REGISTER_MASK_OVERFLOW
                        | STATUS_REGISTER_MASK_ZERO
                        | STATUS_REGISTER_MASK_NEGATIVE
                }
            },
            result
        );
    }

    #[test]
    fn add_bytes_signed_overflow_set_overflow() {
        let result = Cpu::add_bytes(0x70, 0x10);
        assert_eq!(
            ResultWithStatusRegister {
                result: 0x80,
                status_register_result: StatusRegisterResult {
                    status_register: STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_NEGATIVE,
                    status_register_mask: STATUS_REGISTER_MASK_CARRY
                        | STATUS_REGISTER_MASK_EXTEND
                        | STATUS_REGISTER_MASK_OVERFLOW
                        | STATUS_REGISTER_MASK_ZERO
                        | STATUS_REGISTER_MASK_NEGATIVE
                }
            },
            result
        );
    }

    #[test]
    fn add_bytes_both_overflow_set_carry_and_extend_and_overflow() {
        // for i in 0 .. 255 {
        //     Cpu::add_unsigned_bytes(i, 3);
        // }

        let result = Cpu::add_bytes(0x80, 0x80);
        assert_eq!(
            ResultWithStatusRegister {
                result: 0x00,
                status_register_result: StatusRegisterResult {
                    status_register: STATUS_REGISTER_MASK_CARRY
                        | STATUS_REGISTER_MASK_EXTEND
                        | STATUS_REGISTER_MASK_OVERFLOW
                        | STATUS_REGISTER_MASK_ZERO,
                    status_register_mask: STATUS_REGISTER_MASK_CARRY
                        | STATUS_REGISTER_MASK_EXTEND
                        | STATUS_REGISTER_MASK_OVERFLOW
                        | STATUS_REGISTER_MASK_ZERO
                        | STATUS_REGISTER_MASK_NEGATIVE
                }
            },
            result
        );
    }

    #[test]
    fn evaluate_condition_cc_when_carry_cleared() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::CC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_cc_when_carry_set() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY);
        let res = sr.evaluate_condition(&ConditionalTest::CC);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_cs_when_carry_cleared() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::CS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_cs_when_carry_set() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY);
        let res = sr.evaluate_condition(&ConditionalTest::CS);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_eq_when_zero_cleared() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::EQ);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_eq_when_zero_set() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::EQ);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_f_false() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::F);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::F);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ge_false() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(false, res);

        let extra_flags =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_ZERO;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ge_true() {
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(true, res);

        let extra_flags =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_ZERO;

        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_gt_false() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr =
            StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr =
            StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_ZERO | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_gt_true() {
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(true, res);
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::GT);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_hi_false() {
        let sr = StatusRegister::from_empty();
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY);
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_hi_true() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::HI);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_le_false() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW,
        );
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_le_true() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_ls_false() {
        let sr = StatusRegister::from_word(0x0000);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ls_true() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_CARRY | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LS);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_lt_false() {
        let sr = StatusRegister::from_word(0x0000);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW,
        );
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(false, res);

        let extra_flags =
            STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(false, res);
        let sr = StatusRegister::from_word(
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags,
        );
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_lt_true() {
        let sr = StatusRegister::from_empty();
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(true, res);

        let extra_flags =
            STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(true, res);
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::LT);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_mi_false() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::MI);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::MI);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_mi_true() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::MI);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::MI);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_ne_false() {
        //  Z => EQ=TRUE => NE=FALSE
        // !Z => NE=TRUE => EQ=TRUE
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO);
        let res = sr.evaluate_condition(&ConditionalTest::NE);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_ZERO | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::NE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ne_true() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::NE);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::NE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_pl_false() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE);
        let res = sr.evaluate_condition(&ConditionalTest::PL);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_NEGATIVE | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::PL);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_pl_true() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::PL);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::PL);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_t_true() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::T);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::T);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_vc_false() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::VC);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::VC);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_vc_true() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::VC);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::VC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_vs_false() {
        let sr = StatusRegister::from_empty();
        let res = sr.evaluate_condition(&ConditionalTest::VS);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(0x0000 | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::VS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_vs_true() {
        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW);
        let res = sr.evaluate_condition(&ConditionalTest::VS);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        let sr = StatusRegister::from_word(STATUS_REGISTER_MASK_OVERFLOW | extra_flags);
        let res = sr.evaluate_condition(&ConditionalTest::VS);
        assert_eq!(true, res);
    }

    #[test]
    fn declare_word_when_get_disassembly_for_unknown_instruction_word() {
        // arrange
        let code = [0x49, 0x54].to_vec(); // DC.W $4954
        let mut mm = crate::tests::instr_test_setup(code, None);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("DC.W"),
                String::from("#$4954"),
                vec![0x4954]
            ),
            debug_result
        );
    }
}
