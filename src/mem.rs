use std::any::Any;

use crate::mem::{ciamemory::CiaMemory, unmappedmemory::UnmappedMemory};

use self::{memory::Memory, memrange::MemRange};

pub mod ciamemory;
pub mod memory;
pub mod memrange;
pub mod rammemory;
pub mod rommemory;
pub mod unmappedmemory;

pub struct Mem {
    ranges: Vec<MemRange>,
    default_range: MemRange,
    cia_range: MemRange,
}

impl Mem {
    pub fn new(ranges: Vec<memrange::MemRange>, cia_memory: CiaMemory) -> Mem {
        for (pos, range) in ranges.iter().enumerate() {
            println!("MemRange: {}", range);
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

        let default_range = UnmappedMemory::new(0x00000000, 0xffffffff);
        // let cia_memory = CiaMemory::new();
        let cia_range = MemRange::from_memory(Box::new(cia_memory));

        let overlay = match cia_range.memory.as_any().downcast_ref::<CiaMemory>() {
            Some(s) => s,
            None => panic!("none"),
        };
        // let a = MemRange::from_memory(Box::new(cia_memory));
        // ranges.push(MemRange::from_memory(Box::new(cia_memory)));
        // let x =
        println!("Default range: {}", default_range);
        Mem {
            ranges,
            default_range: MemRange::from_memory(Box::new(default_range)),
            cia_range: cia_range,
        }
    }

    fn get_memory(self: &Mem, address: u32) -> &MemRange {
        let overlay = match self.cia_range.memory.as_any().downcast_ref::<CiaMemory>() {
            Some(s) => s.overlay,
            None => panic!("none"),
        };
        if !overlay {
            println!("overlay: {}", overlay);
        }

        // let overlay = match Any::downcast_ref::<CiaMemory>() {
        //     Some(s) => panic!("some s"),
        //     None => panic!("none"),
        // };
        if address >= self.cia_range.start_address && address <= self.cia_range.end_address {
            println!("CIA accessed");
            return &self.cia_range;
        }
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.start_address && address <= x.end_address);
        match pos {
            None => &self.default_range,
            Some(pos) => &self.ranges[pos],
        }
    }

    fn get_memory_mut(self: &mut Mem, address: u32) -> &mut MemRange {
        if address >= self.cia_range.start_address && address <= self.cia_range.end_address {
            println!("CIA accessed");
            return &mut self.cia_range;
        }
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.start_address && address <= x.end_address);
        match pos {
            None => &mut self.default_range,
            Some(pos) => &mut self.ranges[pos],
        }
    }

    pub fn get_long(self: &Mem, address: u32) -> u32 {
        let range = self.get_memory(address);
        let result = range.memory.get_long(address);
        result
    }

    pub fn set_long(self: &mut Mem, address: u32, value: u32) {
        let range = self.get_memory_mut(address);
        let result = range.memory.set_long(address, value);
    }

    pub fn get_word(self: &Mem, address: u32) -> u16 {
        let range = self.get_memory(address);
        let result = range.memory.get_word(address);
        result
    }

    pub fn set_word(self: &mut Mem, address: u32, value: u16) {
        let range = self.get_memory_mut(address);
        let result = range.memory.set_word(address, value);
    }

    pub fn get_byte(self: &Mem, address: u32) -> u8 {
        let range = self.get_memory(address);
        let result = range.memory.get_byte(address);
        result
    }

    pub fn set_byte(self: &mut Mem, address: u32, value: u8) {
        let range = self.get_memory_mut(address);
        let result = range.memory.set_byte(address, value);
    }

    pub fn print_hex_dump(self: &mut Mem, start_address: u32, end_address: u32) {
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
