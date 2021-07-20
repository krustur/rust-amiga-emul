pub const STATUS_REGISTER_MASK_CARRY: u16 = 0b0000000000000001;
pub const STATUS_REGISTER_MASK_OVERFLOW: u16 = 0b0000000000000010;
pub const STATUS_REGISTER_MASK_ZERO: u16 = 0b0000000000000100;
pub const STATUS_REGISTER_MASK_NEGATIVE: u16 = 0b0000000000001000;
pub const STATUS_REGISTER_MASK_EXTEND: u16 = 0b0000000000010000;

pub struct Register {
    pub reg_d: [u32; 8],
    pub reg_a: [u32; 8],
    pub reg_sr: u16,
    pub reg_pc: u32,
}

impl Register {
    pub fn new() -> Register {
        let register = Register {
            reg_d: [0x00000000; 8],
            reg_a: [0x00000000; 8],
            reg_sr: 0x0000,
            reg_pc: 0x00000000,
        };
        register
    }

    pub fn is_sr_carry_set(&self) -> bool {
        return  (self.reg_sr & STATUS_REGISTER_MASK_CARRY) == STATUS_REGISTER_MASK_CARRY;
    }
   
    pub fn is_sr_coverflow_set(&self) -> bool {
        return  (self.reg_sr & STATUS_REGISTER_MASK_OVERFLOW) == STATUS_REGISTER_MASK_OVERFLOW;
    }

    pub fn is_sr_zero_set(&self) -> bool {
        return  (self.reg_sr & STATUS_REGISTER_MASK_ZERO) == STATUS_REGISTER_MASK_ZERO;
    }

    pub fn is_sr_negative_set(&self) -> bool {
        return  (self.reg_sr & STATUS_REGISTER_MASK_NEGATIVE) == STATUS_REGISTER_MASK_NEGATIVE;
    }

    pub fn is_sr_extend_set(&self) -> bool {
        return  (self.reg_sr & STATUS_REGISTER_MASK_EXTEND) == STATUS_REGISTER_MASK_EXTEND;
    }
}
