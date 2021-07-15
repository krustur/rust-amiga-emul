use crate::memrange;
use std::convert::TryInto;

pub struct Mem<'a> {
    ranges: Vec<&'a memrange::MemRange>,
}

impl<'a> Mem<'a> {
    pub fn new(ranges: Vec<&'a memrange::MemRange>) -> Mem<'a> {
        Mem { ranges: ranges }
    }

    fn get_range(self: &Mem<'a>, address: u32) -> &memrange::MemRange {
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
        self.ranges[pos]
    }

    /// Get's a unsigned longword (u32) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_longword(self: &Mem<'a>, address: u32) -> u32 {
        let range = self.get_range(address);
        let result = range.get_unsigned_longword(address);
        result
    } 

    /// Get's a signed longword (i32) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_longword(self: &Mem<'a>, address: u32) -> i32 {
        let range = self.get_range(address);
        let result = range.get_signed_longword(address);
        result
    }

    /// Get's a unsigned word (u16) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_word(self: &Mem<'a>, address: u32) -> u16 {
        let range = self.get_range(address);
        let result = range.get_unsigned_word(address);
        result
    }
    /// Get's a signed word (i16) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_word(self: &Mem<'a>, address: u32) -> i16 {
        let range = self.get_range(address);
        let result = range.get_signed_word(address);
        result
    }

    /// Get's a unsigned byte (u8) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_unsigned_byte(self: &Mem<'a>, address: u32) -> u8 {
        let range = self.get_range(address);
        let result = range.get_unsigned_byte(address);
        result
    }

    /// Get's a signed byte (i8) from specified memory address
    ///
    /// # Arguments
    ///
    /// * `address` - Memory address as u32
    pub fn get_signed_byte(self: &Mem<'a>, address: u32) -> i8 {
        let range = self.get_range(address);
        let result = range.get_signed_byte(address);
        result
    }
    /// Prints out content of memory in given range as hex dump
    ///
    /// # Arguments
    ///
    /// * `start_address` - Start address of memory to print
    /// * `end_address` - End address of memory to print
    pub fn print_range(self: &Mem<'a>, start_address: u32, end_address: u32) {
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
