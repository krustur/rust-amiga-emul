use super::memory::{Memory, SetMemoryResult};
use std::{any::Any, fmt};

pub struct CustomMemory {
    pub color_rgb4: [u16; 32],
}

impl fmt::Display for CustomMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CUSTOM: ${:08X}-${:08X} ({}) bytes)",
            self.get_start_address(),
            self.get_end_address(),
            self.get_length()
        )
    }
}

impl Memory for CustomMemory {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_start_address(&self) -> u32 {
        self.get_start_address()
    }

    fn get_end_address(&self) -> u32 {
        self.get_end_address()
    }

    fn get_length(&self) -> usize {
        self.get_length()
    }

    fn get_long(self: &CustomMemory, address: u32) -> u32 {
        panic!("custom memory get_long: ${:06X}", address);
    }

    fn set_long(self: &mut CustomMemory, address: u32, value: u32) {
        panic!("custom memory set_long: ${:06X}", address);
    }

    fn get_word(self: &CustomMemory, address: u32) -> u16 {
        panic!("custom memory get_word: ${:06X}", address);
    }

    fn set_word(self: &mut CustomMemory, address: u32, value: u16) {
        match address {
            0xDFF180..=0xDFF1Be => {
                let color_index = (address as usize - 0xDFF180) / 2;
                self.set_color_rgb4(color_index, value);
            }
            _ => {
                println!(
                    "   -CUSTOM: TODO: set_word() for CUSTOM memory ${:06X} to {}",
                    address, value
                );
            }
        }
    }

    fn get_byte(self: &CustomMemory, address: u32) -> u8 {
        panic!("custom memory get_byte: ${:06X}", address);
    }

    fn set_byte(self: &mut CustomMemory, address: u32, value: u8) -> Option<SetMemoryResult> {
        panic!("custom memory set_byte: ${:06X}", address);
    }
}

impl CustomMemory {
    pub fn new() -> CustomMemory {
        CustomMemory {
            color_rgb4: [0x0000; 32],
        }
    }

    pub fn get_start_address(&self) -> u32 {
        0x00DFF000
    }

    pub fn get_end_address(&self) -> u32 {
        0x00DFF1FF
    }

    pub fn get_length(&self) -> usize {
        0x001FF
    }

    pub fn set_color_rgb4(&mut self, color_index: usize, color_rgb4: u16) {
        let mut r = (color_rgb4 >> 8) & 0x000f;
        let mut g = (color_rgb4 >> 4) & 0x000f;
        let mut b = color_rgb4 & 0x000f;

        r = (r << 4) + r;
        g = (g << 4) + g;
        b = (b << 4) + b;
        // let r = 0xff;
        // let b = 0xff;
        println!(
            "   -CUSTOM: Changing color_rgb4 for {} to {:04X} [\x1b[38;2;{};{};{}m\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\x1b[0m]",
            color_index, color_rgb4, r, g, b,
        );
        self.color_rgb4[color_index] = color_rgb4;
    }
}
