use crate::cpu::step_log::StepLog;

use super::memory::{Memory, SetMemoryResult};
use std::{any::Any, fmt};

pub struct CiaMemory {
    pub led: bool,
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

    fn get_long(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u32 {
        panic!("cia memory get_long: ${:06X}", address);
    }

    fn set_long(self: &mut CiaMemory, step_log: &mut StepLog, address: u32, value: u32) {
        panic!("cia memory set_long: ${:06X}", address);
    }

    fn get_word(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u16 {
        panic!("cia memory get_word: ${:06X}", address);
    }

    fn set_word(self: &mut CiaMemory, step_log: &mut StepLog, address: u32, value: u16) {
        panic!("cia memory set_word: ${:06X}", address);
    }

    fn get_byte(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u8 {
        match address {
            0xBFE001 => {
                let mut value = 0x00;
                if self.led {
                    value |= 0x02;
                }
                if self.overlay {
                    value |= 0x01;
                }
                // step_log.add_log_sting(format!(
                //     "CIA: TODO: get_byte() for CIA memory ${:06X}",
                //     address
                // ));
                value
            }
            _ => {
                step_log.add_log_string(format!(
                    "CIA: TODO: get_byte() for CIA memory ${:06X}",
                    address
                ));
                0
            }
        }
    }

    fn set_byte(
        self: &mut CiaMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        match address {
            0xBFE001 => {
                // http://amigadev.elowar.com/read/ADCD_2.1/Hardware_Manual_guide/node018F.html
                let pra_fir1 = (value & 0x80) == 0x80;
                let pra_fir0 = (value & 0x40) == 0x40;
                let pra_rdy = (value & 0x20) == 0x20; // DSKRDY - Disk ready (active low)
                let pra_tk0 = (value & 0x10) == 0x10; // DSKTRACK0 - Track zero detect
                let pra_wpro = (value & 0x08) == 0x08; // DSKPROT - Disk is write protected (active low)
                let pra_chng = (value & 0x04) == 0x04; // DSKCHANGE - Disk has been removed from the drive.  The signal goes low whenever a disk is removed.
                let pra_led = (value & 0x02) == 0x02;
                let pra_ovl = (value & 0x01) == 0x01;
                self.set_led(pra_led);
                let overlay_changed = self.set_overlay(pra_ovl);
                if value > 3 {
                    step_log.add_log_string(format!(
                        "CIA: TODO: set_byte() for CIA memory ${:06X} to ${:02X}",
                        address, value
                    ));
                }
                if let Some(overlay) = overlay_changed {
                    Some(SetMemoryResult {
                        set_overlay: Some(overlay),
                    })
                } else {
                    None
                }
            }
            _ => {
                step_log.add_log_string(format!(
                    "CIA: TODO: set_byte() for CIA memory ${:06X} to ${:02X}",
                    address, value
                ));
                None
            }
        }
    }
}

impl CiaMemory {
    pub fn new() -> CiaMemory {
        CiaMemory {
            led: true,
            overlay: true,
        }
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

    fn set_led(self: &mut CiaMemory, led: bool) {
        if self.led != led {
            let on_or_off = match led {
                true => "OFF",
                false => "ON",
            };
            println!("   -CIA: PRA LED changed to {} [{}]", on_or_off, led);
            self.led = led;
        }
    }

    fn set_overlay(self: &mut CiaMemory, ovl: bool) -> Option<bool> {
        if self.overlay != ovl {
            println!("   -CIA: PRA OVL changed to {}", ovl);
            self.overlay = ovl;
            Some(ovl)
        } else {
            None
        }
    }
}
