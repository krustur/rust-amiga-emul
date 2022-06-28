use super::memory::{Memory, SetMemoryResult};
use std::{any::Any, fmt};

pub struct CiaMemory {
    pub overlay: bool,
}

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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_start_address(&self) -> u32 {
        self.get_start_address()
        // return 0x00BF0000;
    }

    fn get_end_address(&self) -> u32 {
        self.get_end_address()
        // return 0x00BFFFFF;
    }

    fn get_length(&self) -> usize {
        self.get_length()
    }

    fn get_long(self: &CiaMemory, address: u32) -> u32 {
        panic!("cia memory get_long: ${:06X}", address);
    }

    fn set_long(self: &mut CiaMemory, address: u32, value: u32) {
        panic!("cia memory set_long: ${:06X}", address);
    }

    fn get_word(self: &CiaMemory, address: u32) -> u16 {
        panic!("cia memory get_word: ${:06X}", address);
    }

    fn set_word(self: &mut CiaMemory, address: u32, value: u16) {
        panic!("cia memory set_word: ${:06X}", address);
    }

    fn get_byte(self: &CiaMemory, address: u32) -> u8 {
        println!("   -CIA: TODO: get_byte() for CIA memory ${:06X}", address);
        0
    }

    fn set_byte(self: &mut CiaMemory, address: u32, value: u8) -> Option<SetMemoryResult> {
        match address {
            0xBFE001 => {
                let pra_fir1 = (value & 0x80) == 0x80;
                let pra_fir0 = (value & 0x40) == 0x40;
                let pra_rdy = (value & 0x20) == 0x20;
                let pra_tk0 = (value & 0x10) == 0x10;
                let pra_wpro = (value & 0x08) == 0x08;
                let pra_chng = (value & 0x04) == 0x04;
                let pra_led = (value & 0x02) == 0x02;
                let pra_ovl = (value & 0x01) == 0x01;
                self.set_overlay(pra_ovl);
                println!(
                    "   -CIA: TODO: set_byte() for CIA memory ${:06X} to {}",
                    address, value
                );
                Some(SetMemoryResult {
                    set_overlay: Some(pra_ovl),
                })
            }
            _ => {
                println!(
                    "   -CIA: TODO: set_byte() for CIA memory ${:06X} to {}",
                    address, value
                );
                None
            }
        }
    }
}

impl CiaMemory {
    pub fn new() -> CiaMemory {
        CiaMemory { overlay: true }
    }

    pub fn get_start_address(&self) -> u32 {
        0x00BF0000
    }

    pub fn get_end_address(&self) -> u32 {
        0x00BFFFFF
    }

    pub fn get_length(&self) -> usize {
        0x10000
    }

    fn set_overlay(self: &mut CiaMemory, ovl: bool) -> Option<bool> {
        if ovl != self.overlay {
            println!("   -CIA: PRA OVL changed to {}", ovl);
            self.overlay = ovl;
            Some(ovl)
        } else {
            None
        }
    }
}
