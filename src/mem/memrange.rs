use std::fmt;

use super::memory::Memory;

pub struct MemRange {
    pub start_address: u32,
    pub end_address: u32,
    pub length: usize,
    pub memory: Box<dyn Memory>,
}

impl fmt::Display for MemRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.memory)
    }
}

impl MemRange {
    pub fn from_memory(memory: Box<dyn Memory>) -> MemRange {
        let start_address = memory.get_start_address();
        let end_address = memory.get_end_address();
        let length = memory.get_length();
        MemRange {
            start_address,
            end_address,
            length,
            memory,
        }
    }
}
