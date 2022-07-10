#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use mem::memory::Memory;

use crate::mem::custommemory::CustomMemory;
use {
    cpu::Cpu,
    mem::{ciamemory::CiaMemory, rammemory::RamMemory, rommemory::RomMemory, Mem},
};

mod cpu;
mod mem;
mod register;

// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
// static ROM_FILE_PATH: &str = "C:\\WS\\Amiga\\Kickstart v3.1 rev 40.68 (1993)(Commodore)(A1200).rom";
static ROM_FILE_PATH: &str = "D:\\Amiga\\AmigaOS 3.1.4 for 68k Amiga 1200\\OS314_A1200\\ROMs\\emulation_or_maprom\\kick.a1200.46.143";
// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 2.0.rom";

fn main() {
    println!("Begin emulation!");

    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();

    let rom = RomMemory::from_file(0xF80000, ROM_FILE_PATH).unwrap();
    mem_ranges.push(Box::new(rom));

    // Hack for "CDTV & CD32 Extended ROM / A4000 Diagnostics ROM"
    // ROM code checks for $1111 at 0F00000 ()
    // let no_extended_rom_hack = RamMemory::from_range(0x00f00000, 0x00F7FFFF);
    // mem_ranges.push(MemRange::from_memory(Box::new(no_extended_rom_hack)));

    // Hack for "A600 & A1200 IDE controller"
    // $00DA8000 -> $ 00DAFFFF = Credit Card & IDE configuration registers
    // ROM code writes $00.B to $00DA8000
    // let no_creditcard_registers_hack = RamMemory::from_range(0x00DA8000, 0x00DBFFFF);
    // mem_ranges.push(MemRange::from_memory(Box::new(
    //     no_creditcard_registers_hack,
    // )));

    // 2 MB chip ram
    let chip_ram = RamMemory::from_range(0x00000000, 0x001FFFFF);
    mem_ranges.push(Box::new(chip_ram));

    // CIA memory
    let cia_memory = CiaMemory::new();
    mem_ranges.push(Box::new(cia_memory));

    // CUSTOM memory
    let custom_memory = CustomMemory::new();
    mem_ranges.push(Box::new(custom_memory));

    // ROM overlay
    let rom_overlay = RomMemory::from_file(0x000000, ROM_FILE_PATH).unwrap();

    let mem = Mem::new(mem_ranges, Box::new(rom_overlay));

    let mut cpu = Cpu::new(mem);
    println!("Beginning of ROM");
    cpu.memory.print_hex_dump(0xf80000, 0xf801ff);
    println!("Checksum:");
    cpu.memory.print_hex_dump(0xffffe8, 0xffffeb);

    cpu.register.print_registers();

    // let mut disassembly_pc = cpu.register.reg_pc.clone();
    // for i in 0..70 {
    //     let disassembly_result = cpu.register.get_disassembly(&mut disassembly_pc);

    //     disassembly_pc = ProgramCounter::from_address(disassembly_result.address_next);

    //     cpu.print_disassembly(&disassembly_result);
    // }

    loop {
        // cpu.print_registers();
        let pc_address = cpu.register.reg_pc.get_address();
        let comment = match pc_address {
            0x00F800D2 => Some(String::from("Stack Pointer = $400")),
            0x00F800D6 => Some(String::from("Calculate check sum to D5")),
            0x00F80CA4 => Some(String::from("Check CPU model and if FPU is present")),
            0x00F800F4 => Some(String::from("Branch if we're at $00F00000")),
            0x00F80102 => Some(String::from(
                "If $1111 at $00F00000 (extended rom), then jmp to it+2 ",
            )),
            0x00F8010C => Some(String::from("A600 & A1200 IDE controller")),
            0x00F80116 => Some(String::from(
                "Zorro 2 IO expansion / PCMCIA registers (A600 & A1200)",
            )),
            0x00F80154 => Some(String::from("A600 & A1200 IDE controller (again?)")),
            0x00F8015C => Some(String::from("CIA something")),
            0x00F80168 => Some(String::from("CIA disable overlay (and something more)")),
            0x00F801A4 => Some(String::from(
                "If checksum (D5) isn't correct, branch to error with red background",
            )),

            0x00F801AE => Some(String::from("Setup exception vectors")),
            0x00F801C0 => Some(String::from(
                "Verify exception vectors ok, if not branch to error with green background",
            )),
            0x00F801D2 => Some(String::from("What now?")),
            _ => None,
        };
        if let Some(comment) = comment {
            println!("                              ; {}", comment);
        }

        let print_disassembly = match pc_address {
            0x00F800E2..=0x00F800E8 => false,
            _ => true,
        };
        let print_registers = match pc_address {
            0x00F800EC => true,
            _ => false,
        };
        if print_disassembly {
            let disassembly_result = cpu.get_next_disassembly();

            cpu.print_disassembly(&disassembly_result);
        }
        cpu.execute_next_instruction();
        if print_registers {
            cpu.register.print_registers();
        }
    }
}

#[cfg(test)]

fn instr_test_setup(code: Vec<u8>, mem_ranges: Option<Vec<RamMemory>>) -> cpu::Cpu {
    use register::ProgramCounter;

    let mut mem_ranges_internal: Vec<Box<dyn Memory>> = Vec::new();
    let code = RamMemory::from_bytes(0xC00000, code);
    let stack = RamMemory::from_range(0x1000000, 0x1000400);
    let cia_memory = CiaMemory::new();
    mem_ranges_internal.push(Box::new(code));
    mem_ranges_internal.push(Box::new(stack));
    mem_ranges_internal.push(Box::new(cia_memory));
    if let Some(mem_ranges) = mem_ranges {
        for mem_range in mem_ranges {
            mem_ranges_internal.push(Box::new(mem_range));
        }
    }
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = mem::Mem::new(mem_ranges_internal, overlay_hack);
    let mut cpu = cpu::Cpu::new(mem);
    cpu.register.reg_pc = ProgramCounter::from_address(0xC00000);
    cpu.register.set_ssp_reg(0x1000400);
    cpu
}
