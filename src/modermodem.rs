use crate::cpu::instruction::GetDisassemblyResult;
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::kickstart::Kickstart;
use crate::mem::ciamemory::CiaMemory;
use crate::mem::custommemory::CustomMemory;
use crate::mem::Mem;
use crate::register::ProgramCounter;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Modermodem {
    kickstart: Option<Rc<RefCell<Kickstart>>>,
    pub cpu: Cpu,
    pub mem: Mem,
    custom_memory: Option<Rc<RefCell<CustomMemory>>>,
    cia_memory: Option<Rc<RefCell<CiaMemory>>>,
    step_log: StepLog,
    previous_now: Option<Instant>,
    emulator_time: Duration,
    emulator_time_next_log: Duration,
}

impl Modermodem {
    pub fn bare(cpu: Cpu, mem: Mem) -> Self {
        Self {
            kickstart: None,
            cpu,
            mem,
            step_log: StepLog::none(),
            custom_memory: None,
            cia_memory: None,
            previous_now: None,
            emulator_time: Duration::ZERO,
            emulator_time_next_log: Duration::ZERO,
        }
    }

    pub fn new(
        kickstart: Rc<RefCell<Kickstart>>,
        step_log: StepLog,
        cpu: Cpu,
        mem: Mem,
        custom_memory: Rc<RefCell<CustomMemory>>,
        cia_memory: Rc<RefCell<CiaMemory>>,
    ) -> Self {
        Self {
            kickstart: Some(kickstart),
            cpu,
            mem,
            step_log,
            custom_memory: Some(custom_memory),
            cia_memory: Some(cia_memory),
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
            println!(
                "last_cycle: {} this_cycle: {} num_cycles: [{}]",
                last_cycle,
                this_cycle,
                this_cycle - last_cycle
            );
            self.emulator_time_next_log += Duration::from_secs(1);
        }

        self.step_log.reset_log();
        self.step_log.log_disassembly(&mut self.cpu, &mut self.mem);
        self.cpu
            .execute_next_instruction_step_log(&mut self.mem, &mut self.step_log);
        self.step_log.print(&mut self.cpu, &mut self.mem);

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
            .get_next_disassembly(&mut self.mem, &mut StepLog::none())
    }

    pub fn get_disassembly_no_log(
        &mut self,
        start_address: u32,
        stop_address: u32,
    ) -> Vec<GetDisassemblyResult> {
        let mut result = Vec::new();
        let mut address = start_address;
        while address <= stop_address {
            let mut pc = ProgramCounter::from_address(address);
            let address_disassembly =
                self.cpu
                    .get_disassembly(&mut pc, &mut self.mem, &mut StepLog::none());
            address = address_disassembly.address_next;
            result.push(address_disassembly);
        }
        result
    }
}
