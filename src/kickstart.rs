use std::cell::RefCell;
use std::rc::Rc;
use crate::mem::Mem;
use crate::mem::rommemory::RomMemory;

pub struct Kickstart {
    rom_memory: Rc<RefCell<RomMemory>>,
}

impl Kickstart {
    pub fn new(file_path: &str, mem: &mut Mem) -> Self {
        let rom_memory = Rc::new(RefCell::new(
            RomMemory::from_file(0xF80000, file_path).unwrap(),
        ));
        let rom_overlay = Rc::new(RefCell::new(
            RomMemory::from_file(0x000000, file_path).unwrap(),
        ));

        // let mut mem = mem.borrow_mut();
        mem.add_range(rom_memory.clone());
        mem.set_overlay(rom_overlay);
        Self { rom_memory }
    }
}

pub trait KickstartDebug {
    fn get_address_comment(&self, pc_address: u32) -> Option<String>;
    fn disable_print_disassembly_for_address(&self, pc_address: u32) -> bool;
    fn should_print_registers_after_step(&self, pc_address: u32) -> bool;
    fn should_dump_memory_after_step(&self, pc_address: u32) -> Option<(u32, u32)>;
    fn should_dump_areg_memory_after_step(&self, pc_address: u32) -> Option<(usize, u32)>;
    fn should_dump_disassembly_after_step(&self, pc_address: u32) -> Option<(u32, u32)>;
}

pub struct NoKickstartDebug {
}

impl NoKickstartDebug {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl KickstartDebug for NoKickstartDebug {
    fn get_address_comment(&self, pc_address: u32) -> Option<String> {
        None
    }

    fn disable_print_disassembly_for_address(&self, pc_address: u32) -> bool {
        false
    }

    fn should_print_registers_after_step(&self, pc_address: u32) -> bool {
        false
    }

    fn should_dump_memory_after_step(&self, pc_address: u32) -> Option<(u32, u32)> {
        None
    }

    fn should_dump_areg_memory_after_step(&self, pc_address: u32) -> Option<(usize, u32)> {
        None
    }

    fn should_dump_disassembly_after_step(&self, pc_address: u32) -> Option<(u32, u32)> {
        None
    }
}