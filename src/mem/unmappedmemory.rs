use super::memory::Memory;
use std::fmt::{self};

pub struct UnmappedMemory {
    pub start_address: u32,
    pub end_address: u32,
    length: usize,
}

impl fmt::Display for UnmappedMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UNMAPPED: ${:08X}-${:08X} ({}) bytes)",
            self.start_address, self.end_address, self.length
        )
    }
}

impl Memory for UnmappedMemory {
    fn get_start_address(&self) -> u32 {
        return self.start_address;
    }

    fn get_end_address(&self) -> u32 {
        return self.end_address;
    }

    fn get_length(&self) -> usize {
        return self.length;
    }

    fn get_long(self: &UnmappedMemory, address: u32) -> u32 {
        println!(
            "   -Trying to get_long on UNMAPPED memory: ${:08X}",
            address
        );
        0
    }

    fn set_long(self: &mut UnmappedMemory, address: u32, value: u32) {
        println!(
            "   -Trying to set_long on UNMAPPED memory: ${:08X}",
            address
        );
    }

    fn get_word(self: &UnmappedMemory, address: u32) -> u16 {
        println!(
            "   -Trying to get_word on UNMAPPED memory: ${:08X}",
            address
        );
        0
    }

    fn set_word(self: &mut UnmappedMemory, address: u32, value: u16) {
        println!(
            "   -Trying to set_word on UNMAPPED memory: ${:08X}",
            address
        );
    }

    fn get_byte(self: &UnmappedMemory, address: u32) -> u8 {
        println!(
            "   -Trying to get_byte on UNMAPPED memory: ${:08X}",
            address
        );
        0
    }

    fn set_byte(self: &mut UnmappedMemory, address: u32, value: u8) {
        println!(
            "   -Trying to set_byte on UNMAPPED memory: ${:08X}",
            address
        );
    }
}

impl UnmappedMemory {
    pub fn new(start_address: u32, end_address: u32) -> UnmappedMemory {
        let length = end_address as usize - start_address as usize + 1usize;

        UnmappedMemory {
            start_address,
            end_address,
            length,
        }
    }
}
