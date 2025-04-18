use crate::cpu::{step_log::StepLog, Cpu};

use super::memory::{Memory, SetMemoryResult};
use std::{any::Any, fmt};

pub struct CustomMemory {
    pub dmacon: u16, // 096 / 002
    pub vhpos: u32,  // --- / 004-006
    pub intena: u16, // 09A / 01C
    pub intreq: u16, // 09C / 01E
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

    fn get_long(self: &CustomMemory, step_log: &mut StepLog, address: u32) -> u32 {
        let address = Self::remap_memory(address);
        // panic!("custom memory get_long: ${:06X}", address);
        let hi = self.get_word(step_log, address);
        let low = self.get_word(step_log, address + 2);
        let value = Cpu::join_words_to_long(hi, low);
        value
    }

    fn set_long(self: &mut CustomMemory, step_log: &mut StepLog, address: u32, value: u32) {
        let address = Self::remap_memory(address);
        let hi = Cpu::get_word_from_long(value >> 16);
        self.set_word(step_log, address, hi);
        let low = Cpu::get_word_from_long(value);
        self.set_word(step_log, address + 2, low);
    }

    fn get_word(self: &CustomMemory, step_log: &mut StepLog, address: u32) -> u16 {
        let address = Self::remap_memory(address);
        match address {
            0xDFF002 => {
                // DMACONR
                self.read_dmacon_bits(step_log)
            }
            0xDFF004 => {
                // VPOSR
                // step_log.add_log_sting("CUSTOM: TODO: Reading VPOSR".to_string());
                (self.vhpos >> 16) as u16
            }
            0xDFF006 => {
                // VHPOSR
                // step_log.add_log_sting("CUSTOM: TODO: Reading VHPOSR".to_string());
                (self.vhpos & 0x0000ffff) as u16
            }
            0xDFF01C => {
                // INTENAR
                self.read_intena_bits(step_log)
            }
            0xDFF01E => {
                // INTREQR
                self.read_intreq_bits(step_log)
            }
            0xDFF096 => {
                // DMACON
                step_log.add_log_string("CUSTOM: TODO: Reading DMACON, returns $0000".to_string());
                0x0000
            }
            0xDFF09A => {
                // INTENA
                step_log.add_log_string("CUSTOM: TODO: Reading INTENA, returns $0000".to_string());
                0x0000
            }
            0xDFF09C => {
                // INTREQ
                step_log.add_log_string("CUSTOM: TODO: Reading INTREQ, returns $0000".to_string());
                0x0000
            }
            // 0xDFF100 => {
            //     // BPLCON0
            // }
            // 0xDFF180..=0xDFF1Be => {
            //     // COLORxx
            //     let color_index = (address as usize - 0xDFF180) / 2;
            //     self.set_color_rgb4(color_index, value);
            // }
            _ => {
                step_log.add_log_string(format!(
                    "CUSTOM: TODO: get_word() for CUSTOM memory ${:06X}",
                    address
                ));
                0x0000
            }
        }
    }

    fn set_word(self: &mut CustomMemory, step_log: &mut StepLog, address: u32, value: u16) {
        let address = Self::remap_memory(address);
        match address {
            0xDFF002 => {
                // DMACONR
                step_log.add_log_string("CUSTOM: TODO: Writing DMACONR, nothingness".to_string());
                ()
            }
            0xDFF01C => {
                // INTENAR
                step_log.add_log_string("CUSTOM: TODO: Writing INTENAR, nothingness".to_string());
                ()
            }
            0xDFF01E => {
                // INTREQR
                step_log.add_log_string("CUSTOM: TODO: Writing INTREQR, nothingness".to_string());
                ()
            }
            0xDFF096 => {
                // DMACON
                match value & 0x8000 {
                    0x8000 => {
                        self.set_dmacon_bits(step_log, value & 0x7fff);
                    }
                    _ => {
                        self.clear_dmacon_bits(step_log, value & 0x7fff);
                    }
                }
            }
            0xDFF09A => {
                // INTENA
                match value & 0x8000 {
                    0x8000 => {
                        self.set_intena_bits(step_log, value & 0x7fff);
                    }
                    _ => {
                        self.clear_intena_bits(step_log, value & 0x7fff);
                    }
                }
            }
            0xDFF09C => {
                // INTREQ
                match value & 0x8000 {
                    0x8000 => {
                        self.set_intreq_bits(step_log, value & 0x7fff);
                    }
                    _ => {
                        self.clear_intreq_bits(step_log, value & 0x7fff);
                    }
                }
            }
            // 0xDFF100 => {
            //     // BPLCON0
            // }
            0xDFF180..=0xDFF1Be => {
                // COLORxx
                let color_index = (address as usize - 0xDFF180) / 2;
                self.set_color_rgb4(step_log, color_index, value);
            }
            _ => {
                step_log.add_log_string(format!(
                    "CUSTOM: TODO: set_word() for CUSTOM memory ${:06X} to ${:04X} [%{:016b}]",
                    address, value, value
                ));
            }
        }
    }

    fn get_byte(self: &CustomMemory, step_log: &mut StepLog, address: u32) -> u8 {
        let address = Self::remap_memory(address);
        panic!("custom memory get_byte: ${:06X}", address);
    }

    fn set_byte(
        self: &mut CustomMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        let address = Self::remap_memory(address);
        panic!("custom memory set_byte: ${:06X}", address);
    }
}

impl CustomMemory {
    pub fn new() -> CustomMemory {
        CustomMemory {
            dmacon: 0x0000,
            vhpos: 0x00000000,
            intena: 0x0000,
            intreq: 0x0000,
            color_rgb4: [0x0000; 32],
        }
    }

    pub fn is_custom_memory(address: u32) -> bool {
        match address {
            0x00c00000..=0x00deffff => true,
            0x00dff000..=0x00dfffff => true,
            _ => false,
        }
    }

    fn remap_memory(address: u32) -> u32 {
        let remapped = 0x00dff000 + (address & 0x000001ff);
        if remapped != address {
            println!("                                                            ; Remapping CUSTOM memory from ${:06X} to ${:06X}", address, remapped);
        }
        remapped
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

    pub fn set_color_rgb4(&mut self, step_log: &mut StepLog, color_index: usize, color_rgb4: u16) {
        let mut r = (color_rgb4 >> 8) & 0x000f;
        let mut g = (color_rgb4 >> 4) & 0x000f;
        let mut b = color_rgb4 & 0x000f;

        r = (r << 4) + r;
        g = (g << 4) + g;
        b = (b << 4) + b;
        // let r = 0xff;
        // let b = 0xff;
        step_log.add_log_string(format!(
            "CUSTOM: Changing COLOR{:02} rgb4  to {:04X} [\x1b[38;2;{};{};{}m\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\x1b[0m]",
            color_index, color_rgb4, r, g, b,
        ));
        self.color_rgb4[color_index] = color_rgb4;
    }

    pub fn set_dmacon_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let dmacon = self.dmacon | bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing DMACON to ${:04X}. [from: ${:04X}, bits set was ${:04X}]",
            dmacon, self.dmacon, bits
        ));
        self.dmacon = dmacon;
    }

    pub fn clear_dmacon_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let dmacon = self.dmacon & !bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing DMACON to ${:04X}. [from: ${:04X}, bits cleared was ${:04X}]",
            dmacon, self.dmacon, bits
        ));
        self.dmacon = dmacon;
    }

    pub fn read_dmacon_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.dmacon & 0x7fff;
        step_log.add_log_string(format!("CUSTOM: Reading DMACONR, returns ${:04X}", result));
        result
    }

    pub fn set_intena_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intena = self.intena | bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing INTENA to ${:04X}. [{}] [from: ${:04X}, bits set was ${:04X}]",
            intena,
            Self::bit_field_to_string(intena, Self::intena_bit_names()),
            self.intena,
            bits
        ));
        self.intena = intena;
    }

    pub fn clear_intena_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intena = self.intena & !bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing INTENA to ${:04X}. [{}] [from: ${:04X}, bits cleared was ${:04X}]",
            intena,
            Self::bit_field_to_string(intena, Self::intena_bit_names()),
            self.intena,
            bits
        ));
        self.intena = intena;
    }

    pub fn read_intena_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.intena & 0x7fff;
        step_log.add_log_string(format!(
            "CUSTOM: Reading INTENAR, returns ${:04X} [{}]",
            result,
            Self::bit_field_to_string(result, Self::intena_bit_names())
        ));
        result
    }

    pub fn set_intreq_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intreq = self.intreq | bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing INTREQ to ${:04X}. [from: ${:04X}, bits set was ${:04X}",
            intreq, self.intreq, bits
        ));
        self.intreq = intreq;
    }

    pub fn clear_intreq_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intreq = self.intreq & !bits;
        step_log.add_log_string(format!(
            "CUSTOM: Changing INTREQ to ${:04X}. [from: ${:04X}, bits cleared was ${:04X}",
            intreq, self.intreq, bits
        ));
        self.intreq = intreq;
    }

    pub fn read_intreq_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.intreq & 0x7fff;
        step_log.add_log_string(format!("CUSTOM: Reading INTREQR, returns ${:04X}", result));
        result
    }


    // 14    INTEN       Master interrupt (enable only,
    //                                     no request)
    // 13    EXTER   6   External interrupt
    // 12    DSKSYN  5   Disk sync register ( DSKSYNC )
    // matches disk data
    // 11    RBF     5   Serial port receive buffer full
    // 10    AUD3    4   Audio channel 3 block finished
    // 09    AUD2    4   Audio channel 2 block finished
    // 08    AUD1    4   Audio channel 1 block finished
    // 07    AUD0    4   Audio channel 0 block finished
    // 06    BLIT    3   Blitter finished
    // 05    VERTB   3   Start of vertical blank
    // 04    COPER   3   Copper
    // 03    PORTS   2   I/O ports and timers
    // 02    SOFT    1   Reserved for software-initiated
    // interrupt
    // 01    DSKBLK  1   Disk block finished
    // 00    TBE     1   Serial port transmit buffer empty
    fn intena_bit_names() -> &'static [&'static str] {
        static RESULT: [&str; 16] = [
            "TBE", "DSKBLK", "SOFT", "PORTS", "COPER",
            "VERTB", "BLIT", "AUD0", "AUD1", "AUD2",
            "AUD3", "RBF", "DSKSYN", "EXTER", "INTEN", "SET/CLR",
        ];
        &RESULT
    }

    // TODO: Move elsewhere
    fn bit_field_to_string(bit_field: u16, bit_names: &[&str]) -> String {
        bit_names
            .iter()
            .enumerate()
            .filter_map(|(i, &name)| {
                if bit_field & (1 << i) != 0 {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}
