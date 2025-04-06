#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use std::cell::RefCell;
use std::rc::Rc;


use crate::{mem::custommemory::CustomMemory};
use {
    cpu::Cpu,
    mem::{ciamemory::CiaMemory, rammemory::RamMemory, Mem},
};

use crate::kickstart_1_2::Kickstart_1_2;

use crate::modermodem::Modermodem;

mod cpu;
mod kickstart;
mod kickstart_1_2;
mod kickstart_3_1_4;
mod mem;
mod modermodem;
mod register;

// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
// static ROM_FILE_PATH: &str = "C:\\WS\\Amiga\\Kickstart v3.1 rev 40.68 (1993)(Commodore)(A1200).rom";
// static ROM_FILE_PATH_3_1_4: &str = "D:\\Amiga\\AmigaOS 3.1.4 for 68k Amiga 1200\\OS314_A1200\\ROMs\\emulation_or_maprom\\kick.a1200.46.143";
static ROM_FILE_PATH_1_2: &str = "D:\\Amiga\\ROM\\Kickstart 1.2.rom";
// static ROM_FILE_PATH_2_0: &str = "D:\\Amiga\\ROM\\Kickstart 2.0.rom";

fn main() {
    println!("Begin emulation!");

    let mut mem = Mem::new();

    let kickstart = Rc::new(RefCell::new(Kickstart_1_2::new(ROM_FILE_PATH_1_2, &mut mem)));
    
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
    let fast_ram = RamMemory::from_range(0x00C00000, 0x00DBFFFF);
    mem.add_range(Rc::new(RefCell::new(fast_ram)));

    // CIA memory
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(cia_memory)));

    // CUSTOM memory
    let custom_memory = Rc::new(RefCell::new(CustomMemory::new()));
    mem.add_range(custom_memory.clone());

    let cpu = Cpu::new(&mem);
    println!("Beginning of ROM");
    // mem.borrow().print_hex_dump(0xf80000, 0xf801ff);
    println!("Checksum:");
    // mem.borrow().print_hex_dump(0xffffe8, 0xffffeb);
    println!("Chip memory-string:");
    // mem.borrow().print_hex_dump(0x00F803AE, 0x00F803B9);

    cpu.register.print_registers();

    // let mut disassembly_pc = cpu.register.reg_pc.clone();
    // for i in 0..70 {
    //     let disassembly_result = cpu.register.get_disassembly(&mut disassembly_pc);

    //     disassembly_pc = ProgramCounter::from_address(disassembly_result.address_next);

    //     cpu.print_disassembly(&disassembly_result);
    // }

    // NTSC has 262/263 scan lines
    // PAL has 312/313 scan lines
    // let mut exec_base: u32 = 0xffffffff;
    // let mut step_log = StepLog::new();
    // step_log.add_log(StepLogEntry::ReadRegisterLong{register_type: RegisterType::Data, register_index:1, value: 0xdddddd11});
    // step_log.add_log(StepLogEntry::WriteRegisterLong{register_type:RegisterType::Address,register_index:2, value:  0xaaaaaa22});
    // step_log.print_logs();
    // step_log.reset_log();
    // step_log.add_log(StepLogEntry::WriteRegisterLong{register_type:RegisterType::Address,register_index:7, value:  0xaaaaaa77});
    // step_log.print_logs();

    let mut modermodem = Modermodem::new(Some(kickstart), cpu, mem, Some(custom_memory));

    loop {
        modermodem.step();
    }
}



#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::{cpu, mem};
    use crate::mem::ciamemory::CiaMemory;
    use crate::mem::rammemory::RamMemory;
    use crate::modermodem::Modermodem;
    use crate::register::ProgramCounter;

    pub(crate) fn instr_test_setup(code: Vec<u8>, mem_ranges: Option<Vec<RamMemory>>) -> Modermodem {
        // let mut mem_ranges_internal: Vec<Box<dyn Memory>> = Vec::new();
        let mut mem = mem::Mem::new();
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
        // mem.add_range(Rc::new(RefCell::new(RamMemory::from_range(0xffffffff, 0xffffffff))));
        let mut cpu = cpu::Cpu::new(&mem);
        cpu.register.reg_pc = ProgramCounter::from_address(0xC00000);
        cpu.register.set_ssp_reg(0x01000400);
        let modermodem = Modermodem::new(None, cpu, mem, None);
        modermodem
    }
}