use byteorder::{BigEndian, ReadBytesExt};
use num_traits::ToPrimitive;
use std::convert::TryInto;

pub struct MemRange {
    pub start_address: u32,
    pub end_address: u32,
    pub length: u32,
    pub bytes: Vec<u8>,
}

impl MemRange {
    pub fn from_file(
        start_address: u32,
        length: u32,
        filename: &str,
    ) -> Result<MemRange, std::io::Error> {
        let bytes = std::fs::read(filename)?;

        // TODO: Check vec length against incoming size
        let mem = MemRange {
            start_address: start_address,
            end_address: start_address + length - 1,
            length: length,
            bytes: bytes,
        };
        Ok(mem)
    }

    pub fn from_bytes<'a>(start_address: u32, bytes: Vec<u8>) -> MemRange{
        let length = bytes.len().to_u32().unwrap();
        let mem = MemRange {
            start_address: start_address,
            end_address: start_address + length - 1,
            length: length,
            bytes: bytes,
        };
        mem
    }

    fn remap_address_to_index(self: &MemRange, address: u32) -> usize {
        if address < self.start_address || address > self.end_address {
            panic!("Can't remap address to index. Address {:#010x} not in range of {:#010x} to {:#010x}", address, self.start_address, self.end_address)
        }
        let index = address - self.start_address;

        return index.try_into().unwrap();
    }

    pub fn get_unsigned_long(self: &MemRange, address: u32) -> u32 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 4];
        let result = bytes.read_u32::<BigEndian>().unwrap();
        result
    }

    pub fn get_signed_long(self: &MemRange, address: u32) -> i32 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 4];
        let result = bytes.read_i32::<BigEndian>().unwrap();
        result
    }

    pub fn get_unsigned_word(self: &MemRange, address: u32) -> u16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 2];
        let result = bytes.read_u16::<BigEndian>().unwrap();
        // let b0 : u16 = self.bytes[index].into();
        // let b1 : u16 = self.bytes[index + 1].into();
        // let result = (b0 << 8) + b1;
        result
    }

    pub fn get_signed_word(self: &MemRange, address: u32) -> i16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 2];
        let result = bytes.read_i16::<BigEndian>().unwrap();
        result
    }

    pub fn get_unsigned_byte(self: &MemRange, address: u32) -> u8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 1];
        let result = bytes.read_u8().unwrap();
        result
    }

    pub fn get_signed_byte(self: &MemRange, address: u32) -> i8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index + 1];
        let result = bytes.read_i8().unwrap();
        result
    }
}
