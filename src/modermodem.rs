use crate::cpu::instruction::GetDisassemblyResult;
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::kickstart::Kickstart;
use crate::mem::custommemory::CustomMemory;
use crate::mem::Mem;
use crate::register::ProgramCounter;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use crate::mem::ciamemory::CiaMemory;

pub struct Modermodem {
    kickstart: Option<Rc<RefCell<dyn Kickstart>>>,
    pub cpu: Cpu,
    pub mem: Mem,
    custom_memory: Option<Rc<RefCell<CustomMemory>>>,
    cia_memory: Option<Rc<RefCell<CiaMemory>>>,
    step_log: StepLog,
    previous_now: Option<Instant>,
    emulator_time: Duration,
    emulator_time_next_log: Duration,
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
        cia_memory: Option<Rc<RefCell<CiaMemory>>>,
    ) -> Self {
        Self {
            kickstart,
            cpu,
            mem,
            step_log: StepLog::new(),
            custom_memory,
            cia_memory,
            previous_now: None,
            emulator_time: Duration::ZERO,
            emulator_time_next_log: Duration::ZERO,
        }
    }

    pub fn step(&mut self) {

        let now = Instant::now();
        let passed = match self.previous_now {
            Some(time) => now.duration_since(time),
            None => now.duration_since(now),
        };
        self.previous_now = Some(now);
        let last_cycle = self.emulator_time.as_nanos() * self.cpu.cpu_speed.get_hz() / 1000000000;
        self.emulator_time += passed;

        let this_cycle = self.emulator_time.as_nanos() * self.cpu.cpu_speed.get_hz() / 1000000000;

        // println!("emulator_time: {}.{} [{}]", self.emulator_time.as_secs(), self.emulator_time.subsec_millis(), passed.as_nanos());
        if self.emulator_time > self.emulator_time_next_log {
            println!("last_cycle: {} this_cycle: {} num_cycles: [{}]", last_cycle, this_cycle, this_cycle - last_cycle);
            self.emulator_time_next_log += Duration::from_secs(1);
        }

        let logging_mode = LoggingMode::Disassembly;


        let pc_address = self.cpu.register.reg_pc.get_address();
        if self.cpu.stopped == false {
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
                        disassembly_result
                            .print_disassembly(false);
                    }
                }
            }
        }

        self.step_log.reset_log();
        self.cpu
            .execute_next_instruction_step_log(&mut self.mem, &mut self.step_log);

        if self.cpu.stopped == false {
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
                            disassembly_result
                                .print_disassembly(true);
                            disassembly_pc =
                                ProgramCounter::from_address(disassembly_result.address_next);
                        }
                    }
                }
            }
        }

        // step custom register stuff
        // TODO: Move to inside CustomMemory
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
        if let Some(cia_memory) = &self.cia_memory {
            let mut cia_memory = cia_memory.borrow_mut();
            cia_memory.step_clock_cycle();
        }
    }

    pub fn get_next_disassembly_no_log(&mut self) -> GetDisassemblyResult {
        self.cpu
            .get_next_disassembly(&mut self.mem, &mut StepLog::new())
    }

    pub fn get_disassembly_no_log(&mut self, start_address: u32, stop_address: u32) -> Vec<GetDisassemblyResult> {
        let mut result = Vec::new();
        let mut address = start_address;
        while address <= stop_address {
            let mut pc = ProgramCounter::from_address(address);
            let address_disassembly = self.cpu.get_disassembly(&mut pc, &mut self.mem, &mut StepLog::new());
            address = address_disassembly.address_next;
            result.push(address_disassembly);
        }
        result
    }
}
