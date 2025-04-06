pub trait Kickstart {
    fn get_comment(&self, pc_address: u32) -> Option<String>;
    fn get_no_print_disassembly_before_step(&self, pc_address: u32) -> bool;
    fn get_print_registers_after_step(&self, pc_address: u32) -> bool;
    fn get_dump_memory_after_step(&self, pc_address: u32) -> Option<(u32, u32)>;
    fn get_dump_areg_memory_after_step(&self, pc_address: u32) -> Option<(usize, u32)>;
    fn get_print_disassembly_after_step(&self, pc_address: u32) -> Option<(u32, u32)>;
}
