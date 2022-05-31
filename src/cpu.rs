use crate::cpu::instruction::*;
use crate::mem::Mem;
use crate::register::*;
use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{convert::TryInto, fmt};

pub mod instruction;

pub struct EffectiveAddress {
    pub address: u32,
    pub num_extension_words: u32,
}

pub struct EffectiveAddressValue<T> {
    pub value: T,
    pub num_extension_words: u32,
}

pub struct SetEffectiveAddressValueResult {
    pub num_extension_words: u32,
    pub status_register_result: StatusRegisterResult,
}

pub struct EffectiveAddressDebug {
    // pub address: u32,
    pub format: String,
    pub num_extension_words: u32,
}

impl fmt::Display for EffectiveAddressDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format)
    }
}

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
    pub fn merge_status_register(&self, status_register: u16) -> u16 {
        (status_register & !self.status_register_mask)
            | (self.status_register & self.status_register_mask)
    }
}

pub struct Cpu {
    pub register: Register,
    pub memory: Mem,
    instructions: Vec<Instruction>,
}

impl Cpu {
    pub fn new(mut mem: Mem) -> Cpu {
        let reg_ssp = mem.get_unsigned_long(0x0);
        let reg_pc = mem.get_unsigned_long(0x4);
        let instructions = vec![
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                instruction::add::step,
                instruction::add::get_disassembly,
            ),
            Instruction::new(
                String::from("DBcc"),
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
            // Instruction::new(
            //     String::from("ADDX"),
            //     0xf130,
            //     0xd100,
            //     // InstructionFormat::Uncommon {
            //     step: instruction::addx::step,
            //     get_debug: instruction::addx::get_debug,
            //     // },
            // ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                instruction::bcc::step,
                instruction::bcc::get_disassembly,
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
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                instruction::lea::step,
                instruction::lea::get_disassembly,
            ),
            // Instruction::new(
            //     String::from("Scc"),
            //     0xf0c0,
            //     0x50c0,
            //     // InstructionFormat::Uncommon {
            //     step: instruction::todo::step,
            //     get_debug: instruction::todo::get_debug,
            //     // },
            // ),
            // Instruction::new(
            //     String::from("SUBQ"),
            //     0xf100,
            //     0x5100,
            //     // InstructionFormat::EffectiveAddress {
            //     step: instruction::subq::common_step,
            //     get_debug: instruction::subq::common_get_debug,
            //     //     areg_direct_step: instruction::subq::areg_direct_step,
            //     //     areg_direct_get_debug: instruction::subq::areg_direct_get_debug,
            //     // },
            // ),

            // Instruction::new(
            //     String::from("TRAPcc"),
            //     0xf0f8,
            //     0x50f8,
            //     // InstructionFormat::Uncommon {
            //     step: instruction::todo::step,
            //     get_debug: instruction::todo::get_debug,
            //     // },
            // ),
            Instruction::new(
                String::from("MOVE"),
                0xc000,
                0x0000,
                instruction::mov::step,
                instruction::mov::get_disassembly,
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
        ];
        let mut register = Register::new();
        register.reg_a[7] = reg_ssp;
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            register: register,
            memory: mem,
            instructions: instructions,
        };
        cpu
    }

    fn sign_extend_i8(address: i8) -> u32 {
        // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes: [u8; 4] = if address < 0 {
            [0xff, 0xff, 0xff, address_bytes[0]]
        } else {
            [0x00, 0x00, 0x00, address_bytes[0]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    fn sign_extend_i16(address: i16) -> u32 {
        // // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes: [u8; 4] = if address < 0 {
            [0xff, 0xff, address_bytes[0], address_bytes[1]]
        } else {
            [0x00, 0x00, address_bytes[0], address_bytes[1]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    fn get_address_with_i8_displacement(address: u32, displacement: i8) -> u32 {
        let displacement = Cpu::sign_extend_i8(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    fn get_address_with_i16_displacement(address: u32, displacement: i16) -> u32 {
        let displacement = Cpu::sign_extend_i16(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    fn extract_effective_addressing_mode_from_bit_pos_3(word: u16) -> EffectiveAddressingMode {
        let ea_mode = (word >> 3) & 0x0007;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract EffectiveAddressingMode"),
        };
        ea_mode
    }

    fn extract_effective_addressing_mode_from_bit_pos(
        word: u16,
        bit_pos: u8,
    ) -> EffectiveAddressingMode {
        let ea_mode = (word >> bit_pos) & 0x0007;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract EffectiveAddressingMode"),
        };
        ea_mode
    }

    fn extract_conditional_test_pos_8(word: u16) -> ConditionalTest {
        let ea_mode = (word >> 8) & 0x000f;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract ConditionalTest"),
        };
        ea_mode
    }

    fn extract_op_mode_from_bit_pos_6(word: u16) -> usize {
        let op_mode = (word >> 6) & 0x0007;
        let op_mode = match FromPrimitive::from_u16(op_mode) {
            Some(r) => r,
            None => panic!("Unable to extract OpMode"),
        };
        op_mode
    }

    fn extract_register_index_from_bit_pos(word: u16, bit_pos: u8) -> usize {
        let register = (word >> bit_pos) & 0x0007;
        let register = register.try_into().unwrap();
        register
    }

    fn extract_register_index_from_bit_pos_0(word: u16) -> usize {
        let register = word & 0x0007;
        let register = register.try_into().unwrap();
        register
    }

    pub fn extract_size000110_from_bit_pos_6(word: u16) -> OperationSize {
        let size = (word >> 6) & 0x0003;
        match size {
            0b00 => OperationSize::Byte,
            0b01 => OperationSize::Word,
            0b10 => OperationSize::Long,
            _ => panic!("Unknown size!"),
        }
    }

    pub fn extract_size011110_from_bit_pos(word: u16, bit_pos: u8) -> OperationSize {
        let size = (word >> bit_pos) & 0x0003;
        match size {
            0b01 => OperationSize::Byte,
            0b11 => OperationSize::Word,
            0b10 => OperationSize::Long,
            _ => panic!("Unknown size!"),
        }
    }

    pub fn get_unsigned_byte_from_unsigned_long(long: u32) -> u8 {
        (long & 0x000000ff) as u8
    }

    pub fn get_unsigned_word_from_unsigned_long(long: u32) -> u16 {
        (long & 0x0000ffff) as u16
    }

    pub fn get_signed_byte_from_long(long: u32) -> i8 {
        (long & 0x000000ff) as i8
    }

    pub fn get_signed_word_from_unsigned_long(long: u32) -> i16 {
        (long & 0x0000ffff) as i16
    }

    pub fn get_signed_long_from_unsigned_long(long: u32) -> i32 {
        long as i32
    }

    pub fn set_unsigned_byte_in_unsigned_long(value: u8, long: u32) -> u32 {
        (long & 0xffffff00) | (value as u32)
    }

    pub fn set_unsigned_word_in_unsigned_long(value: u16, long: u32) -> u32 {
        (long & 0xffff0000) | (value as u32)
    }

    pub fn set_signed_byte_in_unsigned_long(value: i8, long: u32) -> u32 {
        (long & 0xffffff00) | (value as u32)
    }

    pub fn set_signed_word_in_unsigned_long(value: i16, long: u32) -> u32 {
        (long & 0xffff0000) | (value as u32)
    }

    pub fn set_signed_long_in_unsigned_long(value: i32, long: u32) -> u32 {
        long as u32
    }

    pub fn add_unsigned_bytes(value_1: u8, value_2: u8) -> ResultWithStatusRegister<u8> {
        let (result, carry) = value_2.overflowing_add(value_1);
        let value_1_signed = value_1 as i8;
        let value_2_signed = value_2 as i8;

        let (result_signed, overflow) = value_2_signed.overflowing_add(value_1_signed);

        // let carrys = match carry {
        //     false => String::from(""),
        //     true => String::from("carry"),
        // };
        // let overflows = match overflow {
        //     false => String::from(""),
        //     true => String::from("overflow"),
        // };

        // println!("value_1:        {:4}, value_2:        {:4}, result:        {:4} {:5}", value_1, value_2, result, carrys);
        // println!("value_1_signed: {:4}, value_2_signed: {:4}, result_signed: {:4} {:5}", value_1_signed, value_2_signed, result_signed, overflows);

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

    pub fn add_unsigned_words(value_1: u16, value_2: u16) -> ResultWithStatusRegister<u16> {
        let (result, carry) = value_2.overflowing_add(value_1);
        let value_1_signed = value_1 as i16;
        let value_2_signed = value_2 as i16;

        let (result_signed, overflow) = value_2_signed.overflowing_add(value_1_signed);

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

    pub fn add_unsigned_longs(value_1: u32, value_2: u32) -> ResultWithStatusRegister<u32> {
        let (result, carry) = value_2.overflowing_add(value_1);
        let value_1_signed = value_1 as i32;
        let value_2_signed = value_2 as i32;

        let (result_signed, overflow) = value_2_signed.overflowing_add(value_1_signed);

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

    pub fn get_ea_format(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        operation_size: Option<OperationSize>,
        reg: &Register,
        mem: &Mem,
    ) -> EffectiveAddressDebug {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let format = format!("D{}", ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let format = format!("A{}", ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirect => {
                let format = format!("(A{})", ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                let format = format!("(A{})+", ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                let format = format!("-(A{})", ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement => {
                let extension_word = mem.get_signed_word(extension_address);
                let format = format!("(${:04X},A{})", extension_word, ea_register);
                EffectiveAddressDebug {
                    format: format,
                    num_extension_words: 1,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithIndex => {
                panic!("get_ea_format() UNKNOWN_EA {:?} {}", ea_mode, ea_register);
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore => match ea_register {
                0b000 => {
                    // Absolute Short Addressing Mode
                    // (xxx).W
                    let extension_word = mem.get_signed_word(extension_address);
                    let format = format!("(${:04X}).W", extension_word);
                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 1,
                    }
                }
                0b001 => {
                    // Absolute Long Addressing Mode
                    // (xxx).L
                    // let first_extension_word = mem.get_unsigned_word(instr_address + 2);
                    // let second_extension_word = mem.get_unsigned_word(instr_address + 4);
                    // let address =
                    //     u32::from(first_extension_word) << 16 | u32::from(second_extension_word);
                    let address = mem.get_unsigned_long(extension_address);
                    let format = format!("(${:08X}).L", address);
                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 2,
                    }
                }
                0b010 => {
                    // Program Counter Indirect with Displacement Mode
                    // (d16,PC)
                    let extension_word = mem.get_signed_word(extension_address);
                    let ea = Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, extension_word);
                    let format = format!("({:#06x},PC)", extension_word);

                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 1,
                    }
                }
                0b011 => {
                    // Program Counter Indirect with Index (8-Bit Displacement) Mode
                    // Program Counter Indirect with Index (Base Displacement) Mode
                    // Program Counter Memory Indirect Postindexed Mode
                    // Program Counter Memory Indirect Preindexed Mode
                    // (d8,PC,Xn)
                    // (bd,PC,Xn,SIZE*SCALE)
                    // ([bd,PC],Xn,SIZE*SCALE,od)
                    // ([bd,PC],Xn,SIZE*SCALE,od)
                    panic!(
                        "get_ea_format() Unhandled PcIndirectAndLotsMore 0b011 {:?} {}",
                        ea_mode, ea_register
                    );
                }
                0b100 => {
                    // Immediate data
                    // #<xxx>
                    match operation_size {
                        None => panic!("Must have operation_size for Immediate data!"),
                        Some(operation_size) => match operation_size {
                            OperationSize::Byte => {
                                let extension_word = mem.get_signed_byte(extension_address + 1);
                                EffectiveAddressDebug {
                                    format: format!("#{}", extension_word),
                                    num_extension_words: 1,
                                }
                            }
                            OperationSize::Word => EffectiveAddressDebug {
                                format: String::from("todo word"),
                                num_extension_words: 1,
                            },
                            OperationSize::Long => EffectiveAddressDebug {
                                format: String::from("todo long"),
                                num_extension_words: 2,
                            },
                        },
                    }
                }
                _ => {
                    panic!(
                        "get_ea_format() UNKNOWN PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
            },
        }
    }

    pub fn get_ea(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        operation_size: Option<OperationSize>,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddress {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                panic!("Cannot get Effective Address for 'Data Register Direct' EA mode");
            }
            EffectiveAddressingMode::ARegDirect => {
                panic!("Cannot get Effective Address for 'Address Register Direct' EA mode");
            }
            EffectiveAddressingMode::ARegIndirect => {
                let address = reg.reg_a[ea_register];
                EffectiveAddress {
                    address: address,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                let address = reg.reg_a[ea_register];
                let size_in_bytes = match operation_size {
                    None => panic!("Must have operation_size for ARegIndirectWithPostIncrement!"),
                    Some(operation_size) => operation_size.size_in_bytes(),
                };
                // println!("ARegIndirectWithPostIncrement: A{}", ea_register);
                reg.reg_a[ea_register] += size_in_bytes;
                EffectiveAddress {
                    address: address,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                let size_in_bytes = match operation_size {
                    None => panic!("Must have operation_size for ARegIndirectWithPreDecrement!"),
                    Some(operation_size) => operation_size.size_in_bytes(),
                };

                // println!("ARegIndirectWithPreDecrement: A{}", ea_register);
                reg.reg_a[ea_register] -= size_in_bytes;
                let address = reg.reg_a[ea_register];
                EffectiveAddress {
                    address: address,
                    num_extension_words: 0,
                }
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex => {
                panic!(
                    "get_ea_value_unsigned_long() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore => match ea_register {
                0b000 => {
                    // Absolute Short Addressing Mode
                    // (xxx).W
                    let extension_word = mem.get_signed_word(extension_address);
                    let address = Cpu::sign_extend_i16(extension_word);
                    // let ea_format = format!("({:#06x}).W", extension_word);
                    EffectiveAddress {
                        address: address,
                        num_extension_words: 1,
                    }
                }
                0b001 => {
                    // Absolute Long Addressing Mode
                    // (xxx).L
                    let address = mem.get_unsigned_long(extension_address);
                    EffectiveAddress {
                        address: address,
                        num_extension_words: 2,
                    }
                }
                0b010 => {
                    // Program Counter Indirect with Displacement Mode
                    // (d16,PC)
                    let extension_word = mem.get_signed_word(extension_address);
                    let address =
                        Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, extension_word);
                    EffectiveAddress {
                        address: address,
                        num_extension_words: 1,
                    }
                }
                0b011 => {
                    // Program Counter Indirect with Index (8-Bit Displacement) Mode
                    // Program Counter Indirect with Index (Base Displacement) Mode
                    // Program Counter Memory Indirect Postindexed Mode
                    // Program Counter Memory Indirect Preindexed Mode
                    // (d8,PC,Xn)
                    // (bd,PC,Xn,SIZE*SCALE)
                    // ([bd,PC],Xn,SIZE*SCALE,od)
                    // ([bd,PC],Xn,SIZE*SCALE,od)
                    panic!(
                        "get_ea() Unhandled PcIndirectAndLotsMore 0b011 {:?} {}",
                        ea_mode, ea_register
                    );
                }
                0b100 => {
                    // Immediate data
                    // #<xxx>
                    panic!("Cannot get Effective Address for 'Data Register Direct' EA mode");
                }
                _ => {
                    panic!(
                        "get_ea() UNKNOWN PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
            },
        }
    }

    pub fn get_ea_value_unsigned_byte(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<u8> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = Cpu::get_unsigned_byte_from_unsigned_long(reg.reg_d[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = Cpu::get_unsigned_byte_from_unsigned_long(reg.reg_a[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_unsigned_byte(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Byte),
                    reg,
                    mem,
                );
                let value = mem.get_unsigned_byte(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn get_ea_value_signed_byte(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<i8> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = Cpu::get_signed_byte_from_long(reg.reg_d[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = Cpu::get_signed_byte_from_long(reg.reg_a[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_signed_byte(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Byte),
                    reg,
                    mem,
                );
                let value = mem.get_signed_byte(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn get_ea_value_unsigned_word(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<u16> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = Cpu::get_unsigned_word_from_unsigned_long(reg.reg_d[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = Cpu::get_unsigned_word_from_unsigned_long(reg.reg_a[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_unsigned_word(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Word),
                    reg,
                    mem,
                );
                let value = mem.get_unsigned_word(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn get_ea_value_signed_word(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<i16> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = Cpu::get_signed_word_from_unsigned_long(reg.reg_d[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = Cpu::get_signed_word_from_unsigned_long(reg.reg_a[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_signed_word(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Word),
                    reg,
                    mem,
                );
                let value = mem.get_signed_word(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn get_ea_value_unsigned_long(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<u32> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = reg.reg_d[ea_register];
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = reg.reg_a[ea_register];
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_unsigned_long(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Long),
                    reg,
                    mem,
                );
                let value = mem.get_unsigned_long(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn get_ea_value_signed_long(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<i32> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let value = Cpu::get_signed_long_from_unsigned_long(reg.reg_d[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let value = Cpu::get_signed_long_from_unsigned_long(reg.reg_a[ea_register]);
                EffectiveAddressValue {
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                let extension_word = mem.get_signed_long(extension_address + 1);
                EffectiveAddressValue {
                    value: extension_word,
                    num_extension_words: 1,
                }
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Long),
                    reg,
                    mem,
                );
                let value = mem.get_signed_long(ea.address);
                EffectiveAddressValue {
                    num_extension_words: ea.num_extension_words,
                    value: value,
                }
            }
        }
    }

    pub fn set_ea_value_unsigned_byte(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        value: u8,
        reg: &mut Register,
        mem: &mut Mem,
    ) -> SetEffectiveAddressValueResult {
        let num_extension_words = match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                reg.reg_d[ea_register] =
                    Cpu::set_unsigned_byte_in_unsigned_long(value, reg.reg_d[ea_register]);
                0
            }
            EffectiveAddressingMode::ARegDirect => {
                reg.reg_a[ea_register] =
                    Cpu::set_unsigned_byte_in_unsigned_long(value, reg.reg_a[ea_register]);
                0
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                panic!("set_ea_value_unsigned_byte invalid EffectiveAddressingMode::PcIndirectAndLotsMore Immediate data");
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Byte),
                    reg,
                    mem,
                );
                mem.set_unsigned_byte(ea.address, value);
                ea.num_extension_words
            }
        };

        let value_signed = value as i8;

        let mut status_register = 0x0000;

        match value_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        SetEffectiveAddressValueResult {
            num_extension_words,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn set_ea_value_unsigned_word(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        extension_address: u32,
        value: u16,
        reg: &mut Register,
        mem: &mut Mem,
    ) -> SetEffectiveAddressValueResult {
        let num_extension_words = match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                reg.reg_d[ea_register] =
                    Cpu::set_unsigned_word_in_unsigned_long(value, reg.reg_d[ea_register]);
                0
            }
            EffectiveAddressingMode::ARegDirect => {
                reg.reg_a[ea_register] =
                    Cpu::set_unsigned_word_in_unsigned_long(value, reg.reg_a[ea_register]);
                0
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore if ea_register == 0b100 => {
                // Immediate data
                // #<xxx>
                panic!("set_ea_value_unsigned_word invalid EffectiveAddressingMode::PcIndirectAndLotsMore Immediate data");
            }
            _ => {
                let ea = Cpu::get_ea(
                    ea_mode,
                    ea_register,
                    extension_address,
                    Some(OperationSize::Word),
                    reg,
                    mem,
                );
                mem.set_unsigned_word(ea.address, value);
                ea.num_extension_words
            }
        };

        let value_signed = value as i16;

        let mut status_register = 0x0000;

        match value_signed {
            0 => status_register |= STATUS_REGISTER_MASK_ZERO,
            i16::MIN..=-1 => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }

        SetEffectiveAddressValueResult {
            num_extension_words,
            status_register_result: StatusRegisterResult {
                status_register,
                status_register_mask: STATUS_REGISTER_MASK_CARRY
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE,
            },
        }
    }

    pub fn print_registers(self: &mut Cpu) {
        for n in 0..8 {
            print!(" D{} {:#010X}", n, self.register.reg_d[n]);
        }
        println!();
        for n in 0..8 {
            print!(" A{} {:#010X}", n, self.register.reg_a[n]);
        }
        println!();
        print!(" SR {:#06X} ", self.register.reg_sr);
        if (self.register.reg_sr & STATUS_REGISTER_MASK_EXTEND) == STATUS_REGISTER_MASK_EXTEND {
            print!("X");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_NEGATIVE) == STATUS_REGISTER_MASK_NEGATIVE {
            print!("N");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_ZERO) == STATUS_REGISTER_MASK_ZERO {
            print!("Z");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_OVERFLOW) == STATUS_REGISTER_MASK_OVERFLOW {
            print!("V");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_CARRY) == STATUS_REGISTER_MASK_CARRY {
            print!("C");
        } else {
            print!("-");
        }
        println!();
        print!(" PC {:#010X}", self.register.reg_pc);
        println!();
    }

    fn evaluate_condition(reg: &mut Register, conditional_test: &ConditionalTest) -> bool {
        match conditional_test {
            ConditionalTest::T => true,

            ConditionalTest::CC => reg.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000,
            ConditionalTest::CS => reg.reg_sr & STATUS_REGISTER_MASK_CARRY != 0x0000,
            ConditionalTest::EQ => reg.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000,
            ConditionalTest::F => false,
            ConditionalTest::GE => {
                let ge_mask = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
                let sr = reg.reg_sr & ge_mask;
                sr == ge_mask || sr == 0x0000
                // (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::GT => {
                let gt_mask = STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_OVERFLOW
                    | STATUS_REGISTER_MASK_ZERO;
                let sr = reg.reg_sr & gt_mask;
                sr == STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW || sr == 0x0000
                // (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000
                //     && reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000)
            }
            ConditionalTest::HI => {
                reg.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000
                    && reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000
            }
            ConditionalTest::LE => {
                let le_mask = STATUS_REGISTER_MASK_ZERO
                    | STATUS_REGISTER_MASK_NEGATIVE
                    | STATUS_REGISTER_MASK_OVERFLOW;
                let sr = reg.reg_sr & le_mask;
                sr == STATUS_REGISTER_MASK_ZERO
                    || sr == STATUS_REGISTER_MASK_NEGATIVE
                    || sr == STATUS_REGISTER_MASK_OVERFLOW
                // (reg.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                //     || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                //         && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::LS => {
                (reg.reg_sr & STATUS_REGISTER_MASK_CARRY != 0x0000)
                    || (reg.reg_sr & STATUS_REGISTER_MASK_ZERO != 0x0000)
            }
            ConditionalTest::LT => {
                (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000
                    && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000)
                    || (reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000
                        && reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000)
            }
            ConditionalTest::MI => reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE != 0x0000,
            ConditionalTest::NE => reg.reg_sr & STATUS_REGISTER_MASK_ZERO == 0x0000,
            ConditionalTest::PL => reg.reg_sr & STATUS_REGISTER_MASK_NEGATIVE == 0x0000,
            ConditionalTest::VC => reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW == 0x0000,
            ConditionalTest::VS => reg.reg_sr & STATUS_REGISTER_MASK_OVERFLOW != 0x0000,
        }
    }

    pub fn execute_next_instruction(self: &mut Cpu) {
        let instr_addr = self.register.reg_pc;
        let instr_word = self.memory.get_unsigned_word(instr_addr);

        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode);
        let instruction = match instruction_pos {
            None => panic!(
                "{:#010x} Unidentified instruction {:#06X}",
                instr_addr, instr_word
            ),
            Some(instruction_pos) => &self.instructions[instruction_pos],
        };

        let step = instruction.step;
        let exec_result = step(instr_addr, instr_word, &mut self.register, &mut self.memory);

        if let InstructionExecutionResult::Done { pc_result } = &exec_result {
            self.register.reg_pc = match pc_result {
                PcResult::Set(new_pc) => *new_pc,
                PcResult::Increment(pc_increment) => self.register.reg_pc + pc_increment,
            }
        }
    }

    pub fn get_next_disassembly(self: &mut Cpu) -> DisassemblyResult {
        let result = self.get_disassembly(self.register.reg_pc);
        result
    }

    pub fn get_disassembly(self: &mut Cpu, instr_addr: u32) -> DisassemblyResult {
        let instr_word = self.memory.get_unsigned_word(instr_addr);

        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode);
        let instruction = match instruction_pos {
            None => panic!(
                "{:#010x} Unidentified instruction {:#06X}",
                instr_addr, instr_word
            ),
            Some(instruction_pos) => &self.instructions[instruction_pos],
        };

        let get_debug = instruction.get_debug;
        let debug_result = get_debug(instr_addr, instr_word, &mut self.register, &mut self.memory);

        debug_result
    }

    pub fn print_disassembly(self: &mut Cpu, disassembly_result: &DisassemblyResult) {
        if let DisassemblyResult::Done {
            name,
            instr_address,
            operands_format,
            next_instr_address,
        } = &disassembly_result
        {
            let instr_format = format!("{} {}", name, operands_format);
            print!("{:#010X} ", instr_address);
            for i in (*instr_address..*instr_address + 8).step_by(2) {
                if i < *next_instr_address {
                    let op_mem = self.memory.get_unsigned_word(i);
                    print!("{:04X} ", op_mem);
                } else {
                    print!("     ");
                }
            }
            println!("{: <30}", instr_format);
        }
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
    fn sign_extend_i8_positive() {
        let res = Cpu::sign_extend_i8(45);
        assert_eq!(45, res);
    }

    #[test]
    fn sign_extend_i8_negative() {
        let res = Cpu::sign_extend_i8(-45);
        assert_eq!(0xFFFFFFD3, res);
    }

    #[test]
    fn sign_extend_i8_negative2() {
        let res = Cpu::sign_extend_i8(-1);
        assert_eq!(0xFFFFFFFF, res);
    }

    #[test]
    fn sign_extend_i16_positive() {
        let res = Cpu::sign_extend_i16(345);
        assert_eq!(345, res);
    }

    #[test]
    fn sign_extend_i16_negative() {
        let res = Cpu::sign_extend_i16(-345);
        assert_eq!(0xFFFFFEA7, res);
    }

    #[test]
    fn sign_extend_i16_negative2() {
        let res = Cpu::sign_extend_i16(-1);
        assert_eq!(0xFFFFFFFF, res);
    }

    #[test]
    fn get_address_with_i8_displacement() {
        let res = Cpu::get_address_with_i8_displacement(0x00100000, i8::MAX);
        assert_eq!(0x0010007f, res);
    }

    #[test]
    fn get_address_with_i8_displacement_negative() {
        let res = Cpu::get_address_with_i8_displacement(0x00100000, i8::MIN);
        assert_eq!(0x000fff80, res);
    }

    #[test]
    fn get_address_with_i8_displacement_overflow() {
        let res = Cpu::get_address_with_i8_displacement(0xffffffff, i8::MAX);
        assert_eq!(0x0000007e, res);
    }

    #[test]
    fn get_address_with_i8_displacement_overflow_negative() {
        let res = Cpu::get_address_with_i8_displacement(0x00000000, i8::MIN);
        assert_eq!(0xffffff80, res);
    }

    #[test]
    fn get_address_with_i16_displacement() {
        let res = Cpu::get_address_with_i16_displacement(0x00100000, i16::MAX);
        assert_eq!(0x00107fff, res);
    }

    #[test]
    fn get_address_with_i16_displacement_negative() {
        let res = Cpu::get_address_with_i16_displacement(0x00100000, i16::MIN);
        assert_eq!(0x000f8000, res);
    }

    #[test]
    fn get_address_with_i16_displacement_overflow() {
        let res = Cpu::get_address_with_i16_displacement(0xffffffff, i16::MAX);
        assert_eq!(0x00007ffe, res);
    }

    #[test]
    fn get_address_with_i16_displacement_overflow_neg() {
        let res = Cpu::get_address_with_i16_displacement(0x00000000, i16::MIN);
        assert_eq!(0xffff8000, res);
    }

    #[test]
    fn get_unsigned_byte_from_unsigned_long_x78() {
        let res = Cpu::get_unsigned_byte_from_unsigned_long(0x12345678);
        assert_eq!(0x78, res);
    }

    #[test]
    fn get_unsigned_byte_from_unsigned_long_xff() {
        let res = Cpu::get_unsigned_byte_from_unsigned_long(0xffffffff);
        assert_eq!(0xff, res);
    }

    #[test]
    fn get_unsigned_byte_from_unsigned_long_x00() {
        let res = Cpu::get_unsigned_byte_from_unsigned_long(0x88888800);
        assert_eq!(0x00, res);
    }

    #[test]
    fn get_unsigned_word_from_unsigned_long_x5678() {
        let res = Cpu::get_unsigned_word_from_unsigned_long(0x12345678);
        assert_eq!(0x5678, res);
    }

    #[test]
    fn get_unsigned_word_from_unsigned_long_xffff() {
        let res = Cpu::get_unsigned_word_from_unsigned_long(0xffffffff);
        assert_eq!(0xffff, res);
    }

    #[test]
    fn get_unsigned_word_from_unsigned_long_x0000() {
        let res = Cpu::get_unsigned_word_from_unsigned_long(0x88880000);
        assert_eq!(0x0000, res);
    }

    #[test]
    fn get_signed_byte_from_long_x78() {
        let res = Cpu::get_signed_byte_from_long(0x12345678);
        assert_eq!(0x78, res);
    }

    #[test]
    fn get_signed_byte_from_long_xff() {
        let res = Cpu::get_signed_byte_from_long(0xffffffff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_byte_from_long_x80() {
        let res = Cpu::get_signed_byte_from_long(0xffffff80);
        assert_eq!(-128, res);
    }

    #[test]
    fn get_signed_byte_from_long_x00() {
        let res = Cpu::get_signed_byte_from_long(0x88880000);
        assert_eq!(0x00, res);
    }

    #[test]
    fn get_signed_word_from_unsigned_long_x5678() {
        let res = Cpu::get_signed_word_from_unsigned_long(0x12345678);
        assert_eq!(0x5678, res);
    }

    #[test]
    fn get_signed_word_from_unsigned_long_xffff() {
        let res = Cpu::get_signed_word_from_unsigned_long(0xffffffff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_word_from_unsigned_long_x8000() {
        let res = Cpu::get_signed_word_from_unsigned_long(0xffff8000);
        assert_eq!(-32768, res);
    }

    #[test]
    fn get_signed_word_from_unsigned_long_x0000() {
        let res = Cpu::get_signed_word_from_unsigned_long(0x88880000);
        assert_eq!(0x0000, res);
    }

    #[test]
    fn get_signed_long_from_unsigned_long_x12345678() {
        let res = Cpu::get_signed_long_from_unsigned_long(0x12345678);
        assert_eq!(0x12345678, res);
    }

    #[test]
    fn get_signed_long_from_unsigned_long_xffffffff() {
        let res = Cpu::get_signed_long_from_unsigned_long(0xffffffff);
        assert_eq!(-1, res);
    }

    #[test]
    fn get_signed_long_from_unsigned_long_x80000000() {
        let res = Cpu::get_signed_long_from_unsigned_long(0x80000000);
        assert_eq!(-2147483648, res);
    }

    #[test]
    fn get_signed_long_from_unsigned_long_x00000000() {
        let res = Cpu::get_signed_long_from_unsigned_long(0x00000000);
        assert_eq!(0x00000000, res);
    }

    #[test]
    fn add_unsigned_bytes_unsigned_overflow_set_carry_and_extend() {
        let result = Cpu::add_unsigned_bytes(0xf0, 0x20);
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
    fn add_unsigned_bytes_signed_overflow_set_overflow() {
        let result = Cpu::add_unsigned_bytes(0x70, 0x10);
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
    fn add_unsigned_bytes_both_overflow_set_carry_and_extend_and_overflow() {
        // for i in 0 .. 255 {
        //     Cpu::add_unsigned_bytes(i, 3);
        // }

        let result = Cpu::add_unsigned_bytes(0x80, 0x80);
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
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_cc_when_carry_set() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_cs_when_carry_cleared() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_cs_when_carry_set() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CS);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_eq_when_zero_cleared() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::EQ);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_eq_when_zero_set() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::EQ);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_f_false() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::F);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::F);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ge_false() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(false, res);

        let extra_flags =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_ZERO;

        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ge_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(true, res);
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(true, res);

        let extra_flags =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_ZERO;

        let mut register = Register::new();
        register.reg_sr =
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(true, res);
        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_gt_false() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_gt_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(true, res);
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        register.reg_sr =
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(true, res);
        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::GT);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_hi_false() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        register.reg_sr = STATUS_REGISTER_MASK_CARRY | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_hi_true() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::HI);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_le_false() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(false, res);
        register.reg_sr =
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_le_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY;

        register.reg_sr = STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_ls_false() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ls_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = STATUS_REGISTER_MASK_CARRY | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LS);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_lt_false() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(false, res);
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(false, res);

        let extra_flags =
            STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(false, res);
        register.reg_sr =
            STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_lt_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(true, res);

        let extra_flags =
            STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_ZERO;

        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(true, res);
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::LT);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_mi_false() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::MI);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::MI);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_mi_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::MI);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::MI);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_ne_false() {
        //  Z => EQ=TRUE => NE=FALSE
        // !Z => NE=TRUE => EQ=TRUE
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_ZERO;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::NE);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = STATUS_REGISTER_MASK_ZERO | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::NE);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_ne_true() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::NE);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::NE);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_pl_false() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::PL);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::PL);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_pl_true() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::PL);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::PL);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_t_true() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::T);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::T);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_vc_false() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VC);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VC);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_vc_true() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VC);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_vs_false() {
        let mut register = Register::new();
        register.reg_sr = 0x0000;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VS);
        assert_eq!(false, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = 0x0000 | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VS);
        assert_eq!(false, res);
    }

    #[test]
    fn evaluate_condition_vs_true() {
        let mut register = Register::new();
        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VS);
        assert_eq!(true, res);

        let extra_flags = STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;

        register.reg_sr = STATUS_REGISTER_MASK_OVERFLOW | extra_flags;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::VS);
        assert_eq!(true, res);
    }
}
