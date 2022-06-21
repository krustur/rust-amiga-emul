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
                        panic!(
                            "Found overlapping MemRanges:\n ${:08X}-${:08X}\n ${:08X}-${:08X}",
                            range.start_address,
                            range.end_address,
                            other_range.start_address,
                            other_range.end_address
                        );
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
            None => panic!("Could not find MemRange for address: ${:08X}", address),
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
            None => panic!("Could not find MemRange for address: ${:08X}", address),
            Some(pos) => pos,
        };
        &mut self.ranges[pos]
    }

    pub fn get_long(self: &Mem, address: u32) -> u32 {
        let range = self.get_range(address);
        let result = range.get_long(address);
        result
    }

    pub fn set_long(self: &mut Mem, address: u32, value: u32) {
        let range = self.get_range_mut(address);
        let result = range.set_long(address, value);
    }

    pub fn get_word(self: &Mem, address: u32) -> u16 {
        let range = self.get_range(address);
        let result = range.get_word(address);
        result
    }

    pub fn set_word(self: &mut Mem, address: u32, value: u16) {
        let range = self.get_range_mut(address);
        let result = range.set_word(address, value);
    }

    pub fn get_byte(self: &Mem, address: u32) -> u8 {
        let range = self.get_range(address);
        let result = range.get_byte(address);
        result
    }

    pub fn set_byte(self: &mut Mem, address: u32, value: u8) {
        let range = self.get_range_mut(address);
        let result = range.set_byte(address, value);
    }

    pub fn print_range(self: &mut Mem, start_address: u32, end_address: u32) {
        let mut col_cnt = 0;
        let mut address = start_address;
        while address <= end_address {
            if col_cnt == 0 {
                print!(" {:010x}", address);
            }
            print!(" {:02x}", self.get_byte(address));
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
