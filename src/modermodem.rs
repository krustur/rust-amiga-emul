use crate::cpu::instruction::GetDisassemblyResult;
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::kickstart::Kickstart;
use crate::mem::custommemory::CustomMemory;
use crate::mem::Mem;
use crate::register::ProgramCounter;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Modermodem {
    kickstart: Option<Rc<RefCell<dyn Kickstart>>>,
    pub cpu: Cpu,
    pub mem: Mem,
    custom_memory: Option<Rc<RefCell<CustomMemory>>>,
    step_log: StepLog,
}

pub enum LoggingMode {
    None,
    Disassembly,
    DisassemblyWithDetails,
    DisassemblyWithDetailsAndKickstartDebug,
}

impl LoggingMode {
    pub fn log_disassembly(&self) -> bool {
        match self {
            LoggingMode::None => false,
            LoggingMode::Disassembly => true,
            LoggingMode::DisassemblyWithDetails => true,
            LoggingMode::DisassemblyWithDetailsAndKickstartDebug => true,
        }
    }

    pub fn log_disassembly_details(&self) -> bool {
        match self {
            LoggingMode::None => false,
            LoggingMode::Disassembly => false,
            LoggingMode::DisassemblyWithDetails => true,
            LoggingMode::DisassemblyWithDetailsAndKickstartDebug => true,
        }
    }

    pub fn log_kickstart_debug(&self) -> bool {
        match self {
            LoggingMode::None => false,
            LoggingMode::Disassembly => false,
            LoggingMode::DisassemblyWithDetails => false,
            LoggingMode::DisassemblyWithDetailsAndKickstartDebug => true,
        }
    }
}

impl Modermodem {
    pub fn new(
        kickstart: Option<Rc<RefCell<dyn Kickstart>>>,
        cpu: Cpu,
        mem: Mem,
        custom_memory: Option<Rc<RefCell<CustomMemory>>>,
    ) -> Self {
        Self {
            kickstart,
            cpu,
            mem,
            step_log: StepLog::new(),
            custom_memory,
        }
    }

    pub fn step(&mut self) {
        let logging_mode = LoggingMode::DisassemblyWithDetailsAndKickstartDebug;

        let pc_address = self.cpu.register.reg_pc.get_address();
        if let Some(kickstart) = &self.kickstart {
            let kickstart = kickstart.borrow_mut();

            if logging_mode.log_kickstart_debug() {
                if let Some(comment) = kickstart.get_comment(pc_address) {
                    println!("                              ; {}", comment);
                }
            }
            if logging_mode.log_disassembly() {
                if !kickstart.get_no_print_disassembly_before_step(pc_address) {
                    let disassembly_result = self
                        .cpu
                        .get_next_disassembly(&mut self.mem, &mut self.step_log);
                    self.cpu
                        .print_disassembly(&self.mem, &disassembly_result, false);
                }
            }
        }

        self.step_log.reset_log();
        self.cpu
            .execute_next_instruction_step_log(&mut self.mem, &mut self.step_log);

        if let Some(kickstart) = &self.kickstart {
            let kickstart = kickstart.borrow_mut();

            if !kickstart.get_no_print_disassembly_before_step(pc_address) {
                self.step_log.print_logs(&logging_mode);
            }
            self.step_log.print_log_strings();

            // if cpu.memory.overlay == false {
            //     let new_exec_base = cpu.memory.get_long_no_log(0x00000004);
            //     if exec_base != new_exec_base {
            //         println!("ExecBase changed from ${:08X} to ${:08X}", exec_base, new_exec_base);
            //         exec_base = new_exec_base;
            //     }
            // }

            if logging_mode.log_disassembly_details() {
                if kickstart.get_print_registers_after_step(pc_address) {
                    self.cpu.register.print_registers();
                }
                if let Some((dump_memory_start, dump_memory_end)) =
                    kickstart.get_dump_memory_after_step(pc_address)
                {
                    self.mem.print_hex_dump(dump_memory_start, dump_memory_end);
                }
                if let Some((dump_areg_memory_register, dump_areg_memory_length)) =
                    kickstart.get_dump_areg_memory_after_step(pc_address)
                {
                    let start = self
                        .cpu
                        .register
                        .get_a_reg_long_no_log(dump_areg_memory_register);
                    let end = start + dump_areg_memory_length;
                    self.mem.print_hex_dump(start, end);
                }
                if let Some((disasm_memory_start, disasm_memory_end)) =
                    kickstart.get_print_disassembly_after_step(pc_address)
                {
                    let mut disassembly_pc = ProgramCounter::from_address(disasm_memory_start);
                    while disassembly_pc.get_address() <= disasm_memory_end {
                        self.step_log.reset_log();
                        let disassembly_result = self.cpu.get_disassembly(
                            &mut disassembly_pc,
                            &mut self.mem,
                            &mut self.step_log,
                        );
                        self.cpu
                            .print_disassembly(&self.mem, &disassembly_result, true);
                        disassembly_pc =
                            ProgramCounter::from_address(disassembly_result.address_next);
                    }
                }
            }
        }

        // step custom register stuff
        if let Some(custom_memory) = &self.custom_memory {
            let mut custom_memory = custom_memory.borrow_mut();
            let mut new_vhpos = custom_memory.vhpos + 1;
            // e2 is the max horizontal position (according to hrm page 23)
            if new_vhpos & 0x00ff > 0xe2 {
                new_vhpos &= 0xff00;
                new_vhpos += 0x0100;
            }
            custom_memory.vhpos = new_vhpos;
        }
    }

    pub fn get_next_disassembly_no_log(&mut self) -> GetDisassemblyResult {
        self.cpu
            .get_next_disassembly(&mut self.mem, &mut StepLog::new())
    }
}
