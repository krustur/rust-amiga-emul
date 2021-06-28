pub struct MemRange {
    pub start_address: usize,
    pub end_address: usize,
    pub length: usize,
    pub bytes: Vec<u8>
}

impl MemRange {
    pub fn from_file(start_address: usize, length: usize, filename: &str) -> Result<MemRange, std::io::Error> {
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

    fn remap_address_to_index(self: &MemRange, address: usize) -> usize {
        if address < self.start_address || address > self.end_address {
            panic!("Can't remap address to index. Address {:#010x} not in range of {:#010x} to {:#010x}", address, self.start_address, self.end_address)
        }
        let index = address - self.start_address;

        return index;
    }

    pub fn get_longword_unsigned(self: &MemRange, address: usize) -> u32 {
        let index = self.remap_address_to_index(address);
        let b0 : u32 = self.bytes[index].into();
        let b1 : u32 = self.bytes[index + 1].into();
        let b2 : u32 = self.bytes[index + 2].into();
        let b3 : u32 = self.bytes[index + 3].into();
        let result = (b0 << 24) + (b1 << 16) + (b2 << 8) + b3;
        result
    }

    pub fn get_word_unsigned(self: &MemRange, address: usize) -> u16 {
        let index = self.remap_address_to_index(address);
        let b0 : u16 = self.bytes[index].into();
        let b1 : u16 = self.bytes[index + 1].into();
        let result = (b0 << 8) + b1;
        result
    }
}