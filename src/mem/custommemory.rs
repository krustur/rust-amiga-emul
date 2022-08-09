use crate::cpu::step_log::StepLog;

use super::memory::{Memory, SetMemoryResult};
use std::{any::Any, fmt};

pub struct CustomMemory {
    pub dmacon: u16, // 096 / 002
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
        panic!("custom memory get_long: ${:06X}", address);
    }

    fn set_long(self: &mut CustomMemory, step_log: &mut StepLog, address: u32, value: u32) {
        panic!("custom memory set_long: ${:06X}", address);
    }

    fn get_word(self: &CustomMemory, step_log: &mut StepLog, address: u32) -> u16 {
        match address {
            0xDFF002 => {
                // DMACONR
                self.read_dmacon_bits(step_log)
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
                0x0000
            }
            0xDFF09A => {
                // INTENA
                0x0000
            }
            0xDFF09C => {
                // INTREQ
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
                step_log.add_log(format!(
                    "CUSTOM: TODO: get_word() for CUSTOM memory ${:06X}",
                    address
                ));
                0x0000
            }
        }
    }

    fn set_word(self: &mut CustomMemory, step_log: &mut StepLog, address: u32, value: u16) {
        match address {
            0xDFF002 => {
                // DMACONR
                ()
            }
            0xDFF01C => {
                // INTENAR
                ()
            }
            0xDFF01E => {
                // INTREQR
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
                step_log.add_log(format!(
                    "CUSTOM: TODO: set_word() for CUSTOM memory ${:06X} to ${:04X} [%{:016b}]",
                    address, value, value
                ));
            }
        }
    }

    fn get_byte(self: &CustomMemory, step_log: &mut StepLog, address: u32) -> u8 {
        panic!("custom memory get_byte: ${:06X}", address);
    }

    fn set_byte(
        self: &mut CustomMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        panic!("custom memory set_byte: ${:06X}", address);
    }
}

impl CustomMemory {
    pub fn new() -> CustomMemory {
        CustomMemory {
            intena: 0x0000,
            intreq: 0x0000,
            dmacon: 0x0000,
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

    pub fn set_color_rgb4(&mut self, step_log: &mut StepLog, color_index: usize, color_rgb4: u16) {
        let mut r = (color_rgb4 >> 8) & 0x000f;
        let mut g = (color_rgb4 >> 4) & 0x000f;
        let mut b = color_rgb4 & 0x000f;

        r = (r << 4) + r;
        g = (g << 4) + g;
        b = (b << 4) + b;
        // let r = 0xff;
        // let b = 0xff;
        step_log.add_log(format!(
            "CUSTOM: Changing COLOR{:02} rgb4  to {:04X} [\x1b[38;2;{};{};{}m\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\u{25A0}\x1b[0m]",
            color_index, color_rgb4, r, g, b,
        ));
        self.color_rgb4[color_index] = color_rgb4;
    }

    pub fn set_dmacon_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let dmacon = self.dmacon | bits;
        step_log.add_log(format!(
            "CUSTOM: Changing DMACON to ${:04X}. [from: ${:04X}, bits set was ${:04X}",
            dmacon, self.dmacon, bits
        ));
        self.dmacon = dmacon;
    }

    pub fn clear_dmacon_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let dmacon = self.dmacon & !bits;
        step_log.add_log(format!(
            "CUSTOM: Changing DMACON to ${:04X}. [from: ${:04X}, bits cleared was ${:04X}",
            dmacon, self.dmacon, bits
        ));
        self.dmacon = dmacon;
    }

    pub fn read_dmacon_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.dmacon & 0x7fff;
        step_log.add_log(format!("CUSTOM: Reading DMACON, returns ${:04X}", result));
        result
    }

    pub fn set_intena_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intena = self.intena | bits;
        step_log.add_log(format!(
            "CUSTOM: Changing INTENA to ${:04X}. [from: ${:04X}, bits set was ${:04X}",
            intena, self.intena, bits
        ));
        self.intena = intena;
    }

    pub fn clear_intena_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intena = self.intena & !bits;
        step_log.add_log(format!(
            "CUSTOM: Changing INTENA to ${:04X}. [from: ${:04X}, bits cleared was ${:04X}",
            intena, self.intena, bits
        ));
        self.intena = intena;
    }

    pub fn read_intena_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.intena & 0x7fff;
        step_log.add_log(format!("CUSTOM: Reading INTENA, returns ${:04X}", result));
        result
    }

    pub fn set_intreq_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intreq = self.intreq | bits;
        step_log.add_log(format!(
            "CUSTOM: Changing INTREQ to ${:04X}. [from: ${:04X}, bits set was ${:04X}",
            intreq, self.intreq, bits
        ));
        self.intreq = intreq;
    }

    pub fn clear_intreq_bits(&mut self, step_log: &mut StepLog, bits: u16) {
        let bits = bits & 0x7fff;
        let intreq = self.intreq & !bits;
        step_log.add_log(format!(
            "CUSTOM: Changing INTREQ to ${:04X}. [from: ${:04X}, bits cleared was ${:04X}",
            intreq, self.intreq, bits
        ));
        self.intreq = intreq;
    }

    pub fn read_intreq_bits(&self, step_log: &mut StepLog) -> u16 {
        let result = self.intreq & 0x7fff;
        step_log.add_log(format!("CUSTOM: Reading INTREQ, returns ${:04X}", result));
        result
    }
}
