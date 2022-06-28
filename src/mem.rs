use crate::mem::{ciamemory::CiaMemory, unmappedmemory::UnmappedMemory};

use self::memory::Memory;

pub mod ciamemory;
pub mod memory;
pub mod rammemory;
pub mod rommemory;
pub mod unmappedmemory;

pub struct Mem {
    ranges: Vec<Box<dyn Memory>>,
    default_range: Box<dyn Memory>,
    overlay: bool,
}

impl Mem {
    pub fn new(ranges: Vec<Box<dyn Memory>>) -> Mem {
        for (pos, range) in ranges.iter().enumerate() {
            println!("MemRange: {}", range);
            for (other_pos, other_range) in ranges.iter().enumerate() {
                if pos != other_pos {
                    // println!("Checking {} with {}", pos, other_pos);

                    //        x----------------------x          <-- range
                    //                     x--------------x     <-- other_range (A)
                    //   x------------x                         <-- other_range (B)
                    //     x-----------------------------x      <-- other_range (C)

                    // (A):
                    if (range.get_start_address() >= other_range.get_start_address() && range.get_start_address() <= other_range.get_end_address()) ||
                    // (B):
                        (range.get_end_address() <= other_range.get_end_address() && range.get_end_address() >= other_range.get_start_address()) ||
                    // (C):
                        (range.get_start_address() <= other_range.get_start_address() && range.get_end_address() >= other_range.get_end_address())
                    {
                        panic!(
                            "Found overlapping MemRanges:\n ${:08X}-${:08X}\n ${:08X}-${:08X}",
                            range.get_start_address(),
                            range.get_end_address(),
                            other_range.get_start_address(),
                            other_range.get_end_address()
                        );
                    }
                }
            }
        }

        let default_range = UnmappedMemory::new(0x00000000, 0xffffffff);

        println!("Default range: {}", default_range);
        Mem {
            ranges,
            default_range: Box::new(default_range),
            overlay: true,
            // cia_range: cia_range,
        }
    }

    fn get_memory(self: &Mem, address: u32) -> &Box<dyn Memory> {
        // let overlay = match self.cia_range.as_any().downcast_ref::<CiaMemory>() {
        //     Some(s) => s.overlay,
        //     None => panic!("none"),
        // };
        // if !overlay {
        //     println!("overlay: {}", overlay);
        // }

        // let overlay = match Any::downcast_ref::<CiaMemory>() {
        //     Some(s) => panic!("some s"),
        //     None => panic!("none"),
        // };

        // if address >= self.cia_range.get_start_address()
        //     && address <= self.cia_range.get_end_address()
        // {
        //     println!("CIA accessed");
        //     return &self.cia_range;
        // }
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.get_start_address() && address <= x.get_end_address());
        match pos {
            None => &self.default_range,
            Some(pos) => &self.ranges[pos],
        }
    }

    fn get_memory_mut(self: &mut Mem, address: u32) -> &mut Box<dyn Memory> {
        // if address >= self.cia_range.get_start_address()
        //     && address <= self.cia_range.get_end_address()
        // {
        //     println!("CIA accessed");
        //     return &mut self.cia_range;
        // }
        let pos = self
            .ranges
            .iter()
            .position(|x| address >= x.get_start_address() && address <= x.get_end_address());
        match pos {
            None => &mut self.default_range,
            Some(pos) => &mut self.ranges[pos],
        }
    }

    pub fn get_long(self: &Mem, address: u32) -> u32 {
        let range = self.get_memory(address);
        let result = range.get_long(address);
        result
    }

    pub fn set_long(self: &mut Mem, address: u32, value: u32) {
        let range = self.get_memory_mut(address);
        let result = range.set_long(address, value);
    }

    pub fn get_word(self: &Mem, address: u32) -> u16 {
        let range = self.get_memory(address);
        let result = range.get_word(address);
        result
    }

    pub fn set_word(self: &mut Mem, address: u32, value: u16) {
        let range = self.get_memory_mut(address);
        let result = range.set_word(address, value);
    }

    pub fn get_byte(self: &Mem, address: u32) -> u8 {
        let range = self.get_memory(address);
        let result = range.get_byte(address);
        result
    }

    pub fn set_byte(self: &mut Mem, address: u32, value: u8) {
        let range = self.get_memory_mut(address);
        match range.set_byte(address, value) {
            Some(r) => {
                if let Some(overlay) = r.set_overlay {
                    self.overlay = overlay;
                }
            }
            None => (),
        }
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
