#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

// TODO: [X] Clean up the step log code!
// TODO: [ ] Generics everything, maybe typed byte/word/long?
// TODO: [ ] Interrupts: VHPOS
// TODO: [ ] Interrupts: CIA timers
// TODO: [ ] Then Bugfix SUBX
// TODO: [ ] Prefix _all_ tests with instruction name and size
// TODO: [ ] Missing tests for ROLR-instructions
// TODO: [ ] Missing tests for Scc-instruction

use std::cell::RefCell;
use std::rc::Rc;


use crate::mem::custommemory::CustomMemory;
use {
    cpu::Cpu,
    mem::{ciamemory::CiaMemory, rammemory::RamMemory, Mem},
};

use crate::kickstart_debug_1_2::KickstartDebug_1_2;

use crate::modermodem::Modermodem;

mod cpu;
mod kickstart;
mod kickstart_debug_1_2;
mod kickstart_debug_3_1_4;
mod mem;
mod modermodem;
mod register;
mod aint;

use crate::cpu::step_log::{DisassemblyLogMode, StepLog};
use crate::cpu::CpuSpeed;
use crate::kickstart::Kickstart;

// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
// static ROM_FILE_PATH: &str = "C:\\WS\\Amiga\\Kickstart v3.1 rev 40.68 (1993)(Commodore)(A1200).rom";
// static ROM_FILE_PATH_3_1_4: &str = "D:\\Amiga\\AmigaOS 3.1.4 for 68k Amiga 1200\\OS314_A1200\\ROMs\\emulation_or_maprom\\kick.a1200.46.143";
static ROM_FILE_PATH_1_2: &str = "D:\\Amiga\\ROM\\Kickstart 1.2.rom";
// static ROM_FILE_PATH_2_0: &str = "D:\\Amiga\\ROM\\Kickstart 2.0.rom";

fn main() {
    println!("Begin emulation!");

    // let mut prev = Instant::now();
    // for _ in 0..1_000_000 {
    //     let now = Instant::now();
    //     if now != prev {
    //         println!("Timer resolution: {:?}", now.duration_since(prev));
    //         break;
    //     }
    //     prev = now;
    // }
    // panic!();

    // CUSTOM memory
    // CIA memory
    let custom_memory = Rc::new(RefCell::new(CustomMemory::new()));
    let cia_memory = Rc::new(RefCell::new(CiaMemory::new()));
    let mut mem = Mem::new(Some(custom_memory.clone()), Some(cia_memory.clone()));

    let kickstart = Rc::new(RefCell::new(Kickstart::new(ROM_FILE_PATH_1_2, &mut mem)));
    let kickstart_debug = KickstartDebug_1_2::new();
    
    // Hack "CDTV & CD32 Extended ROM / A4000 Diagnostics ROM" as RAM
    // ROM code checks for $1111 at 0F00000 ()
    let no_extended_rom_hack = RamMemory::from_range(0x00f00000, 0x00F7FFFF);
    mem.add_range(Rc::new(RefCell::new(no_extended_rom_hack)));

    // Hack for "A600 & A1200 IDE controller"
    // $00DA8000 -> $ 00DAFFFF = Credit Card & IDE configuration registers
    // ROM code writes $00.B to $00DA8000
    // let no_creditcard_registers_hack = RamMemory::from_range(0x00DA8000, 0x00DBFFFF);
    // mem.borrow_mut().add_range(Rc::new(RefCell::new(no_creditcard_registers_hack)));

    // 2 MB chip ram
    // let chip_ram = RamMemory::from_range(0x00000000, 0x001FFFFF);
    // mem.borrow_mut().add_range(Rc::new(RefCell::new(chip_ram)));

    // 0.5 MB chip ram
    let chip_ram = RamMemory::from_range(0x00000000, 0x0007FFFF);
    mem.add_range(Rc::new(RefCell::new(chip_ram)));

    // 0.5 MB of fast ram
    // let fast_ram = RamMemory::from_range(0x00200000, 0x0027FFFF);
    // mem.add_range(Rc::new(RefCell::new(fast_ram)));

    let ssp_address = mem.get_long_no_log(0x0);
    let pc_address = mem.get_long_no_log(0x4);

    let cpu = Cpu::new(CpuSpeed::PAL_7_093790_MHz, ssp_address, pc_address);
    println!("Beginning of ROM");
    mem.print_hex_dump(0xf80000, 0xf801ff);

    println!("Checksum:");
    mem.print_hex_dump(0xffffe8, 0xffffeb);

    println!("Chip memory-string:");
    mem.print_hex_dump(0x00F803AE, 0x00F803B9);

    cpu.register.print_registers();

    // let mut disassembly_pc = cpu.register.reg_pc.clone();
    // for i in 0..70 {
    //     let disassembly_result = cpu.register.get_disassembly(&mut disassembly_pc);

    //     disassembly_pc = ProgramCounter::from_address(disassembly_result.address_next);

    //     cpu.print_disassembly(&disassembly_result);
    // }

    // NTSC has 262/263 scan lines
    // PAL has 312/313 scan lines

    let step_log = StepLog::new(DisassemblyLogMode::DisassemblyWithKickstartDebugAndDetails, Box::new(kickstart_debug));
    let mut modermodem = Modermodem::new(kickstart, step_log, cpu, mem, custom_memory, cia_memory);

    let disassembly = modermodem.get_disassembly_no_log(0x00FE930E, 0x00FE9336);
    for disassembly_row in disassembly {
        disassembly_row.print_disassembly(true);
    }

    loop {
        modermodem.step();
    }
}



#[cfg(test)]
mod tests {
    use crate::cpu::CpuSpeed;
    use crate::mem::ciamemory::CiaMemory;
    use crate::mem::rammemory::RamMemory;
    use crate::modermodem::Modermodem;
    use crate::{cpu, mem};
    use std::cell::RefCell;
    use std::rc::Rc;

    pub(crate) fn instr_test_setup(code: Vec<u8>, mem_ranges: Option<Vec<RamMemory>>) -> Modermodem {
        let mut mem = mem::Mem::new(None, None);
        let code = RamMemory::from_bytes(0x00C00000, code);
        let stack = RamMemory::from_range(0x01000000, 0x010003ff);
        let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
        let cia_memory = CiaMemory::new();
        mem.add_range(Rc::new(RefCell::new(code)));
        mem.add_range(Rc::new(RefCell::new(stack)));
        mem.add_range(Rc::new(RefCell::new(vectors)));
        mem.add_range(Rc::new(RefCell::new(cia_memory)));
        if let Some(mem_ranges) = mem_ranges {
            for mem_range in mem_ranges {
                mem.add_range(Rc::new(RefCell::new(mem_range)));
            }
        }
        let cpu = cpu::Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x01000400, 0xC00000);
        let modermodem = Modermodem::bare(cpu, mem);
        modermodem
    }
}