use crate::cpu::instruction::*;
use crate::mem::Mem;
use crate::register::*;
use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{convert::TryInto, fmt};

pub mod instruction;

pub struct EffectiveAddressValue<T> {
    pub address: u32,
    pub value: T,
    pub num_extension_words: u32,
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

pub struct Cpu {
    pub register: Register,
    pub memory: Mem,
    instructions: Vec<Instruction>,
}

impl Cpu {
    pub fn new(mem: Mem) -> Cpu {
        let reg_ssp = mem.get_unsigned_longword(0x0);
        let reg_pc = mem.get_unsigned_longword(0x4);
        let instructions = vec![
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                // InstructionFormat::EffectiveAddress {
                instruction::add::step,
                instruction::add::get_debug,
                // areg_direct_step: instruction::add::areg_direct_step_func,
                // areg_direct_get_debug: instruction::add::areg_direct_get_debug,
                // },
            ),
            Instruction::new(
                String::from("DBcc"),
                0xf0f8,
                0x50c8,
                instruction::dbcc::step,
                instruction::dbcc::get_debug,
            ),
            Instruction::new(
                String::from("ADDQ"),
                0xf100,
                0x5000,
                instruction::addq::step,
                instruction::addq::get_debug,
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
                instruction::bcc::get_debug,
            ),
            Instruction::new(
                String::from("CMP"),
                0xb000,
                0xb000,
                instruction::cmp::step,
                instruction::cmp::get_debug,
            ),
            Instruction::new(
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                instruction::lea::step,
                instruction::lea::get_debug,
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
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                instruction::moveq::step,
                instruction::moveq::get_debug,
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

    fn extract_effective_addressing_mode(word: u16) -> EffectiveAddressingMode {
        let ea_mode = (word >> 3) & 0x0007;
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

    pub fn extract_size_from_bit_pos_6(word: u16) -> Option<OperationSize> {
        let size = word & 0x0007;
        let size = (word >> 6) & 0x0003;
        match size {
            0b00 => Some(OperationSize::Byte),
            0b01 => Some(OperationSize::Word),
            0b10 => Some(OperationSize::Long),
            _ => None,
        }
    }

    pub fn get_byte_unsigned_from_long(long: u32) -> u8 {
        (long & 0x000000ff) as u8
    }

    pub fn get_word_unsigned_from_long(long: u32) -> u16 {
        (long & 0x0000ffff) as u16
    }

    pub fn get_byte_signed_from_long(long: u32) -> i8 {
        (long & 0x000000ff) as i8
    }

    pub fn get_word_signed_from_long(long: u32) -> i16 {
        (long & 0x0000ffff) as i16
    }

    pub fn get_long_signed_from_long(long: u32) -> i32 {
        long as i32
    }

    pub fn get_ea_format(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
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
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex => {
                panic!("get_ea_format() UNKNOWN_EA {:?} {}", ea_mode, ea_register);
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore => match ea_register {
                0b000 => {
                    // absolute short addressing mode
                    // (xxx).W
                    let extension_word = mem.get_signed_word(instr_address + 2);
                    let format = format!("(${:04X}).W", extension_word);
                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 1,
                    }
                }
                0b001 => {
                    // absolute long addressing mode
                    // (xxx).L
                    let first_extension_word = mem.get_unsigned_word(instr_address + 2);
                    let second_extension_word = mem.get_unsigned_word(instr_address + 4);
                    let address =
                        u32::from(first_extension_word) << 16 | u32::from(second_extension_word);
                    let format = format!("(${:08X}).L", address);
                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 2,
                    }
                }
                0b010 => {
                    // PC indirect with displacement mode
                    // (d16,PC)
                    let extension_word = mem.get_signed_word(instr_address + 2);
                    let ea = Cpu::get_address_with_i16_displacement(reg.reg_pc + 2, extension_word);
                    let format = format!("({:#06x},PC)", extension_word);

                    EffectiveAddressDebug {
                        format: format,
                        num_extension_words: 1,
                    }
                }
                0b011 => {
                    // (d8,PC,Xn)
                    panic!(
                        "get_ea_format() UNKNOWN_PcIndirectAndLotsMore 0b011 {:?} {}",
                        ea_mode, ea_register
                    );
                }
                _ => {
                    panic!(
                        "get_ea_format() UNKNOWN_PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
            },
        }
    }

    // pub fn get_ea (
    //     ea_mode: EffectiveAddressingMode,
    //     ea_register: usize,
    //     reg: &mut Register,
    //     mem: &Mem,
    // ) -> u32 {
    //     match ea_mode {
    //         EffectiveAddressingMode::DRegDirect => reg.reg_d[ea_register],
    //         EffectiveAddressingMode::ARegDirect => reg.reg_a[ea_register],
    //         EffectiveAddressingMode::ARegIndirect
    //         | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
    //             reg.reg_a[ea_register]
    //         }
    //         EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
    //             reg.reg_a[ea_register] += 1;
    //             reg.reg_a[ea_register]
    //         }
    //         EffectiveAddressingMode::ARegIndirectWithDisplacement
    //         | EffectiveAddressingMode::ARegIndirectWithIndex
    //         | EffectiveAddressingMode::PcIndirectAndLotsMore => {
    //             panic!(
    //                 "get_ea() UNKNOWN_EA {:?} {}",
    //                 ea_mode, ea_register
    //             );
    //         }
    //     }
    // }

    pub fn get_ea_value_unsigned_byte(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> u8 {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                Cpu::get_byte_unsigned_from_long(reg.reg_d[ea_register])
            }
            EffectiveAddressingMode::ARegDirect => {
                Cpu::get_byte_unsigned_from_long(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                mem.get_unsigned_byte(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                mem.get_unsigned_byte(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex
            | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                panic!(
                    "get_ea_value_unsigned_byte() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
        }
    }

    pub fn get_ea_value_signed_byte(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> i8 {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                Cpu::get_byte_signed_from_long(reg.reg_d[ea_register])
            }
            EffectiveAddressingMode::ARegDirect => {
                Cpu::get_byte_signed_from_long(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                mem.get_signed_byte(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                mem.get_signed_byte(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex
            | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                panic!(
                    "get_ea_value_signed_byte() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
        }
    }

    pub fn get_ea_value_unsigned_word(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> u16 {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                Cpu::get_word_unsigned_from_long(reg.reg_d[ea_register])
            }
            EffectiveAddressingMode::ARegDirect => {
                Cpu::get_word_unsigned_from_long(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                mem.get_unsigned_word(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                mem.get_unsigned_word(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex
            | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                panic!(
                    "get_ea_value_unsigned_word() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
        }
    }

    pub fn get_ea_value_signed_word(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> i16 {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                Cpu::get_word_signed_from_long(reg.reg_d[ea_register])
            }
            EffectiveAddressingMode::ARegDirect => {
                Cpu::get_word_signed_from_long(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                mem.get_signed_word(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                mem.get_signed_word(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex
            | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                panic!(
                    "get_ea_value_signed_word() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
        }
    }

    pub fn get_ea_value_unsigned_long(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> EffectiveAddressValue<u32> {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                let address = reg.reg_d[ea_register];
                let value = reg.reg_d[ea_register];
                EffectiveAddressValue {
                    address: address,
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegDirect => {
                let address = reg.reg_a[ea_register];
                let value = reg.reg_a[ea_register];
                EffectiveAddressValue {
                    address: address,
                    num_extension_words: 0,
                    value: value,
                }
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                let address = reg.reg_a[ea_register];
                let value = mem.get_unsigned_longword(address);
                let res = EffectiveAddressValue {
                    address: address,
                    num_extension_words: 0,
                    value: value,
                };
                res
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                let address = reg.reg_a[ea_register];
                let value = mem.get_unsigned_longword(address);
                let res = EffectiveAddressValue {
                    address: address,
                    num_extension_words: 0,
                    value: value,
                };
                res
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
                    // absolute short addressing mode
                    // (xxx).W
                    let extension_word = mem.get_signed_word(instr_address + 2);
                    let address = Cpu::sign_extend_i16(extension_word);
                    // let ea_format = format!("({:#06x}).W", extension_word);
                    let value = mem.get_unsigned_longword(address);
                    EffectiveAddressValue {
                        address: address,
                        num_extension_words: 1,
                        value: value,
                    }
                }
                0b001 => {
                    // absolute long addressing mode
                    // (xxx).L
                    let first_extension_word = mem.get_unsigned_word(instr_address + 2);
                    let second_extension_word = mem.get_unsigned_word(instr_address + 4);
                    let address =
                        u32::from(first_extension_word) << 16 | u32::from(second_extension_word);
                    let value = mem.get_unsigned_longword(address);
                    EffectiveAddressValue {
                        address: address,
                        num_extension_words: 2,
                        value: value,
                    }
                }
                0b010 => {
                    // // PC indirect with displacement mode
                    // // (d16,PC)
                    // let extension_word = self.memory.get_signed_word(instr_addr + 2);
                    // let ea = Cpu::get_address_with_i16_displacement(
                    //     self.register.reg_pc + 2,
                    //     extension_word,
                    // );
                    // //  let operand =
                    // //  self.memory.get_unsigned_longword_with_i16_displacement(
                    // //         instr_addr + 2,
                    // //         extension_word,
                    // //     );
                    // let ea_format = format!("({:#06x},PC)", extension_word);
                    // // let debug_result = common_get_debug(
                    // //     instr_addr,
                    // //     instr_word,
                    // //     &mut self.register,
                    // //     &mut self.memory,
                    // //     ea_format,
                    // //     ea,
                    // // );
                    // // let exec_result = common_step(
                    // //     instr_addr,
                    // //     instr_word,
                    // //     &mut self.register,
                    // //     &mut self.memory,
                    // //     ea,
                    // // );
                    // // (exec_result, debug_result)
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore 0b010 {:?} {}",
                        ea_mode, ea_register
                    );
                }
                0b011 => {
                    // (d8,PC,Xn)
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore 0b011 {:?} {}",
                        ea_mode, ea_register
                    );
                }
                _ => {
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
            },
        }
    }

    pub fn get_ea_value_signed_long(
        ea_mode: EffectiveAddressingMode,
        ea_register: usize,
        instr_address: u32,
        reg: &mut Register,
        mem: &Mem,
    ) -> i32 {
        match ea_mode {
            EffectiveAddressingMode::DRegDirect => {
                Cpu::get_long_signed_from_long(reg.reg_d[ea_register])
            }
            EffectiveAddressingMode::ARegDirect => {
                Cpu::get_long_signed_from_long(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirect
            | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                mem.get_signed_longword(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
                reg.reg_a[ea_register] += 1;
                mem.get_signed_longword(reg.reg_a[ea_register])
            }
            EffectiveAddressingMode::ARegIndirectWithDisplacement
            | EffectiveAddressingMode::ARegIndirectWithIndex => {
                panic!(
                    "get_ea_value_signed_long() UNKNOWN_EA {:?} {}",
                    ea_mode, ea_register
                );
            }
            EffectiveAddressingMode::PcIndirectAndLotsMore => match ea_register {
                0b000 => {
                    // absolute short addressing mode
                    // (xxx).W
                    let extension_word = mem.get_signed_word(instr_address + 2);
                    let ea = Cpu::sign_extend_i16(extension_word);
                    // let ea_format = format!("({:#06x}).W", extension_word);
                    // BUG: This should p
                    mem.get_signed_longword(ea)
                }
                0b001 => {
                    // (xxx).L
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_EA {:?} {}",
                        ea_mode, ea_register
                    );
                }
                0b010 => {
                    // // PC indirect with displacement mode
                    // // (d16,PC)
                    // let extension_word = self.memory.get_signed_word(instr_addr + 2);
                    // let ea = Cpu::get_address_with_i16_displacement(
                    //     self.register.reg_pc + 2,
                    //     extension_word,
                    // );
                    // //  let operand =
                    // //  self.memory.get_unsigned_longword_with_i16_displacement(
                    // //         instr_addr + 2,
                    // //         extension_word,
                    // //     );
                    // let ea_format = format!("({:#06x},PC)", extension_word);
                    // // let debug_result = common_get_debug(
                    // //     instr_addr,
                    // //     instr_word,
                    // //     &mut self.register,
                    // //     &mut self.memory,
                    // //     ea_format,
                    // //     ea,
                    // // );
                    // // let exec_result = common_step(
                    // //     instr_addr,
                    // //     instr_word,
                    // //     &mut self.register,
                    // //     &mut self.memory,
                    // //     ea,
                    // // );
                    // // (exec_result, debug_result)
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
                0b011 => {
                    // (d8,PC,Xn)
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
                _ => {
                    panic!(
                        "get_ea_value_signed_long() UNKNOWN_PcIndirectAndLotsMore {:?} {}",
                        ea_mode, ea_register
                    );
                }
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
            ConditionalTest::F => false,
            ConditionalTest::CC => reg.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000,
            _ => panic!("ConditionalTest not implemented"),
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
        // let instruction = self.get_instruction(instr_addr, instr_word);

        // // TODO: Expose the debugger somewhere else, not when executing instructions
        // let debug_string = match instruction.instruction_format {
        //     InstructionFormat::Uncommon{step, get_debug} => {
        //         println!("{}", instruction.name);
        //         get_debug(instr_addr, instr_word, &mut self.register, &mut self.memory)
        //     }
        // };

        // let (exec_result, debug_result) = //match instruction.instruction_format {
        // InstructionFormat::Uncommon { step, get_debug } => {
        // println!("{}", instruction.name);

        // (exec_result, debug_result)
        // }
        // InstructionFormat::EffectiveAddress {
        //     common_step,
        //     common_get_debug,
        //     areg_direct_step,
        //     areg_direct_get_debug,
        // } => {
        //     let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
        //     let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

        //     match ea_mode {
        //         EffectiveAddressingMode::DRegDirect => {
        //             panic!(
        //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //                 instr_addr, instr_word, ea_mode, ea_register
        //             );
        //         }
        //         EffectiveAddressingMode::ARegDirect => {
        //             let debug_result = areg_direct_get_debug(
        //                 instr_addr,
        //                 instr_word,
        //                 &mut self.register,
        //                 &mut self.memory,
        //                 ea_register,
        //             );
        //             let exec_result = areg_direct_step(
        //                 instr_addr,
        //                 instr_word,
        //                 &mut self.register,
        //                 &mut self.memory,
        //                 ea_register,
        //             );
        //             (exec_result, debug_result)
        //         }
        //         EffectiveAddressingMode::ARegIndirect
        //         | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
        //             let ea = self.register.reg_a[ea_register];
        //             let ea_format = format!("(A{})+", ea_register);

        //             let debug_result = common_get_debug(
        //                 instr_addr,
        //                 instr_word,
        //                 &mut self.register,
        //                 &mut self.memory,
        //                 ea_format,
        //                 ea,
        //             );
        //             let exec_result = common_step(
        //                 instr_addr,
        //                 instr_word,
        //                 &mut self.register,
        //                 &mut self.memory,
        //                 ea,
        //             );

        //             if ea_mode == EffectiveAddressingMode::ARegIndirectWithPostIncrement {
        //                 if let InstructionExecutionResult::Done { pc_result } = exec_result {
        //                     // todo!("post incrementen");
        //                     self.register.reg_a[ea_register] += 2; //op_size.size_in_bytes();
        //                 }
        //             }
        //             (exec_result, debug_result)
        //         }
        //         EffectiveAddressingMode::ARegIndirectWithPreDecrement
        //         | EffectiveAddressingMode::ARegIndirectWithDisplacement
        //         | EffectiveAddressingMode::ARegIndirectWithIndex => {
        //             panic!(
        //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //                 instr_addr, instr_word, ea_mode, ea_register
        //             );
        //             // pc_increment = Some(2);
        //         }
        //         EffectiveAddressingMode::PcIndirectAndLotsMore => {
        //             match ea_register {
        //                 0b000 => {
        //                     // absolute short addressing mode
        //                     // (xxx).W
        //                     let extension_word = self.memory.get_signed_word(instr_addr + 2);
        //                     let ea = Cpu::sign_extend_i16(extension_word);
        //                     let ea_format = format!("({:#06x}).W", extension_word);
        //                     let debug_result = common_get_debug(
        //                         instr_addr,
        //                         instr_word,
        //                         &mut self.register,
        //                         &mut self.memory,
        //                         ea_format,
        //                         ea,
        //                     );
        //                     let exec_result = common_step(
        //                         instr_addr,
        //                         instr_word,
        //                         &mut self.register,
        //                         &mut self.memory,
        //                         ea,
        //                     );
        //                     (exec_result, debug_result)
        //                 }
        //                 0b001 => {
        //                     // (xxx).L
        //                     panic!(
        //                         "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //                         instr_addr, instr_word, ea_mode, ea_register
        //                     );
        //                 }
        //                 0b010 => {
        //                     // PC indirect with displacement mode
        //                     // (d16,PC)
        //                     let extension_word = self.memory.get_signed_word(instr_addr + 2);
        //                     let ea = Cpu::get_address_with_i16_displacement(
        //                         self.register.reg_pc + 2,
        //                         extension_word,
        //                     );
        //                     //  let operand =
        //                     //  self.memory.get_unsigned_longword_with_i16_displacement(
        //                     //         instr_addr + 2,
        //                     //         extension_word,
        //                     //     );
        //                     let ea_format = format!("({:#06x},PC)", extension_word);
        //                     let debug_result = common_get_debug(
        //                         instr_addr,
        //                         instr_word,
        //                         &mut self.register,
        //                         &mut self.memory,
        //                         ea_format,
        //                         ea,
        //                     );
        //                     let exec_result = common_step(
        //                         instr_addr,
        //                         instr_word,
        //                         &mut self.register,
        //                         &mut self.memory,
        //                         ea,
        //                     );
        //                     (exec_result, debug_result)
        //                 }
        //                 0b011 => {
        //                     // (d8,PC,Xn)
        //                     panic!(
        //                         "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //                         instr_addr, instr_word, ea_mode, ea_register
        //                     );
        //                 }
        //                 _ => {
        //                     panic!(
        //                         "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //                         instr_addr, instr_word, ea_mode, ea_register
        //                     );
        //                 }
        //             }
        //         }
        //     }
        // } // InstructionFormat::EffectiveAddressWithOpmodeAndRegister(exec_func) => {
        //   //     let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        //   //     let ea_opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
        //   //     let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
        //   //     let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
        //   //     println!(
        //   //         "register {} ea_mode {:?} ea_register {} ea_opmode {:?} ",
        //   //         register, ea_mode, ea_register, ea_opmode
        //   //     );

        //   //     match ea_mode {
        //   //         EffectiveAddressingMode::DRegDirect | EffectiveAddressingMode::ARegDirect => {
        //   //             panic!(
        //   //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //   //                 instr_addr, instr_word, ea_mode, ea_register
        //   //             );
        //   //         }
        //   //         EffectiveAddressingMode::ARegIndirect
        //   //         | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
        //   //             let ea = self.register.reg_a[ea_register];
        //   //             let ea_format = format!("(A{})+", ea_register);

        //   //             // let operand = self.memory.get_unsigned_longword(ea);
        //   //             let exec_result = exec_func(
        //   //                 instr_addr,
        //   //                 instr_word,
        //   //                 &mut self.register,
        //   //                 &mut self.memory,
        //   //                 ea_format,
        //   //                 ea_opmode,
        //   //                 register,
        //   //                 ea,
        //   //             );
        //   //             if ea_mode == EffectiveAddressingMode::ARegIndirectWithPostIncrement {
        //   //                 self.register.reg_a[ea_register] += exec_result.op_size.size_in_bytes();
        //   //             }
        //   //             exec_result
        //   //         }
        //   //         EffectiveAddressingMode::ARegIndirectWithPreDecrement
        //   //         | EffectiveAddressingMode::ARegIndirectWithDisplacement
        //   //         | EffectiveAddressingMode::ARegIndirectWithIndex
        //   //         | EffectiveAddressingMode::PcIndirectAndLotsMore => {
        //   //             panic!(
        //   //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
        //   //                 instr_addr, instr_word, ea_mode, ea_register
        //   //             );
        //   //         }
        //   //     }
        //   //     // panic!("EffectiveAddressWithOpmodeAndRegister not quite done");
        //   // }
        // };

        let step = instruction.step;
        let exec_result = step(instr_addr, instr_word, &mut self.register, &mut self.memory);

        if let InstructionExecutionResult::Done {
            // comment,
            // op_size,
            pc_result,
        } = &exec_result
        {
            self.register.reg_pc = match pc_result {
                PcResult::Set(new_pc) => *new_pc,
                PcResult::Increment(pc_increment) => self.register.reg_pc + pc_increment,
            }
        }

        // debug_result
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

        // if let DisassemblyResult::Done {
        //     name,
        //     operands_format,
        //     next_instr_address,
        // } = &debug_result
        // {
        //     let instr_format = format!("{} {}", name, operands_format);
        //     print!("{:#010X} ", instr_addr);
        //     for i in (instr_addr..instr_addr + 8).step_by(2) {
        //         if i < *next_instr_address {
        //             let op_mem = self.memory.get_unsigned_word(i);
        //             print!("{:04X} ", op_mem);
        //         } else {
        //             print!("     ");
        //         }
        //     }
        //     println!("{: <30}", instr_format);
        // }

        debug_result
    }

    pub fn print_disassembly(self: &mut Cpu, debug_result: &DisassemblyResult) {
        if let DisassemblyResult::Done {
            name,
            instr_address,
            operands_format,
            next_instr_address,
        } = &debug_result
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
    fn evaluate_condition_cc_cleared() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_cc_set() {
        let mut register = Register::new();
        register.reg_sr = 0x0001;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(false, res);
    }
}
