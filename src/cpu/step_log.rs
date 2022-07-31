use std::fmt::Display;

use crate::register::{RegisterType, StatusRegister};

#[derive(Copy, Clone)]
pub enum StepLogEntry {
    Null,
    SrReadSupervisorBit {
        value: bool,
    },
    SrChanged {
        value: u16,
        value_old: u16,
    },
    SrNotChanged {
        value: u16,
    },
    ReadRegister {
        reg_type: RegisterType,
        reg_index: usize,
        value: u32,
    },
    WriteRegister {
        reg_type: RegisterType,
        reg_index: usize,
        value: u32,
    },
    ReadMemLong {
        address: u32,
        value: u32,
    },
    ReadMemWord {
        address: u32,
        value: u16,
    },
    ReadMemByte {
        address: u32,
        value: u8,
    },
    WriteMemLong {
        address: u32,
        value: u32,
    },
    WriteMemWord {
        address: u32,
        value: u16,
    },
    WriteMemByte {
        address: u32,
        value: u8,
    },
}

impl Display for StepLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepLogEntry::Null => write!(f, "NULL"),
            StepLogEntry::SrReadSupervisorBit { value } => write!(f, "get_S:{}", value),
            StepLogEntry::SrChanged { value, value_old } => {
                let new_sr = StatusRegister::from_word(*value);
                let old_sr = StatusRegister::from_word(*value_old);
                write!(
                    f,
                    "change_sr=${:04X} {} [was ${:04X} {}]",
                    value, new_sr, value_old, old_sr
                )
            }
            StepLogEntry::SrNotChanged { value } => {
                let new_sr = StatusRegister::from_word(*value);
                write!(f, "not_change_sr=${:04X} {}", value, new_sr)
            }
            StepLogEntry::ReadRegister {
                reg_type: register_type,
                reg_index: register_index,
                value,
            } => write!(
                f,
                "get_reg {}{}=${:08X}",
                register_type.get_format(),
                register_index,
                value
            ),
            StepLogEntry::WriteRegister {
                reg_type: register_type,
                reg_index: register_index,
                value,
            } => write!(
                f,
                "write_reg {}{}=${:08X}",
                register_type.get_format(),
                register_index,
                value
            ),
            StepLogEntry::ReadMemLong { address, value } => {
                write!(f, "get_mem.l (${:08X})=${:08X}", address, value)
            }
            StepLogEntry::ReadMemWord { address, value } => {
                write!(f, "get_mem.w (${:08X})=${:04X}", address, value)
            }
            StepLogEntry::ReadMemByte { address, value } => {
                write!(f, "get_mem.b (${:08X})=${:02X}", address, value)
            }
            StepLogEntry::WriteMemLong { address, value } => {
                write!(f, "write_mem.l (${:08X})=${:08X}", address, value)
            }
            StepLogEntry::WriteMemWord { address, value } => {
                write!(f, "write_mem.w (${:08X})=${:04X}", address, value)
            }
            StepLogEntry::WriteMemByte { address, value } => {
                write!(f, "write_mem.b (${:08X})=${:02X}", address, value)
            }
        }
    }
}

pub struct StepLog {
    log_count: usize,
    logs: [StepLogEntry; 60],
}

impl StepLog {
    pub fn new() -> StepLog {
        StepLog {
            log_count: 0,
            logs: [StepLogEntry::Null; 60],
        }
    }

    pub fn reset_log(&mut self) {
        self.log_count = 0;
    }

    pub fn add_log(&mut self, log_entry: StepLogEntry) {
        self.logs[self.log_count] = log_entry;
        self.log_count += 1;
    }

    pub fn print_logs(&self) {
        for i in 0..self.log_count {
            print!(" > {}", self.logs[i]);
        }
        println!("");
    }
}
