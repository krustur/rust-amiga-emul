use std::{
    convert::TryInto,
    fmt::{self, Display},
};

use byteorder::{BigEndian, ReadBytesExt};

// trait ValRequireTrait<T: ValTrait = Self>: ValTrait<T> {}

pub trait Memory: Display {
    // pub trait Memory {
    fn get_start_address(&self) -> u32;
    fn get_end_address(&self) -> u32;
    fn get_length(&self) -> usize;

    fn get_long(&self, address: u32) -> u32;
    fn set_long(&mut self, address: u32, value: u32);
    fn get_word(&self, address: u32) -> u16;
    fn set_word(&mut self, address: u32, value: u16);
    fn get_byte(&self, address: u32) -> u8;
    fn set_byte(&mut self, address: u32, value: u8);
}

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

impl Memory for RamMemory {
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

    fn set_byte(self: &mut RamMemory, address: u32, value: u8) {
        let index = self.remap_address_to_index(address);
        self.bytes[index] = ((value) & 0x000000ff) as u8;
    }
}

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
            "RAM: ${:08X}-${:08X} ({}) bytes)",
            self.start_address, self.end_address, self.length
        )
    }
}

impl RomMemory {
    pub fn from_file(start_address: u32, filename: &str) -> Result<RomMemory, std::io::Error> {
        let bytes = std::fs::read(filename)?;
        let length: u32 = bytes.len().try_into().unwrap();
        let end_address = start_address + length - 1;
        let length = bytes.len();
        // TODO: Check vec length against incoming size
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

impl Memory for RomMemory {
    fn get_start_address(&self) -> u32 {
        return self.start_address;
    }

    fn get_end_address(&self) -> u32 {
        return self.end_address;
    }

    fn get_length(&self) -> usize {
        return self.length;
    }

    fn get_long(self: &RomMemory, address: u32) -> u32 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 4];
        let result = bytes.read_u32::<BigEndian>().unwrap();
        result
    }

    fn set_long(self: &mut RomMemory, address: u32, value: u32) {
        eprintln!("Trying to set_long on ROM memory: ${:08X}", address);
    }

    fn get_word(self: &RomMemory, address: u32) -> u16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 2];
        let result = bytes.read_u16::<BigEndian>().unwrap();
        // let b0 : u16 = self.bytes[index].into();
        // let b1 : u16 = self.bytes[index + 1].into();
        // let result = (b0 << 8) + b1;
        result
    }

    fn set_word(self: &mut RomMemory, address: u32, value: u16) {
        eprintln!("Trying to set_word on ROM memory: ${:08X}", address);
    }

    fn get_byte(self: &RomMemory, address: u32) -> u8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 1];
        let result = bytes.read_u8().unwrap();
        result
    }

    fn set_byte(self: &mut RomMemory, address: u32, value: u8) {
        eprintln!("Trying to set_byte on ROM memory: ${:08X}", address);
    }
}

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
        eprintln!("Trying to get_long on UNMAPPED memory: ${:08X}", address);
        0
    }

    fn set_long(self: &mut UnmappedMemory, address: u32, value: u32) {
        eprintln!("Trying to set_long on UNMAPPED memory: ${:08X}", address);
    }

    fn get_word(self: &UnmappedMemory, address: u32) -> u16 {
        eprintln!("Trying to get_word on UNMAPPED memory: ${:08X}", address);
        0
    }

    fn set_word(self: &mut UnmappedMemory, address: u32, value: u16) {
        eprintln!("Trying to set_word on UNMAPPED memory: ${:08X}", address);
    }

    fn get_byte(self: &UnmappedMemory, address: u32) -> u8 {
        eprintln!("Trying to get_byte on UNMAPPED memory: ${:08X}", address);
        0
    }

    fn set_byte(self: &mut UnmappedMemory, address: u32, value: u8) {
        eprintln!("Trying to set_byte on UNMAPPED memory: ${:08X}", address);
    }
}
