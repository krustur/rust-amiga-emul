use crate::memrange;

pub struct Mem {
    ranges: Vec<memrange::MemRange>,
}

impl Mem {
    pub fn new(ranges: Vec<memrange::MemRange>) -> Mem {
        for (pos, range) in ranges.iter().enumerate() {
            // println!("MemRange: ${:08X}-${:08X}", range.start_address, range.end_address);
            for (other_pos, other_range) in ranges.iter().enumerate() {
                if pos != other_pos {
                    // println!("Checking {} with {}", pos, other_pos);
                    //        x----------------------x          <-- other
                    //                     x--------------x     <-- other_range (A)
                    //   x------------x                         <-- other_range (B)
                    //     x-----------------------------x      <-- other_range (C)
                    // (A):
                    if (range.start_address >= other_range.start_address && range.start_address <= other_range.end_address) ||
                    // (B):
                        (range.end_address <= other_range.end_address && range.end_address >= other_range.start_address) ||
                    // (C):
                        (range.start_address <= other_range.start_address && range.end_address >= other_range.end_address) 
                    {
                        panic!("Found overlapping MemRanges:\n ${:08X}-${:08X}\n ${:08X}-${:08X}", range.start_address, range.end_address, other_range.start_address, other_range.end_address);
                    }
                }
            }
        }
        Mem { ranges: ranges }
    }

    fn get_range(self: &Mem, address: u32) -> &memrange::MemRange {
        // TODO: How to handle addresses not in Ranges?
        // TODO: How to handle custom regs etc.?
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.start_address && address <= x.end_address);
        let pos = match pos {
            None => panic!("Could not find MemRange for address: {:010x}", address),
            Some(pos) => pos,
        };
        &self.ranges[pos]
    }

    fn get_range_mut(self: &mut Mem, address: u32) -> &mut memrange::MemRange {
        // TODO: How to handle addresses not in Ranges?
        // TODO: How to handle custom regs etc.?
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.start_address && address <= x.end_address);
        let pos = match pos {
            None => panic!("Could not find MemRange for address: {:010x}", address),
            Some(pos) => pos,
        };
        &mut self.ranges[pos]
    }

    /// Get's a unsigned long (u32) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_long(self: &Mem, address: u32) -> u32 {
        let range = self.get_range(address);
        let result = range.get_unsigned_long(address);
        result
    }

    /// Get's a signed long (i32) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_long(self: &Mem, address: u32) -> i32 {
        let range = self.get_range(address);
        let result = range.get_signed_long(address);
        result
    }

    pub fn set_unsigned_long(self: &mut Mem, address: u32, value: u32) {
        let range = self.get_range_mut(address);
        let result = range.set_unsigned_long(address, value);
    }

    /// Get's a unsigned word (u16) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_word(self: &Mem, address: u32) -> u16 {
        let range = self.get_range(address);
        let result = range.get_unsigned_word(address);
        result
    }

    /// Get's a signed word (i16) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_word(self: &Mem, address: u32) -> i16 {
        let range = self.get_range(address);
        let result = range.get_signed_word(address);
        result
    }

    pub fn set_unsigned_word(self: &mut Mem, address: u32, value: u16) {
        let range = self.get_range_mut(address);
        let result = range.set_unsigned_word(address, value);
    }

    /// Get's a unsigned byte (u8) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_byte(self: &Mem, address: u32) -> u8 {
        let range = self.get_range(address);
        let result = range.get_unsigned_byte(address);
        result
    }

    /// Get's a signed byte (i8) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_byte(self: &Mem, address: u32) -> i8 {
        let range = self.get_range(address);
        let result = range.get_signed_byte(address);
        result
    }

    pub fn set_unsigned_byte(self: &mut Mem, address: u32, value: u8) {
        let range = self.get_range_mut(address);
        let result = range.set_unsigned_byte(address, value);
    }

    /// Prints out content of memory in given range as hex dump
    ///
    /// # Arguments
    ///
    /// * `start_address` - Start address of memory to print
    /// * `end_address` - End address of memory to print
    pub fn print_range(self: &mut Mem, start_address: u32, end_address: u32) {
        let mut col_cnt = 0;
        let mut address = start_address;
        while address <= end_address {
            if col_cnt == 0 {
                print!(" {:010x}", address);
            }
            print!(" {:02x}", self.get_unsigned_byte(address));
            col_cnt = col_cnt + 1;
            if col_cnt == 16 {
                col_cnt = 0;
                println!();
            }
            address = address + 1;
        }

        if col_cnt != 0 {
            println!();
        }
    }
}
