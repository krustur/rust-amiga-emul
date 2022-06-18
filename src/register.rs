use crate::{
    cpu::{
        ea::EffectiveAddressingData,
        instruction::{EffectiveAddressingMode, OperationSize, ScaleFactor},
        Cpu,
    },
    mem::Mem,
};

pub const STATUS_REGISTER_MASK_CARRY: u16 = 0b0000000000000001;
pub const STATUS_REGISTER_MASK_OVERFLOW: u16 = 0b0000000000000010;
pub const STATUS_REGISTER_MASK_ZERO: u16 = 0b0000000000000100;
pub const STATUS_REGISTER_MASK_NEGATIVE: u16 = 0b0000000000001000;
pub const STATUS_REGISTER_MASK_EXTEND: u16 = 0b0000000000010000;

#[derive(Copy, Clone, Debug, std::cmp::PartialEq)]
pub enum RegisterType {
    Data,
    Address,
}

impl RegisterType {
    pub fn get_format(&self) -> char {
        match self {
            RegisterType::Address => 'A',
            RegisterType::Data => 'D',
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ProgramCounter {
    address: u32,
    address_next: u32,
}

impl ProgramCounter {
    pub fn from_address(address: u32) -> ProgramCounter {
        ProgramCounter {
            address,
            address_next: address,
        }
    }

    pub fn from_address_and_address_next(address: u32, address_next: u32) -> ProgramCounter {
        ProgramCounter {
            address,
            address_next,
        }
    }

    pub fn branch_byte(&mut self, displacement: u8) {
        self.address_next = Cpu::get_address_with_byte_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        )
    }

    pub fn branch_word(&mut self, displacement: u16) {
        self.address_next = Cpu::get_address_with_word_displacement_sign_extended(
            self.address.wrapping_add(2),
            displacement,
        )
    }

    pub fn skip_byte(&mut self) {
        self.address_next += 1;
    }

    pub fn fetch_next_byte(&mut self, mem: &Mem) -> u8 {
        let word = mem.get_byte(self.address_next);
        self.address_next += 1;
        word
    }

    pub fn peek_next_word(&self, mem: &Mem) -> u16 {
        let word = mem.get_word(self.address_next);
        word
    }

    pub fn fetch_next_word(&mut self, mem: &Mem) -> u16 {
        let word = mem.get_word(self.address_next);
        self.address_next += 2;
        word
    }

    pub fn fetch_next_long(&mut self, mem: &Mem) -> u32 {
        let word = mem.get_long(self.address_next);
        self.address_next += 4;
        word
    }

    pub fn get_next_pc(&self) -> ProgramCounter {
        ProgramCounter {
            address: self.address_next,
            address_next: self.address_next,
        }
    }

    pub fn get_address(&self) -> u32 {
        self.address
    }

    pub fn get_address_next(&self) -> u32 {
        self.address_next
    }

    pub fn fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        &mut self,
        reg: &Register,
        mem: &Mem,
        operation_size: Option<OperationSize>,
    ) -> EffectiveAddressingData {
        // TODO: Replace operation_size with Closure
        let instr_word = self.fetch_next_word(mem);
        self.get_effective_addressing_data_from_instr_word_bit_pos(
            instr_word,
            reg,
            mem,
            operation_size,
            3,
            0,
        )
    }

    pub fn get_effective_addressing_data_from_instr_word_bit_pos(
        &mut self,
        instr_word: u16,
        reg: &Register,
        mem: &Mem,
        operation_size: Option<OperationSize>,
        bit_pos: u8,
        reg_bit_pos: u8,
    ) -> EffectiveAddressingData {
        let ea_mode = (instr_word >> bit_pos) & 0x0007;
        let ea_register = Cpu::extract_register_index_from_bit_pos(instr_word, reg_bit_pos);
        let ea_mode = match ea_mode {
            0b000 => EffectiveAddressingMode::DRegDirect {
                ea_register: (ea_register),
            },
            0b001 => EffectiveAddressingMode::ARegDirect {
                ea_register: (ea_register),
            },
            0b010 => {
                let address = reg.reg_a[ea_register];

                EffectiveAddressingMode::ARegIndirect {
                    ea_register,
                    ea_address: address,
                }
            }
            0b011 => {
                let address = reg.reg_a[ea_register];
                let operation_size = match operation_size {
                    None => panic!("Must have operation_size for ARegIndirectWithPostIncrement!"),
                    Some(operation_size) => operation_size,
                };
                // reg.reg_a[register] += size_in_bytes;
                EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                    operation_size,
                    ea_register,
                    ea_address: address,
                }
            }
            0b100 => {
                // (-An)
                let operation_size = match operation_size {
                    None => panic!("Must have operation_size for ARegIndirectWithPreDecrement!"),
                    Some(operation_size) => operation_size,
                };
                // reg.reg_a[register] -= size_in_bytes;
                let address = reg.reg_a[ea_register] - operation_size.size_in_bytes();
                EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                    operation_size,
                    ea_register,
                    ea_address: address,
                }
            }
            0b101 => {
                let displacement = self.fetch_next_word(mem);
                let (address, _) =
                    reg.reg_a[ea_register].overflowing_add(Cpu::sign_extend_word(displacement));

                EffectiveAddressingMode::ARegIndirectWithDisplacement {
                    ea_register,
                    ea_address: address,
                    ea_displacement: displacement,
                }
            }
            0b110 => {
                let extension_word = self.fetch_next_word(mem);
                let displacement = Cpu::get_byte_from_word(extension_word);
                let register = Cpu::extract_register_index_from_bit_pos(extension_word, 12);
                let (register_value, register_type) = match extension_word & 0x8000 {
                    0x8000 => (reg.reg_a[register], RegisterType::Address),
                    _ => (reg.reg_d[register], RegisterType::Data),
                };
                // let register_type = match extension_word & 0x8000 {
                //     0x8000 => RegisterType::Address,
                //     _ => RegisterType::Data,
                // };
                let (register_value, index_size) = match extension_word & 0x0800 {
                    0x0800 => (register_value, OperationSize::Long),
                    _ => (
                        Cpu::sign_extend_word(Cpu::get_word_from_long(register_value)),
                        OperationSize::Word,
                    ),
                };
                // let operation_size = match extension_word & 0x0800 {
                //     0x0800 => OperationSize::Long,
                //     _ => OperationSize::Word,
                // };
                let scale_factor = Cpu::extract_scale_factor_from_bit_pos(extension_word, 9);
                let extension_word_format = match extension_word & 0x0100 {
                    0x0100 => 'F', // full
                    _ => 'B',      // brief
                };
                if extension_word_format == 'F' {
                    todo!("Full extension word format not implemented")
                }
                let register_value = match scale_factor {
                    ScaleFactor::One => register_value,
                    ScaleFactor::Two => register_value << 1,
                    ScaleFactor::Four => register_value << 2,
                    ScaleFactor::Eight => register_value << 3,
                };

                let displacement_long = Cpu::sign_extend_byte(displacement);
                let (ea_address, _) = reg.reg_a[ea_register].overflowing_add(displacement_long);
                let (ea_address, _) = ea_address.overflowing_add(register_value);

                EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect {
                    ea_register,
                    ea_address,
                    extension_word,
                    displacement,
                    register_type,
                    register,
                    index_size,
                    scale_factor,
                }
            }
            0b111 => match ea_register {
                0b010 => {
                    let displacement = self.fetch_next_word(mem);
                    let ea_address = Cpu::get_address_with_word_displacement_sign_extended(
                        reg.reg_pc.get_address() + 2,
                        displacement,
                    );
                    EffectiveAddressingMode::PcIndirectWithDisplacement {
                        ea_address,
                        displacement,
                    }
                }
                0b011 => {
                    // panic!();
                    let extension_word = self.fetch_next_word(mem);
                    let register = Cpu::extract_register_index_from_bit_pos(extension_word, 12);
                    // BUG: Compare this with ARegIndirectWithIndexOrMemoryIndirect above. Is it really correct to use index_size_bytes below?
                    //      Scale factor is never use! Probably green cause we test with *4 and .L that have a matching size
                    let (index_size, index_size_bytes) = match extension_word & 0x0800 {
                        0x0800 => (OperationSize::Long, 4),
                        _ => (OperationSize::Word, 2),
                    };
                    let scale_factor = Cpu::extract_scale_factor_from_bit_pos(extension_word, 9);
                    let (register_type, register_displacement) = match extension_word & 0x8000 {
                        0x8000 => (
                            RegisterType::Address,
                            reg.reg_a[register] * index_size_bytes,
                        ),
                        _ => (RegisterType::Data, reg.reg_d[register] * index_size_bytes),
                    };
                    let extension_word_format = match extension_word & 0x0100 {
                        0x0100 => 'F', // full
                        _ => 'B',      // brief
                    };
                    if extension_word_format == 'F' {
                        todo!("Full extension word format not implemented")
                    }
                    let displacement = Cpu::get_byte_from_word(extension_word);
                    let address = Cpu::get_address_with_byte_displacement_sign_extended(
                        reg.reg_pc.get_address() + 2,
                        displacement,
                    );
                    let ea_address =
                        Cpu::get_address_with_u32_displacement(address, register_displacement);

                    EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect {
                        ea_register,
                        ea_address,
                        extension_word,
                        displacement,
                        register_type,
                        register,
                        index_size,
                        scale_factor,
                    }
                }
                0b000 => {
                    let displacement = self.fetch_next_word(mem);
                    let ea_address = Cpu::sign_extend_word(displacement);
                    EffectiveAddressingMode::AbsoluteShortAddressing {
                        ea_address,
                        displacement,
                    }
                }
                0b001 => {
                    let ea_address = self.fetch_next_long(mem);
                    EffectiveAddressingMode::AbsolutLongAddressing { ea_address }
                }
                0b100 => match operation_size {
                    None => panic!("Must have operation_size for Immediate data!"),
                    Some(operation_size) => match operation_size {
                        OperationSize::Byte => {
                            self.skip_byte();
                            let data = self.fetch_next_byte(mem);
                            EffectiveAddressingMode::ImmediateDataByte { data }
                        }
                        OperationSize::Word => {
                            let data = self.fetch_next_word(mem);
                            EffectiveAddressingMode::ImmediateDataWord { data }
                        }
                        OperationSize::Long => {
                            let data = self.fetch_next_long(mem);
                            EffectiveAddressingMode::ImmediateDataLong { data }
                        }
                    },
                },
                _ => panic!("Unable to extract EffectiveAddressingMode"),
            },
            _ => panic!("Unable to extract EffectiveAddressingMode"),
        };
        EffectiveAddressingData::create(instr_word, ea_mode)
    }
}

pub struct Register {
    pub reg_d: [u32; 8],
    pub reg_a: [u32; 8],
    pub reg_sr: u16,
    pub reg_pc: ProgramCounter,
}

impl Register {
    pub fn new() -> Register {
        let register = Register {
            reg_d: [0x00000000; 8],
            reg_a: [0x00000000; 8],
            reg_sr: 0x0000,
            reg_pc: ProgramCounter::from_address(0x00000000),
        };
        register
    }

    pub fn is_sr_carry_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_CARRY) == STATUS_REGISTER_MASK_CARRY;
    }

    pub fn is_sr_overflow_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW) == STATUS_REGISTER_MASK_OVERFLOW;
    }

    pub fn is_sr_zero_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_ZERO) == STATUS_REGISTER_MASK_ZERO;
    }

    pub fn is_sr_negative_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE) == STATUS_REGISTER_MASK_NEGATIVE;
    }

    pub fn is_sr_extend_set(&self) -> bool {
        return (self.reg_sr & STATUS_REGISTER_MASK_EXTEND) == STATUS_REGISTER_MASK_EXTEND;
    }
}
