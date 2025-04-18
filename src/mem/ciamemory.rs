use crate::cpu::step_log::StepLog;

use super::memory::{Memory, SetMemoryResult};
use std::cell::Cell;
use std::collections::VecDeque;
use std::{any::Any, fmt};

/*
   CIA timers:
    - [x] Count from present down to zero
    - [ ] Different modes, can be selected through control register
        - [ ] one for each timer
    - [ ] bit 5+6 in the control register determins what signals decrement the timers
        - [ ] A: 1. Timer A is decremented each clock cycle (INMODE=0)
        - [ ] A: 2. Each high pulse on the CNT line decrements the timer (INMODE=1)
        - [ ] B: 1. Clock cycles (INMODE bits = 00)
        - [ ] B: 2. CNT pulses (INMODE bits = 01)
        - [ ] B: 3. Timer A timeouts (allows two timers to form a 32-bit timer). (INMODE bits = 10)
        - [ ] B: 4. Timer A timeouts when the CNT line is high (allows the length of a pulse on the CNT line to be measuered) (INMODE bits = 11)
    - [X] the timeouts of a timer are registered in the Interrupt Control Reigster (ICR)
        - [X] A: TA bit (no 0)
        - [X] B: TB bit (no 1)
    - [ ] these bits, like all of th ebits in the ICR, remain set until the ICR is read.
    - [ ] In addition it is also possible to output the timeouts to parallel port B.
        - [ ] If the PBon bit is set in the control register for the given timer (CRA or CRB), the each timeout appears on the appropriate port line.
            [ ] (PB6 for timer A and PB7 for timer B)
    - [ ] Two output modes can be selected with the OUTMODE bit:
        [ ] OUTMODE = 0 = Pulse mode
        [ ] OUTMODE = 1 = Toggle mode
    - [X] The timers are started and stopped with the START bit in the control register
        [X] START = 0 => stop
        [X] START = 1 => start
    - [X] RUNMODE bit selects between the one-shot and continous mode
        - [X] one-shot => timer stops after timeout and set the START bit back to 0
        - [X] continous => timer restarts after timeout
    - [ ] writes to timer register doesn't write directly to the count, but to a latch
        - [ ] transfer from latch to timer:
            - [ ] 1. Se the LOAD bit in the control register
            - [X] 2. Each time the timer runs out, the latch is automatically transferred
            - [X] 3. After a write access to the timer high register, time time is stopped (stop = 0), it is automatically loaded with the value in the latch. Therefore the low byte of the timer should always be init first.

*/
enum  CiaSlot {
    A,
    B,
}

struct CiaChip {
    // Data ports
    pra: u8, // Port A
    prb: u8, // Port B

    // Data direction registers
    ddra: u8, // Data Direction Register A
    ddrb: u8, // Data Direction Register B

    // Timers
    timer_a_latch: u16,
    timer_b_latch: u16,
    timer_a: Cell<u16>,
    timer_b: Cell<u16>,
    // timer_a_running: bool,
    // timer_b_running: bool,

    // Counter
    tod: u32,

    // Serial
    sp: u8,

    // Interrupts
    icr_mask: u8, // Interrupt Control Register
    icr_data: u8,

    // Peripheral control registers
    cra: u8, // Control Register A
    crb: u8, // Control Register B

    // Event queue (for emulation purposes)
    event_queue: VecDeque<String>,
}

impl CiaChip {
    fn new(pra: u8) -> Self {
        Self {
            pra,
            prb: 0x00,
            ddra: 0x00,
            ddrb: 0x00,
            timer_a_latch: 0x0000,
            timer_b_latch: 0x0000,
            // timer_a_running: false,
            // timer_b_running: false,
            timer_a: Cell::new(0xFFFF),
            timer_b: Cell::new(0xFFFF),
            sp: 0x00,
            tod: 0x00000000,
            icr_mask: 0x00,
            icr_data: 0x00,
            cra: 0x00,
            crb: 0x00,
            event_queue: VecDeque::new(),
        }
    }

    // Read from a register
    fn read_register(&self, reg: u8) -> u8 {
        match reg {
            0x00 => self.pra,
            0x01 => self.prb,
            0x02 => self.ddra,
            0x03 => self.ddrb,
            0x04 => (self.timer_a.get() & 0xFF) as u8,
            0x05 => (self.timer_a.get() >> 8) as u8,
            0x06 => (self.timer_b.get() & 0xFF) as u8,
            0x07 => (self.timer_b.get() >> 8) as u8,
            0x08 => (self.tod & 0xff) as u8,
            0x09 => ((self.tod >> 8) & 0xff) as u8,
            0x0a => ((self.tod >> 16) & 0xff) as u8,
            0x0b => 0x00,
            0x0c => self.sp,
            0x0d => {
                let result = self.icr_data;
                unsafe {
                    // This is a pain - we actually write to a register when reading from it
                    // which breaks the entire pattern of using mut only for write calls.
                    let mutable_self = self as *const CiaChip as *mut CiaChip;
                    (*mutable_self).icr_data = 0x00;
                }
                result
            },
            0x0e => self.cra,
            0x0f => self.crb,
            _ => 0xFF, // Unmapped register
        }
    }

    // Write to a register
    fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0x00 => self.pra = value,
            0x01 => self.prb = value,
            0x02 => self.ddra = value,
            0x03 => self.ddrb = value,
            0x04 => {
                // talo
                self.timer_a_latch = (self.timer_a_latch & 0xFF00) | value as u16
            }
            0x05 => {
                // tahi
                // Stop timer a ... nope - false info in Amiga System Programmer's Guide?
                // self.cra = self.cra & 0xfe;
                // One-shot mode auto starts when writing to hi
                if self.cra & 0x08 == 0x08 {
                    self.cra |= 0x01;
                }
                // ... and load from latch to actual timer
                self.timer_a_latch = (self.timer_a_latch & 0x00FF) | ((value as u16) << 8);
                self.timer_a.set(self.timer_a_latch);
            }
            0x06 => {
                // tblo
                self.timer_b_latch = (self.timer_b_latch & 0xFF00) | value as u16
            }
            0x07 => {
                // tbhi
                // Stop timer b ... nope - false info in Amiga System Programmer's Guide?
                // self.crb = self.crb & 0xfe;
                // One-shot mode auto starts when writing to hi
                if self.crb & 0x08 == 0x08 {
                    self.crb |= 0x01;
                }
                // ... and load from latch to actual timer
                self.timer_b_latch = (self.timer_b_latch & 0x00FF) | ((value as u16) << 8);
                self.timer_b.set(self.timer_b_latch);
            }
            0x08 => self.tod = (self.tod & 0xffffff00) | value as u32,
            0x09 => self.tod = (self.tod & 0xffff00ff) | ((value as u32) << 8),
            0x0a => self.tod = (self.tod & 0xff00ffff) | ((value as u32) << 16),
            0x0b => {
                todo!()
            }
            0x0c => self.sp = value,
            0x0d => {
                match value & 0x80 {
                    0x80 => self.icr_mask = self.icr_mask | value & 0x7f,
                    _ => self.icr_mask = self.icr_mask & !value,
                }
            }
            // TODO: cra & crb LOAD strobe = load timer from latch but don't set LOAD bit in memory
            0x0e => self.cra = value,
            0x0f => self.crb = value,
            _ => {}
        }
    }

    // Simulate a single clock cycle
    pub fn step_clock_cycle(&mut self) {
        // Timer A
        if (self.cra & 0x01) == 0x01 {
            let new_timer_a = self.timer_a.get().wrapping_sub(1);
            self.timer_a.set(new_timer_a);
            if new_timer_a == 0 {
                self.timer_a.set(self.timer_a_latch);
                // one-shot = stop
                if self.cra & 0x08 == 0x08 {
                    self.cra &= 0xfe;
                }
                // TA
                self.icr_data |= 0x01;
                self.event_queue.push_back("Timer A Interrupt".to_string());
            }
        }


        // Timer B
        if (self.crb & 0x01) == 0x01 {
            let new_timer_b = self.timer_b.get().wrapping_sub(1);
            self.timer_b.set(new_timer_b);
            if new_timer_b == 0 {
                self.timer_b.set(self.timer_b_latch);
                // one-shot = stop
                if self.crb & 0x08 == 0x08 {
                    self.crb &= 0xfe;
                }
                // TB
                self.icr_data |= 0x02;
                self.event_queue.push_back("Timer B Interrupt".to_string());
            }
        }

        // TOD
        self.tod += 1;
    }
}

pub struct CiaMemory {
    cia_a: CiaChip,
    cia_b: CiaChip,
}

impl fmt::Display for CiaMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ROM: ${:08X}-${:08X} ({}) bytes)",
            self.get_start_address(),
            self.get_end_address(),
            self.get_length()
        )
    }
}

impl Memory for CiaMemory {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_start_address(&self) -> u32 {
        self.get_start_address()
        // return 0x00BF0000;
    }

    fn get_end_address(&self) -> u32 {
        self.get_end_address()
        // return 0x00BFFFFF;
    }

    fn get_length(&self) -> usize {
        self.get_length()
    }

    fn get_long(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u32 {
        panic!("cia memory get_long: ${:06X}", address);
    }

    fn set_long(self: &mut CiaMemory, step_log: &mut StepLog, address: u32, value: u32) {
        panic!("cia memory set_long: ${:06X}", address);
    }

    fn get_word(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u16 {
        todo!(
            "cia memory get_word: ${:06X}. Accessing word can read both cia_a and cia_b",
            address
        );
    }

    fn set_word(self: &mut CiaMemory, step_log: &mut StepLog, address: u32, value: u16) {
        todo!(
            "cia memory set_word: ${:06X}. Accessing word can set both cia_a and cia_b",
            address
        );
    }

    fn get_byte(self: &CiaMemory, step_log: &mut StepLog, address: u32) -> u8 {
        match address {
            _ => {
                step_log.add_log_string(format!(
                    "CIA: TODO: get_byte() for CIA memory ${:06X}",
                    address
                ));
                let (cia_slot, register_index) = Self::get_cia_slot_and_register_index(address);
                match cia_slot {
                    CiaSlot::A => self.cia_a.read_register(register_index),
                    CiaSlot::B => self.cia_b.read_register(register_index),
                }
            }
        }
    }

    fn set_byte(
        self: &mut CiaMemory,
        step_log: &mut StepLog,
        address: u32,
        value: u8,
    ) -> Option<SetMemoryResult> {
        match address {
            0xBFE001 => {
                // http://amigadev.elowar.com/read/ADCD_2.1/Hardware_Manual_guide/node018F.html
                // let pra_fir1 = (value & 0x80) == 0x80;
                // let pra_fir0 = (value & 0x40) == 0x40;
                // let pra_rdy = (value & 0x20) == 0x20; // DSKRDY - Disk ready (active low)
                // let pra_tk0 = (value & 0x10) == 0x10; // DSKTRACK0 - Track zero detect
                // let pra_wpro = (value & 0x08) == 0x08; // DSKPROT - Disk is write protected (active low)
                // let pra_chng = (value & 0x04) == 0x04; // DSKCHANGE - Disk has been removed from the drive.  The signal goes low whenever a disk is removed.
                // let pra_led = (value & 0x02) == 0x02;
                // let pra_ovl = (value & 0x01) == 0x01;
                let old_value = self.cia_a.read_register(0x00);

                self.update_cia_a_pra_led(old_value, value);
                let overlay_changed = self.update_cia_a_pra_ovl(old_value, value);
                self.update_cia_a_pra_todo(step_log, address, old_value, value);

                self.cia_a.write_register(0x00, value);

                if let Some(overlay) = overlay_changed {
                    Some(SetMemoryResult {
                        set_overlay: Some(overlay),
                    })
                } else {
                    None
                }
            }
            _ => {
                let (cia_slot, register_index) = Self::get_cia_slot_and_register_index(address);
                match cia_slot {
                    CiaSlot::A => self.cia_a.write_register(register_index, value),
                    CiaSlot::B => self.cia_b.write_register(register_index, value),
                }
                step_log.add_log_string(format!(
                    "CIA: TODO: set_byte() for CIA memory ${:06X} to ${:02X}",
                    address, value
                ));
                None
            }
        }
    }
}

impl CiaMemory {
    pub fn new() -> CiaMemory {
        CiaMemory {
            cia_a: CiaChip::new(0x01),
            cia_b: CiaChip::new(0x00),
        }
    }

    pub fn step_clock_cycle(&mut self) {
        self.cia_a.step_clock_cycle();
        self.cia_b.step_clock_cycle();
    }

    pub fn is_cia_memory(address: u32) -> bool {
        match address {
            0x00bf0000..=0x00bfffff => true,
            _ => false,
        }
    }

    pub fn get_start_address(&self) -> u32 {
        0x00BF0000
    }

    pub fn get_end_address(&self) -> u32 {
        0x00BFFFFF
    }

    pub fn get_length(&self) -> usize {
        0x10000
    }

    fn get_cia_slot_and_register_index(address: u32) -> (CiaSlot, u8) {
        let register_index = ((address & 0x000f00) >> 8) as u8;
        let cia_slot = if address & 0b1010_0000_0001_0000_0000_0001 == 0b1010_0000_0000_0000_0000_0001 {
            CiaSlot::A
        } else if address & 0b1010_0000_0010_0000_0000_0001 == 0b1010_0000_0000_0000_0000_0000 {
            CiaSlot::B
        } else {
          panic!()
        };
        (cia_slot, register_index)
    }

    fn update_cia_a_pra_led(self: &mut CiaMemory, old_pra: u8, pra: u8) {
        if old_pra & 0x02 != pra & 0x02 {
            let on_or_off = match (pra & 0x02) == 0x02 {
                true => "OFF",
                false => "ON",
            };
            println!(
                "   -CIA: PRA LED changed from ${:02X} to ${:02X} [{}]",
                old_pra & 0x02,
                pra & 0x02,
                on_or_off
            );
        }
    }

    fn update_cia_a_pra_ovl(self: &mut CiaMemory, old_pra: u8, pra: u8) -> Option<bool> {
        if old_pra & 0x01 != pra & 0x01 {
            let ovl = (pra & 0x01) == 0x01;
            println!(
                "   -CIA: PRA OVL changed from ${:02X} to ${:02X} [{}]",
                old_pra & 0x01,
                pra & 0x01,
                ovl
            );
            Some(ovl)
        } else {
            None
        }
    }

    fn update_cia_a_pra_todo(
        self: &mut CiaMemory,
        step_log: &mut StepLog,
        address: u32,
        old_pra: u8,
        pra: u8,
    ) {
        if pra > 3 {
            step_log.add_log_string(format!(
                "CIA: TODO: set_byte() for CIA-A PRA [(]${:06X}] to ${:02X}",
                address, pra
            ));
        }
    }
}
