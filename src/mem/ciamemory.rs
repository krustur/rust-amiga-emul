use super::memory::Memory;
use std::fmt;

pub struct CiaMemory {}

impl fmt::Display for CiaMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ROM: ${:08X}-${:08X} ({}) bytes)",
            self.get_start_address(),
            self.get_end_address(),
            self.get_length()
        )
    }
}

impl Memory for CiaMemory {
    fn get_start_address(&self) -> u32 {
        return 0x00BF0000;
    }

    fn get_end_address(&self) -> u32 {
        return 0x00BFFFFF;
    }

    fn get_length(&self) -> usize {
        return 0x10000;
    }

    fn get_long(self: &CiaMemory, address: u32) -> u32 {
        panic!("cia memory get_long");
    }

    fn set_long(self: &mut CiaMemory, address: u32, value: u32) {
        panic!("cia memory set_long");
    }

    fn get_word(self: &CiaMemory, address: u32) -> u16 {
        panic!("cia memory get_word");
    }

    fn set_word(self: &mut CiaMemory, address: u32, value: u16) {
        panic!("cia memory set_word");
    }

    fn get_byte(self: &CiaMemory, address: u32) -> u8 {
        println!("   -TODO: get_byte() for CIA memory ${:06X}", address);
        0
    }

    fn set_byte(self: &mut CiaMemory, address: u32, value: u8) {
        println!(
            "   -TODO: set_byte() for CIA memory ${:06X} to {}",
            address, value
        )
    }
}

impl CiaMemory {
    pub fn new() -> CiaMemory {
        CiaMemory {}
    }
}
