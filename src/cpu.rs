use crate::register::*;
use crate::{cpu::instruction::*, mem::Mem};
use byteorder::{BigEndian, ReadBytesExt};
use core::panic;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};

use self::ea::EffectiveAddressDebug;

pub mod ea;
pub mod instruction;

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

pub struct Cpu {
    pub register: Register,
    pub memory: Mem,
    instructions: Vec<Instruction>,
}

impl Cpu {
    pub fn new(mem: Mem) -> Cpu {
        let reg_ssp = mem.get_long(0x0);
        let pc_address = mem.get_long(0x4);
        let reg_pc = ProgramCounter::from_address(pc_address);
        let instructions = vec![
            Instruction::new_with_exclude(
                String::from("ADDX"),
                0xf130,
                0xd100,
                0x00c0,
                0x00c0,
                instruction::addx::step,
                instruction::addx::get_disassembly,
            ),
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                instruction::add::step,
                instruction::add::get_disassembly,
            ),
            Instruction::new(
                String::from("ADDI"),
                0xff00,
                0x0600,
                instruction::addi::step,
                instruction::addi::get_disassembly,
            ),
            Instruction::new(
                String::from("AND"),
                0xf000,
                0xc000,
                instruction::and::step,
                instruction::and::get_disassembly,
            ),
            Instruction::new(
                String::from("BRA"),
                0xff00,
                0x6000,
                instruction::bra::step,
                instruction::bra::get_disassembly,
            ),
            Instruction::new(
                String::from("BSR"),
                0xff00,
                0x6100,
                instruction::bsr::step,
                instruction::bsr::get_disassembly,
            ),
            Instruction::new(
                String::from("BTST"), // Bit Number Dynamic
                0xf1c0,
                0x0100,
                instruction::btst::step,
                instruction::btst::get_disassembly,
            ),
            Instruction::new(
                String::from("BTST"), // Bit Number Static
                0xffc0,
                0x0800,
                instruction::btst::step,
                instruction::btst::get_disassembly,
            ),
            Instruction::new(
                String::from("BCLR"), // Bit Number Dynamic
                0xf1c0,
                0x0180,
                instruction::bclr::step,
                instruction::bclr::get_disassembly,
            ),
            Instruction::new(
                String::from("BCLR"), // Bit Number Static
                0xffc0,
                0x0880,
                instruction::bclr::step,
                instruction::bclr::get_disassembly,
            ),
            Instruction::new(
                String::from("DBcc"), // DBcc need to be before ADDQ
                0xf0f8,
                0x50c8,
                instruction::dbcc::step,
                instruction::dbcc::get_disassembly,
            ),
            Instruction::new(
                String::from("ADDQ"),
                0xf100,
                0x5000,
                instruction::addq::step,
                instruction::addq::get_disassembly,
            ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                instruction::bcc::step,
                instruction::bcc::get_disassembly,
            ),
            Instruction::new(
                String::from("CLR"),
                0xff00,
                0x4200,
                instruction::clr::step,
                instruction::clr::get_disassembly,
            ),
            Instruction::new(
                String::from("CMP"),
                0xb000,
                0xb000,
                instruction::cmp::step,
                instruction::cmp::get_disassembly,
            ),
            Instruction::new(
                String::from("CMPI"),
                0xff00,
                0x0c00,
                instruction::cmpi::step,
                instruction::cmpi::get_disassembly,
            ),
            Instruction::new(
                String::from("JMP"),
                0xffc0,
                0x4ec0,
                instruction::jmp::step,
                instruction::jmp::get_disassembly,
            ),
            Instruction::new(
                String::from("JSR"),
                0xffc0,
                0x4e80,
                instruction::jsr::step,
                instruction::jsr::get_disassembly,
            ),
            Instruction::new(
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                instruction::lea::step,
                instruction::lea::get_disassembly,
            ),
            Instruction::new(
                String::from("LINK"), // word
                0xfff8,
                0x4e50,
                instruction::link::step,
                instruction::link::get_disassembly,
            ),
            Instruction::new(
                String::from("LINK"), // long
                0xfff8,
                0x4808,
                instruction::link::step_long,
                instruction::link::get_disassembly_long,
            ),
            Instruction::new(
                String::from("MOVE"),
                0xc000,
                0x0000,
                instruction::mov::step,
                instruction::mov::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEC"),
                0xfffe,
                0x4e7a,
                instruction::movec::step,
                instruction::movec::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEM"),
                0xfb80,
                0x4880,
                instruction::movem::step,
                instruction::movem::get_disassembly,
            ),
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                instruction::moveq::step,
                instruction::moveq::get_disassembly,
            ),
            Instruction::new(
                String::from("NOP"),
                0xffff,
                0x4e71,
                instruction::nop::step,
                instruction::nop::get_disassembly,
            ),
            Instruction::new(
                String::from("NOT"),
                0xff00,
                0x4600,
                instruction::not::step,
                instruction::not::get_disassembly,
            ),
            Instruction::new(
                String::from("RTS"),
                0xffff,
                0x4e75,
                instruction::rts::step,
                instruction::rts::get_disassembly,
            ),
            Instruction::new(
                String::from("SUB"),
                0xf000,
                0x9000,
                instruction::sub::step,
                instruction::sub::get_disassembly,
            ),
            Instruction::new(
                String::from("SUBX"),
                0xf130,
                0x9100,
                instruction::subx::step,
                instruction::subx::get_disassembly,
            ),
            Instruction::new(
                String::from("SUBQ"),
                0xf100,
                0x5100,
                instruction::subq::step,
                instruction::subq::get_disassembly,
            ),
            Instruction::new(
                String::from("TST"),
                0xff00,
                0x4a00,
                instruction::tst::step,
                instruction::tst::get_disassembly,
            ),
        ];
        let mut register = Register::new();
        register.set_ssp_reg(reg_ssp);
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            register: register,
            memory: mem,
            instructions: instructions,
        };
        cpu
    }

    pub fn sign_extend_byte(value: u8) -> u32 {
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
        let displacement = Cpu::sign_extend_byte(displacement);
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
        let ea_mode = (word >> 8) & 0x000f;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract ConditionalTest"),
        };
        ea_mode
    }

    pub fn extract_scale_factor_from_bit_pos(word: u16, bit_pos: u8) -> ScaleFactor {
        let scale_factor = (word >> bit_pos) & 0b0011;
        let scale_factor = match ScaleFactor::from_u16(scale_factor) {
            Some(r) => r,
            None => panic!("Unable to extract ScaleFactor"),
        };
        scale_factor
    }

    fn extract_op_mode_from_bit_pos_6(word: u16) -> usize {
        let op_mode = (word >> 6) & 0x0007;
        let op_mode = match FromPrimitive::from_u16(op_mode) {
            Some(r) => r,
            None => panic!("Unable to extract OpMode"),
        };
        op_mode
    }

    fn extract_op_mode_from_bit_pos_6_new<T>(word: u16) -> Result<T, InstructionError>
    where
        T: TryFrom<u16>,
    {
        let op_mode = (word >> 6) & 0x0007;
        let op_mode = match op_mode.try_into() {
            Ok(a) => Ok(a),
            Err(e) => Err(InstructionError {
                details: format!("Failed to extract op_mode from bit pos 6"),
            }),
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

    pub fn extract_size000110_from_bit_pos_6(word: u16) -> Result<OperationSize, InstructionError> {
        let size = (word >> 6) & 0x0003;
        match size {
            0b00 => Ok(OperationSize::Byte),
            0b01 => Ok(OperationSize::Word),
            0b10 => Ok(OperationSize::Long),
            _ => Err(InstructionError {
                details: format!(
                    "Failed to extract operation size 000110 from bit pos 6, got size: {} from word: ${:04X}",
                    size, word
                ),
            }),
        }
    }

    pub fn extract_size011110_from_bit_pos(
        word: u16,
        bit_pos: u8,
    ) -> Result<OperationSize, InstructionError> {
        let size = (word >> bit_pos) & 0x0003;
        match size {
            0b01 => Ok(OperationSize::Byte),
            0b11 => Ok(OperationSize::Word),
            0b10 => Ok(OperationSize::Long),
            _ => Err(InstructionError {
                details: format!(
                    "Failed to extract operation size 011110 from bit pos {}, got size: {} from word: ${:04X}",
                    bit_pos, size, word
                ),
            }),
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
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::ARegDirect {
                ea_register: register,
            } => {
                // An
                let format = format!("A{}", register);
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::ARegIndirect {
                ea_register: register,
                ea_address: address,
            } => {
                // (An)
                let format = format!("(A{})", register);
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                operation_size,
                ea_register,
            } => {
                // (An)+
                let format = format!("(A{})+", ea_register);
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                operation_size,
                ea_register,
            } => {
                // (-An)
                let format = format!("-(A{})", ea_register);
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement {
                ea_register: register,
                ea_address: address,
                ea_displacement: displacement,
            } => {
                // (d16,An)
                let format = format!(
                    "(${:04X},A{}) [{}]",
                    displacement,
                    register,
                    Cpu::get_signed_long_from_long(Cpu::sign_extend_word(displacement))
                );
                EffectiveAddressDebug { format: format }
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
                    "(${:02X},A{},{}{}.{}{}) [{}]",
                    displacement,
                    ea_register,
                    register_type,
                    register,
                    index_size,
                    scale_factor,
                    displacement
                );
                EffectiveAddressDebug { format: format }
            }
            EffectiveAddressingMode::PcIndirectWithDisplacement {
                ea_address,
                displacement,
            } => {
                // (d16,PC)
                let format = format!("(${:04X},PC) [${:08X}]", displacement, ea_address);

                EffectiveAddressDebug { format }
            }
            EffectiveAddressingMode::AbsoluteShortAddressing {
                ea_address,
                displacement,
            } => {
                // (xxx).W
                let format = match displacement > 0x8000 {
                    false => format!("(${:04X}).W", displacement),
                    true => format!("(${:04X}).W [${:08X}]", displacement, ea_address),
                };
                EffectiveAddressDebug { format: format }
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
                    "(${:02X},PC,{}{}.{}{}) [${:08X}]",
                    displacement,
                    register_type_format,
                    register,
                    index_size_format,
                    scale_factor,
                    ea_address
                );

                EffectiveAddressDebug { format: format }
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

    pub fn exception(&mut self, pc: &mut ProgramCounter, vector: u32) {
        println!("Exception occured!");
        println!("vector: {} [${:02X}]", vector, vector);
        self.register.reg_sr.set_supervisor();
        // TODO: T bit clear
        // TODO: Do we push PC or PC+2?
        self.register.stack_push_pc(&mut self.memory);
        self.register
            .stack_push_word(&mut self.memory, self.register.reg_sr.get_value());

        let vector_offset = (vector) * 4;
        println!("vector_offset: ${:08X}", vector_offset);
        let vector_address = self.memory.get_long(vector_offset);
        println!("vector_address: ${:08X}", vector_address);
        pc.set_long(vector_address);
    }

    pub fn execute_next_instruction(self: &mut Cpu) {
        let mut pc = self.register.reg_pc.clone();
        let instr_word = pc.peek_next_word(&self.memory);

        let instruction_pos = self.instructions.iter().position(|x| {
            ((instr_word & x.mask) == x.opcode) && ((instr_word & x.ex_mask) != x.ex_code)
        });
        let instruction = match instruction_pos {
            None => panic!(
                "{:#010X} Unidentified instruction {:#06X}",
                pc.get_address(),
                instr_word
            ),
            Some(instruction_pos) => &self.instructions[instruction_pos],
        };

        let step_result = (instruction.step)(&mut pc, &mut self.register, &mut self.memory);
        match step_result {
            Ok(step_result) => self.register.reg_pc = pc.get_step_next_pc(),
            Err(step_error) => match step_error {
                StepError::IllegalInstruction => {
                    self.exception(&mut pc, 4);
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

    pub fn get_next_disassembly(self: &mut Cpu) -> GetDisassemblyResult {
        let get_disassembly_result = self.get_disassembly(&mut self.register.reg_pc.clone());
        get_disassembly_result
    }

    pub fn get_disassembly(self: &mut Cpu, pc: &mut ProgramCounter) -> GetDisassemblyResult {
        let instr_word = pc.peek_next_word(&self.memory);

        let instruction_pos = self.instructions.iter().position(|x| {
            ((instr_word & x.mask) == x.opcode) && ((instr_word & x.ex_mask) != x.ex_code)
        });

        match instruction_pos {
            Some(instruction_pos) => {
                let instruction = &self.instructions[instruction_pos];

                let get_disassembly_result =
                    (instruction.get_disassembly)(pc, &mut self.register, &mut self.memory);

                match get_disassembly_result {
                    Ok(result) => result,
                    Err(error) => {
                        if pc.get_address() == pc.get_address_next() {
                            pc.skip_word();
                        }
                        GetDisassemblyResult::from_pc(
                            pc,
                            String::from("DC.W"),
                            format!(
                                "#${:04X} ; Error when getting disassembly from instruction: {}",
                                instr_word, error.details
                            ),
                        )
                    }
                }
            }
            None => {
                pc.skip_word();
                GetDisassemblyResult::from_pc(
                    pc,
                    String::from("DC.W"),
                    format!("#${:04X}", instr_word),
                )
            }
        }
    }

    pub fn print_disassembly(self: &mut Cpu, disassembly_result: &GetDisassemblyResult) {
        let instr_format = format!(
            "{} {}",
            disassembly_result.name, disassembly_result.operands_format
        );
        let instr_address = disassembly_result.address;
        let next_instr_address = disassembly_result.address_next;
        print!("${:08X} ", instr_address);
        for i in (instr_address..instr_address + 8).step_by(2) {
            if i < next_instr_address {
                let op_mem = self.memory.get_word(i);
                print!("{:04X} ", op_mem);
            } else {
                print!("     ");
            }
        }
        println!("{: <30}", instr_format);
    }

    // fn get_instruction(self: &mut Cpu, instr_addr: u32, instr_word: u16) -> &Instruction {
    //     let instruction_pos = self
    //         .instructions
    //         .iter()
    //         .position(|x| (instr_word & x.mask) == x.opcode);
    //     let instruction = match instruction_pos {
    //         None => panic!(
    //             "{:#010x} Unidentified instruction {:#06X}",
    //             instr_addr, instr_word
    //         ),
    //         Some(instruction_pos) => &self.instructions[instruction_pos],
    //     };
    //     instruction
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_byte_positive() {
        let res = Cpu::sign_extend_byte(45);
        assert_eq!(45, res);
    }

    #[test]
    fn sign_extend_byte_negative() {
        let res = Cpu::sign_extend_byte(0xd3); // -45
        assert_eq!(0xFFFFFFD3, res);
    }

    #[test]
    fn sign_extend_byte_negative2() {
        let res = Cpu::sign_extend_byte(0xff); // -1
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
        let mut cpu = crate::instr_test_setup(code, None);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("DC.W"),
                String::from("#$4954")
            ),
            debug_result
        );
    }
}
