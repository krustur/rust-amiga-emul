use crate::cpu::step_log::StepLog;

use super::memory::{Memory, SetMemoryResult};
use std::{
    any::Any,
    fmt::{self},
};

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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_start_address(&self) -> u32 {
        return self.start_address;
    }

    fn get_end_address(&self) -> u32 {
        return self.end_address;
    }

    fn get_length(&self) -> usize {
        return self.length;
    }

    fn get_long(self: &UnmappedMemory, step_log: &mut StepLog, address: u32) -> u32 {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to get_long: ${:08X}",
            address
        ));
        0
    }

    fn set_long(self: &mut UnmappedMemory, step_log: &mut StepLog, address: u32, value: u32) {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to set_long: ${:08X}",
            address
        ));
    }

    fn get_word(self: &UnmappedMemory, step_log: &mut StepLog, address: u32) -> u16 {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to get_word: ${:08X}",
            address
        ));
        0
    }

    fn set_word(self: &mut UnmappedMemory, step_log: &mut StepLog, address: u32, value: u16) {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to set_word: ${:08X}",
            address
        ));
    }

    fn get_byte(self: &UnmappedMemory, step_log: &mut StepLog, address: u32) -> u8 {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to get_byte: ${:08X}",
            address
        ));
        0
    }

    fn set_byte(
        self: &mut UnmappedMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        step_log.add_log_string(format!(
            "   -UNMAPPED: Trying to set_byte: ${:08X}",
            address
        ));
        None
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
