use crate::memrange;

// #[derive(Copy, Clone)]


pub struct Mem<'a> {
    ranges: Vec<&'a memrange::MemRange>
}

impl<'a> Mem<'a> {
    pub fn new(ranges: Vec<&'a memrange::MemRange>) -> Mem<'a> {
        Mem{
            ranges: ranges
        }
    }


    fn get_range(self: &Mem<'a>, address: usize) -> &memrange::MemRange {
        // TODO: How to handle addresses not in Ranges?
        // TODO: How to handle custom regs etc.?
        let pos = self.ranges.iter().position(|x| address >= x.start_address && address <= x.end_address);
        let pos = match pos {
            None => panic!("Could not find MemRange for address: {:010x}", address),
            Some(pos) => pos
        };
        self.ranges[pos]
    }

    pub fn get_longword_unsigned(self: &Mem<'a>, address: usize) -> u32 {
        let range = self.get_range(address);
        let result = range.get_longword_unsigned(address);
        result
    }

    pub fn get_word_unsigned(self: &Mem<'a>, address: usize) -> u16 {
        let range = self.get_range(address);
        let result = range.get_word_unsigned(address);
        result
    }

    // pub fn get_word_signed(self: &Mem<'a>, address: usize) -> i16 {
    //     let range = self.get_range(address);
    //     let result = range.get_word_unsigned(address);
    //     result
    // }
}

