use std::convert::TryInto;

use crate::{
    cpu::instruction::{EffectiveAddressingData, EffectiveAddressingMode},
    mem::Mem,
};

pub const STATUS_REGISTER_MASK_CARRY: u16 = 0b0000000000000001;
pub const STATUS_REGISTER_MASK_OVERFLOW: u16 = 0b0000000000000010;
pub const STATUS_REGISTER_MASK_ZERO: u16 = 0b0000000000000100;
pub const STATUS_REGISTER_MASK_NEGATIVE: u16 = 0b0000000000001000;
pub const STATUS_REGISTER_MASK_EXTEND: u16 = 0b0000000000010000;

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

    pub fn skip_byte(&mut self) {
        self.address_next += 1;
    }

    pub fn fetch_next_unsigned_byte(&mut self, mem: &Mem) -> u8 {
        let word = mem.get_unsigned_byte(self.address_next);
        self.address_next += 1;
        word
    }

    pub fn fetch_next_signed_byte(&mut self, mem: &Mem) -> i8 {
        let word = mem.get_signed_byte(self.address_next);
        self.address_next += 1;
        word
    }

    pub fn peek_next_unsigned_word(&self, mem: &Mem) -> u16 {
        let word = mem.get_unsigned_word(self.address_next);
        word
    }

    pub fn fetch_next_unsigned_word(&mut self, mem: &Mem) -> u16 {
        let word = mem.get_unsigned_word(self.address_next);
        self.address_next += 2;
        word
    }

    pub fn fetch_next_signed_word(&mut self, mem: &Mem) -> i16 {
        let word = mem.get_signed_word(self.address_next);
        self.address_next += 2;
        word
    }

    pub fn fetch_next_unsigned_long(&mut self, mem: &Mem) -> u32 {
        let word = mem.get_unsigned_long(self.address_next);
        self.address_next += 4;
        word
    }

    pub fn fetch_next_signed_long(&mut self, mem: &Mem) -> i32 {
        let word = mem.get_signed_long(self.address_next);
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
        mem: &Mem,
    ) -> EffectiveAddressingData {
        let instr_word = self.fetch_next_unsigned_word(mem);
        self.get_effective_addressing_data_from_instr_word_bit_pos(instr_word, mem, 3, 0)
    }

    pub fn get_effective_addressing_data_from_instr_word_bit_pos(
        &mut self,
        instr_word: u16,
        mem: &Mem,
        bit_pos: u8,
        reg_bit_pos: u8,
    ) -> EffectiveAddressingData {
        let ea_mode = (instr_word >> bit_pos) & 0x0007;
        let register = (instr_word >> reg_bit_pos) & 0x0007;
        let register = register.try_into().unwrap();
        let ea_mode = match ea_mode {
            0b000 => EffectiveAddressingMode::DRegDirect {
                register: (register),
            },
            0b001 => EffectiveAddressingMode::ARegDirect {
                register: (register),
            },
            0b010 => EffectiveAddressingMode::ARegIndirect {
                register: (register),
            },
            0b011 => EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                register: (register),
            },
            0b100 => EffectiveAddressingMode::ARegIndirectWithPreDecrement {
                register: (register),
            },
            0b101 => EffectiveAddressingMode::ARegIndirectWithDisplacement {
                register: (register),
            },
            0b110 => EffectiveAddressingMode::ARegIndirectWithIndexOrMemoryIndirect {
                register: (register),
            },
            0b111 => match register {
                0b010 => EffectiveAddressingMode::PcIndirectWithDisplacement,
                0b011 => EffectiveAddressingMode::PcIndirectWithIndexOrPcMemoryIndirect,
                0b000 => EffectiveAddressingMode::AbsoluteShortAddressing,
                0b001 => EffectiveAddressingMode::AbsolutLongAddressing,
                0b100 => EffectiveAddressingMode::ImmediateData,
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

    pub fn is_sr_coverflow_set(&self) -> bool {
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
