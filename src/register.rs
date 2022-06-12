use crate::{
    cpu::{instruction::EffectiveAddressingData, Cpu},
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

    pub fn get_first_unsigned_word(&self, mem: &Mem) -> u16 {
        let word = mem.get_unsigned_word(self.address);
        word
    }

    pub fn fetch_next_unsigned_word(&mut self, mem: &Mem) -> u16 {
        let word = mem.get_unsigned_word(self.address_next);
        self.address_next += 2;
        println!("self.address_next: {}", self.address_next);
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
        println!("self.address: {}", self.address);
        println!("self.address_next: {}", self.address_next);
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
        let ea_mode =
            Cpu::extract_effective_addressing_mode_from_bit_pos_3_and_reg_pos_0(instr_word);
        EffectiveAddressingData::create(instr_word, ea_mode)
    }

    pub fn get_effective_addressing_data_from_bit_pos(
        &mut self,
        mem: &Mem,
        bit_pos: u8,
        reg_bit_pos: u8,
    ) -> EffectiveAddressingData {
        let instr_word = self.get_first_unsigned_word(mem);
        let ea_mode =
            Cpu::extract_effective_addressing_mode_from_bit_pos(instr_word, bit_pos, reg_bit_pos);
        EffectiveAddressingData::create(instr_word, ea_mode)
    }
    // pub fn get_instruction(&mut self, mem: &Mem) -> u16 {
    //     let instr_word = mem.get_unsigned_word(self.address_temp);
    //     self.address_temp += 2;
    //     instr_word
    // }
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
