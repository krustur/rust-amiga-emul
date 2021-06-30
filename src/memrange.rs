use byteorder::{BigEndian, ReadBytesExt};
use std::convert::TryInto;

pub struct MemRange {
    pub start_address: u32,
    pub end_address: u32,
    pub length: u32,
    pub bytes: Vec<u8>
}

impl MemRange {
    pub fn from_file(start_address: u32, length: u32, filename: &str) -> Result<MemRange, std::io::Error> {
        let bytes = std::fs::read(filename)?;        

        // TODO: Check vec length against incoming size
        let mem = MemRange {
            start_address: start_address,
            end_address: start_address + length - 1,
            length: length,
            bytes: bytes
        };
        Ok(mem)
    }

    fn remap_address_to_index(self: &MemRange, address: u32) -> usize {
        if address < self.start_address || address > self.end_address {
            panic!("Can't remap address to index. Address {:#010x} not in range of {:#010x} to {:#010x}", address, self.start_address, self.end_address)
        }
        let index = address - self.start_address;

        return index.try_into().unwrap();
    }

    pub fn get_longword_unsigned(self: &MemRange, address: u32) -> u32 {        
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index+4];
        let result = bytes.read_u32::<BigEndian>().unwrap();
        // let b0 : u32 = self.bytes[index].into();
        // let b1 : u32 = self.bytes[index + 1].into();
        // let b2 : u32 = self.bytes[index + 2].into();
        // let b3 : u32 = self.bytes[index + 3].into();
        // let result = (b0 << 24) + (b1 << 16) + (b2 << 8) + b3;
        result
    }

    pub fn get_word_unsigned(self: &MemRange, address: u32) -> u16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index+2];
        let result = bytes.read_u16::<BigEndian>().unwrap();
        // let b0 : u16 = self.bytes[index].into();
        // let b1 : u16 = self.bytes[index + 1].into();
        // let result = (b0 << 8) + b1;
        result
    }

    pub fn get_word_signed(self: &MemRange, address: u32) -> i16 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index+2];
        let result = bytes.read_i16::<BigEndian>().unwrap();
        result
    }

    pub fn get_byte_unsigned(self: &MemRange, address: u32) -> u8 {
        let index = self.remap_address_to_index(address);
        let mut bytes = &self.bytes[index..index+1];
        let result = bytes.read_u8().unwrap();
        result
    }

}