use self::memory::Memory;
use crate::{
    cpu::step_log::{StepLog, StepLogEntry},
    mem::unmappedmemory::UnmappedMemory,
};
use std::cell::RefCell;
use std::rc::Rc;

pub mod ciamemory;
pub mod custommemory;
pub mod memory;
pub mod rammemory;
pub mod rommemory;
pub mod unmappedmemory;

pub struct Mem {
    ranges: Vec<Rc<RefCell<dyn Memory>>>,
    default_range: Rc<RefCell<dyn Memory>>,
    overlay_memory: Rc<RefCell<dyn Memory>>,
    pub overlay: bool,
}

impl Mem {
    pub fn new() -> Self {
        let ranges: Vec<Rc<RefCell<dyn Memory>>> = Vec::new();
        let default_range = Rc::new(RefCell::new(UnmappedMemory::new(0x00000000, 0xffffffff)));
        let overlay_memory = Rc::new(RefCell::new(UnmappedMemory::new(0x00000000, 0x00000000)));
        Self {
            ranges,
            default_range,
            overlay_memory,
            overlay: false,
        }
    }

    pub fn add_range(&mut self, range: Rc<RefCell<dyn Memory>>) {
        self.ranges.push(range);
        self.validate_ranges();
    }

    pub fn set_overlay(&mut self, range: Rc<RefCell<dyn Memory>>) {
        self.overlay_memory = range;
        self.overlay = true;
        self.validate_ranges();
    }

    fn validate_ranges(&self) {
        for (pos, range) in self.ranges.iter().enumerate() {
            let range = range.borrow();
            // println!("MemRange: {}", range);
            for (other_pos, other_range) in self.ranges.iter().enumerate() {
                let other_range = other_range.borrow();
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
    }

    fn get_memory(self: &Mem, address: u32) -> Rc<RefCell<dyn Memory>> {
        if self.overlay
            && address >= self.overlay_memory.borrow().get_start_address()
            && address <= self.overlay_memory.borrow().get_end_address()
        {
            println!("OVERLAY access!");
            return self.overlay_memory.clone();
        }
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
        let pos = self.ranges.iter().position(|x| {
            address >= x.borrow().get_start_address() && address <= x.borrow().get_end_address()
        });
        match pos {
            None => self.default_range.clone(),
            Some(pos) => self.ranges[pos].clone(),
        }
    }

    fn get_memory_mut(self: &mut Mem, address: u32) -> Rc<RefCell<dyn Memory>> {
        if self.overlay
            && address >= self.overlay_memory.borrow().get_start_address()
            && address <= self.overlay_memory.borrow().get_end_address()
        {
            println!("OVERLAY access!");
            return self.overlay_memory.clone();
        }
        // if address >= self.cia_range.get_start_address()
        //     && address <= self.cia_range.get_end_address()
        // {
        //     println!("CIA accessed");
        //     return &mut self.cia_range;
        // }
        let pos = self.ranges.iter().position(|x| {
            address >= x.borrow().get_start_address() && address <= x.borrow().get_end_address()
        });
        match pos {
            None => self.default_range.clone(),
            Some(pos) => self.ranges[pos].clone(),
        }
    }

    pub fn get_long(self: &Mem, step_log: &mut StepLog, address: u32) -> u32 {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory(address);
        let result = range.borrow().get_long(step_log, address);
        step_log.add_step_log_entry(StepLogEntry::ReadMemLong {
            address,
            value: result,
        });
        result
    }

    pub fn get_long_no_log(self: &Mem, address: u32) -> u32 {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory(address);
        let result = range.borrow().get_long(&mut StepLog::new(), address);
        result
    }

    pub fn set_long(self: &mut Mem, step_log: &mut StepLog, address: u32, value: u32) {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory_mut(address);
        step_log.add_step_log_entry(StepLogEntry::WriteMemLong { address, value });
        let result = range.borrow_mut().set_long(step_log, address, value);
    }

    pub fn set_long_no_log(self: &mut Mem, address: u32, value: u32) {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory_mut(address);
        let result = range
            .borrow_mut()
            .set_long(&mut StepLog::new(), address, value);
    }

    pub fn get_word(self: &Mem, step_log: &mut StepLog, address: u32) -> u16 {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory(address);
        let result = range.borrow().get_word(step_log, address);
        step_log.add_step_log_entry(StepLogEntry::ReadMemWord {
            address,
            value: result,
        });
        result
    }

    pub fn get_word_no_log(self: &Mem, address: u32) -> u16 {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory(address);
        let result = range.borrow().get_word(&mut StepLog::new(), address);
        result
    }

    pub fn set_word(self: &mut Mem, step_log: &mut StepLog, address: u32, value: u16) {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory_mut(address);
        step_log.add_step_log_entry(StepLogEntry::WriteMemWord { address, value });
        let result = range.borrow_mut().set_word(step_log, address, value);
    }

    pub fn set_word_no_log(self: &mut Mem, address: u32, value: u16) {
        if (address & 0x00000001) != 0 {
            panic!();
        }
        let range = self.get_memory_mut(address);
        let result = range
            .borrow_mut()
            .set_word(&mut StepLog::new(), address, value);
    }

    pub fn get_byte(self: &Mem, step_log: &mut StepLog, address: u32) -> u8 {
        let range = self.get_memory(address);
        let result = range.borrow().get_byte(step_log, address);
        step_log.add_step_log_entry(StepLogEntry::ReadMemByte {
            address,
            value: result,
        });
        result
    }

    pub fn get_byte_no_log(self: &Mem, address: u32) -> u8 {
        let range = self.get_memory(address);
        let result = range.borrow().get_byte(&mut StepLog::new(), address);
        result
    }

    pub fn set_byte(self: &mut Mem, step_log: &mut StepLog, address: u32, value: u8) {
        let range = self.get_memory_mut(address);
        step_log.add_step_log_entry(StepLogEntry::WriteMemByte { address, value });
        let set_byte_result = range
            .borrow_mut()
            .set_byte(step_log, address, value);
        match set_byte_result {
            Some(r) => {
                if let Some(overlay) = r.set_overlay {
                    self.overlay = overlay;
                }
            }
            None => (),
        }
    }

    pub fn set_byte_no_log(self: &mut Mem, address: u32, value: u8) {
        let range = self.get_memory_mut(address);
        let set_byte_result = range
            .borrow_mut()
            .set_byte(&mut StepLog::new(), address, value);
        match set_byte_result {
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
                print!(" ${:08X}", address);
            }
            print!(" {:02x}", self.get_byte_no_log(address));
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
