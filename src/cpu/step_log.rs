use crate::cpu::instruction::GetDisassemblyResult;
use crate::cpu::Cpu;
use crate::kickstart::{KickstartDebug, NoKickstartDebug};
use crate::mem::Mem;
use crate::register::{ProgramCounter, RegisterType, StatusRegister};
use std::fmt::Display;

pub enum DisassemblyLogMode {
    None,
    Disassembly,
    DisassemblyWithKickstartDebug,
    DisassemblyWithKickstartDebugAndDetails,
}

impl DisassemblyLogMode {
    pub fn log_disassembly(&self) -> bool {
        match self {
            DisassemblyLogMode::None => false,
            DisassemblyLogMode::Disassembly => true,
            DisassemblyLogMode::DisassemblyWithKickstartDebug => true,
            DisassemblyLogMode::DisassemblyWithKickstartDebugAndDetails => true,
        }
    }

    pub fn log_kickstart_kickstart_debug(&self) -> bool {
        match self {
            DisassemblyLogMode::None => false,
            DisassemblyLogMode::Disassembly => false,
            DisassemblyLogMode::DisassemblyWithKickstartDebug => true,
            DisassemblyLogMode::DisassemblyWithKickstartDebugAndDetails => true,
        }
    }

    pub fn log_disassembly_details(&self) -> bool {
        match self {
            DisassemblyLogMode::None => false,
            DisassemblyLogMode::Disassembly => false,
            DisassemblyLogMode::DisassemblyWithKickstartDebug => false,
            DisassemblyLogMode::DisassemblyWithKickstartDebugAndDetails => true,
        }
    }
}

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
    disassembly_log_mode: DisassemblyLogMode,
    kickstart_debug: Box<dyn KickstartDebug>,
    address: u32,
    address_comment: Option<String>,
    disassembly: Option<GetDisassemblyResult>,
    log_count: usize,
    logs: [StepLogEntry; 60],
    log_strings: Vec<String>,
}

impl StepLog {
    pub fn none() -> StepLog {
        StepLog {
            disassembly_log_mode: DisassemblyLogMode::None,
            kickstart_debug: Box::new(NoKickstartDebug::new()),
            address: 0x00000000,
            address_comment: None,
            disassembly: None,
            log_count: 0,
            logs: [StepLogEntry::Null; 60],
            log_strings: vec![],
        }
    }

    pub fn new(
        disassembly_log_mode: DisassemblyLogMode,
        kickstart_debug: Box<dyn KickstartDebug>,
    ) -> StepLog {
        StepLog {
            disassembly_log_mode,
            kickstart_debug,
            address: 0x00000000,
            address_comment: None,
            disassembly: None,
            log_count: 0,
            logs: [StepLogEntry::Null; 60],
            log_strings: vec![],
        }
    }

    pub fn reset_log(&mut self) {
        self.log_count = 0;
        self.log_strings.clear();
    }

    pub fn log_disassembly(&mut self, cpu: &mut Cpu, mem: &mut Mem) {
        if cpu.stopped == false && self.disassembly_log_mode.log_disassembly() {
            let address = cpu.register.reg_pc.get_address();
            self.address = address;
            self.address_comment = self.kickstart_debug.get_address_comment(address);
            self.disassembly = Some(cpu.get_next_disassembly(mem, self));
        } else {
            self.address = 0x00000000;
            self.address_comment = None;
            self.disassembly = None;
        }
    }

    pub fn add_step_log_entry(&mut self, log_entry: StepLogEntry) {
        self.logs[self.log_count] = log_entry;
        self.log_count += 1;
    }

    pub fn add_log_string(&mut self, log: String) {
        self.log_strings.push(log);
    }

    pub fn print(&self, cpu: &mut Cpu, mem: &mut Mem) {
        if self.disassembly_log_mode.log_disassembly()
            && !self
                .kickstart_debug
                .disable_print_disassembly_for_address(self.address)
        {
            if let Some(address_comment) = &self.address_comment {
                println!("                                   ; {}", address_comment);
            }
            if let Some(disassembly) = &self.disassembly {
                disassembly.print_disassembly(false);
            }
            if self.disassembly_log_mode.log_disassembly_details() {
                self.print_logs();
            }
            println!();
        }

        self.print_log_strings();

        if self.disassembly_log_mode.log_disassembly_details() {
            if self.kickstart_debug.should_print_registers_after_step(self.address) {
                cpu.register.print_registers();
            }
            if let Some((dump_memory_start, dump_memory_end)) =
                self.kickstart_debug.should_dump_memory_after_step(self.address)
            {
                mem.print_hex_dump(dump_memory_start, dump_memory_end);
            }
            if let Some((dump_areg_memory_register, dump_areg_memory_length)) =
                self.kickstart_debug.should_dump_areg_memory_after_step(self.address)
            {
                let start = cpu
                    .register
                    .get_a_reg_long_no_log(dump_areg_memory_register);
                let end = start + dump_areg_memory_length;
                mem.print_hex_dump(start, end);
            }
            if let Some((disasm_memory_start, disasm_memory_end)) =
                self.kickstart_debug.should_dump_disassembly_after_step(self.address)
            {
                let mut disassembly_pc = ProgramCounter::from_address(disasm_memory_start);
                while disassembly_pc.get_address() <= disasm_memory_end {
                    // self.step_log.reset_log();
                    let disassembly_result = cpu.get_disassembly(
                        &mut disassembly_pc,
                        mem,
                        &mut StepLog::none(),
                    );
                    disassembly_result
                        .print_disassembly(true);
                    disassembly_pc =
                        ProgramCounter::from_address(disassembly_result.address_next);
                }
            }
        }
    }

    fn print_logs(&self) {
        print!(";");
        for i in 0..self.log_count {
            print!(" > {}", self.logs[i]);
        }
    }

    fn print_log_strings(&self) {
        for l in &self.log_strings {
            println!(
                "                                                            ; {}",
                l
            );
        }
    }
}
