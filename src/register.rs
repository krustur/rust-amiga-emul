pub struct Register {
    pub reg_d: [u32; 8],
    pub reg_a: [u32; 8],
    pub reg_pc: u32,
}

impl Register {
    pub fn new() -> Register {        
        let register = Register {
            reg_d: [0x00000000; 8],
            reg_a: [0x00000000; 8],
            reg_pc: 0x00000000,
        };
        register
    }
}