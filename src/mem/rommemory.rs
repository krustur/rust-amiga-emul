use crate::{cpu::step_log::StepLog, mem::memory::SetMemoryResult};

use super::memory::Memory;
use byteorder::{BigEndian, ReadBytesExt};
use std::{
    any::Any,
    convert::TryInto,
    fmt::{self},
};

pub struct RomMemory {
    pub start_address: u32,
    pub end_address: u32,
    length: usize,
    bytes: Vec<u8>,
}

impl fmt::Display for RomMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ROM: ${:08X}-${:08X} ({}) bytes)",
            self.start_address, self.end_address, self.length
        )
    }
}

impl Memory for RomMemory {
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

    fn get_long(self: &RomMemory, step_log: &mut StepLog, address: u32) -> u32 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 4];
        let result = bytes.read_u32::<BigEndian>().unwrap();
        result
    }

    fn set_long(self: &mut RomMemory, step_log: &mut StepLog, address: u32, value: u32) {
        step_log.add_log(format!("ROM: Trying to set_long: ${:08X}", address));
    }

    fn get_word(self: &RomMemory, step_log: &mut StepLog, address: u32) -> u16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 2];
        let result = bytes.read_u16::<BigEndian>().unwrap();
        // let b0 : u16 = self.bytes[index].into();
        // let b1 : u16 = self.bytes[index + 1].into();
        // let result = (b0 << 8) + b1;
        result
    }

    fn set_word(self: &mut RomMemory, step_log: &mut StepLog, address: u32, value: u16) {
        step_log.add_log(format!("ROM: Trying to set_word: ${:08X}", address));
    }

    fn get_byte(self: &RomMemory, step_log: &mut StepLog, address: u32) -> u8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 1];
        let result = bytes.read_u8().unwrap();
        result
    }

    fn set_byte(
        self: &mut RomMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        step_log.add_log(format!("ROM: Trying to set_byte: ${:08X}", address));
        None
    }
}

impl RomMemory {
    pub fn from_file(start_address: u32, filename: &str) -> Result<RomMemory, std::io::Error> {
        let bytes = std::fs::read(filename)?;
        let length: u32 = bytes.len().try_into().unwrap();
        let end_address = start_address + length - 1;
        let length = bytes.len();
        let mem = RomMemory {
            start_address: start_address,
            end_address,
            length: length,
            bytes: bytes,
        };
        Ok(mem)
    }

    fn remap_address_to_index(self: &RomMemory, address: u32) -> usize {
        if address < self.start_address || address > self.end_address {
            panic!("Can't remap address to index. Address {:#010x} not in range of {:#010x} to {:#010x}", address, self.start_address, self.end_address)
        }
        let index = address - self.start_address;

        return index.try_into().unwrap();
    }
}
