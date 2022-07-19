use std::{
    any::Any,
    convert::TryInto,
    fmt::{self},
};

use byteorder::{BigEndian, ReadBytesExt};

use super::memory::{Memory, SetMemoryResult};

pub struct RamMemory {
    pub start_address: u32,
    pub end_address: u32,
    length: usize,
    bytes: Vec<u8>,
}

impl fmt::Display for RamMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RAM: ${:08X}-${:08X} ({}) bytes)",
            self.start_address, self.end_address, self.length
        )
    }
}

impl Memory for RamMemory {
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

    fn get_long(self: &RamMemory, address: u32) -> u32 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 4];
        let result = bytes.read_u32::<BigEndian>().unwrap();
        result
    }

    fn set_long(self: &mut RamMemory, address: u32, value: u32) {
        let index = self.remap_address_to_index(address);
        self.bytes[index] = ((value >> 24) & 0x000000ff) as u8;
        self.bytes[index + 1] = ((value >> 16) & 0x000000ff) as u8;
        self.bytes[index + 2] = ((value >> 8) & 0x000000ff) as u8;
        self.bytes[index + 3] = ((value) & 0x000000ff) as u8;
    }

    fn get_word(self: &RamMemory, address: u32) -> u16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 2];
        let result = bytes.read_u16::<BigEndian>().unwrap();
        // let b0 : u16 = self.bytes[index].into();
        // let b1 : u16 = self.bytes[index + 1].into();
        // let result = (b0 << 8) + b1;
        result
    }

    fn set_word(self: &mut RamMemory, address: u32, value: u16) {
        let index = self.remap_address_to_index(address);
        self.bytes[index] = ((value >> 8) & 0x000000ff) as u8;
        self.bytes[index + 1] = ((value) & 0x000000ff) as u8;
    }

    fn get_byte(self: &RamMemory, address: u32) -> u8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 1];
        let result = bytes.read_u8().unwrap();
        result
    }

    fn set_byte(self: &mut RamMemory, address: u32, value: u8) -> Option<SetMemoryResult> {
        let index = self.remap_address_to_index(address);
        self.bytes[index] = ((value) & 0x000000ff) as u8;
        None
    }
}

impl RamMemory {
    pub fn from_range<'a>(start_address: u32, end_address: u32) -> RamMemory {
        let length = end_address as usize - start_address as usize + 1;
        let bytes = vec![0; length];
        let mem = RamMemory {
            start_address,
            end_address,
            length,
            bytes,
        };
        mem
    }
    pub fn from_bytes<'a>(start_address: u32, bytes: Vec<u8>) -> RamMemory {
        let length: u32 = bytes.len().try_into().unwrap();
        assert_eq!(true, length > 0);
        let end_address = start_address + length - 1;
        let length = bytes.len();
        let mem = RamMemory {
            start_address,
            end_address,
            length,
            bytes,
        };
        mem
    }

    fn remap_address_to_index(self: &RamMemory, address: u32) -> usize {
        if address < self.start_address || address > self.end_address {
            panic!("Can't remap address to index. Address {:#010x} not in range of {:#010x} to {:#010x}", address, self.start_address, self.end_address)
        }
        let index = address - self.start_address;

        return index.try_into().unwrap();
    }
}
