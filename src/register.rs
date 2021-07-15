use num_derive::ToPrimitive;

#[derive(ToPrimitive, Debug)]
pub enum StatusRegisterMask {
    Carry = 0b0000000000000001,
    Overflow = 0b0000000000000010,
    Zero = 0b0000000000000100,
    Negative = 0b0000000000001000,
    Extend = 0b0000000000010000,
}

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
}
