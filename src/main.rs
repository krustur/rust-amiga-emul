#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use crate::{
    cpu::Cpu,
    mem::{
        ciamemory::CiaMemory, memrange::MemRange, rammemory::RamMemory, rommemory::RomMemory, Mem,
    },
};

mod cpu;
mod mem;
mod register;

// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
// static ROM_FILE_PATH: &str = "C:\\WS\\Amiga\\Kickstart v3.1 rev 40.68 (1993)(Commodore)(A1200).rom";
static ROM_FILE_PATH: &str = "D:\\Amiga\\AmigaOS 3.1.4 for 68k Amiga 1200\\OS314_A1200\\ROMs\\emulation_or_maprom\\kick.a1200.46.143";

fn main() {
    println!("Begin emulation!");

    // Incorrect, but let's load the ROM to address 0x0 for now
    let rom_cheat = RomMemory::from_file(0x000000, ROM_FILE_PATH).unwrap();
    let rom = RomMemory::from_file(0xF80000, ROM_FILE_PATH).unwrap();

    let mut mem_ranges = Vec::new();
    mem_ranges.push(MemRange::from_memory(Box::new(rom_cheat)));
    mem_ranges.push(MemRange::from_memory(Box::new(rom)));

    // Hack for "CDTV & CD32 Extended ROM / A4000 Diagnostics ROM"
    // ROM code checks for $1111 at $00F00000 ()
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
    // let chip_ram = RamMemory::from_range(0x00000000, 0x001FFFFF);
    // mem_ranges.push(MemRange::from_memory(Box::new(chip_ram)));

    let cia_memory = CiaMemory::new();
    mem_ranges.push(MemRange::from_memory(Box::new(cia_memory)));

    let mem = Mem::new(mem_ranges);

    let mut cpu = Cpu::new(mem);
    cpu.memory.print_hex_dump(0xf80000, 0xf800ff);

    cpu.print_registers();

    // let mut disassembly_pc = cpu.register.reg_pc.clone();
    // for i in 0..70 {
    //     let disassembly_result = cpu.get_disassembly(&mut disassembly_pc);

    //     disassembly_pc = ProgramCounter::from_address(disassembly_result.address_next);

    //     cpu.print_disassembly(&disassembly_result);
    // }

    loop {
        // cpu.print_registers();
        let disassembly_result = cpu.get_next_disassembly();
        let pc_address = cpu.register.reg_pc.get_address();
        let print_disassembly = match pc_address {
            0x00F800E2..=0x00F800E8 => false,
            _ => true,
        };
        if print_disassembly {
            cpu.print_disassembly(&disassembly_result);
        }
        cpu.execute_next_instruction();
    }
    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // // cpu.memory.print_range(0xf80000, 0xf800ff);
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // // cpu.memory.print_range(0xf80000, 0xf800ff);
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // cpu.print_registers();
    // cpu.get_next_disassembly();
    // cpu.execute_next_instruction();

    // println!("End emulation!")
}

#[cfg(test)]
fn instr_test_setup<'a>(code: Vec<u8>, mem_range: Option<RamMemory>) -> cpu::Cpu {
    use register::ProgramCounter;

    // TODO: Would be nice to not need the rom cheat
    let rom_cheat = RomMemory::from_file(0x000000, ROM_FILE_PATH).unwrap();

    let mut mem_ranges = Vec::new();
    let code = RamMemory::from_bytes(0xC00000, code);
    mem_ranges.push(MemRange::from_memory(Box::new(rom_cheat)));
    mem_ranges.push(MemRange::from_memory(Box::new(code)));
    if let Some(mem_range) = mem_range {
        mem_ranges.push(MemRange::from_memory(Box::new(mem_range)));
    }
    let mem = mem::Mem::new(mem_ranges);
    let mut cpu = cpu::Cpu::new(mem);
    cpu.register.reg_pc = ProgramCounter::from_address(0xC00000);
    cpu
}
