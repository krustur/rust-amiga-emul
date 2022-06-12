#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use crate::{cpu::instruction::DisassemblyResult, register::ProgramCounter};

mod cpu;
mod mem;
mod memrange;
mod register;

// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
// static ROM_FILE_PATH: &str = "C:\\WS\\Amiga\\Kickstart v3.1 rev 40.68 (1993)(Commodore)(A1200).rom";
static ROM_FILE_PATH: &str = "D:\\Amiga\\AmigaOS 3.1.4 for 68k Amiga 1200\\OS314_A1200\\ROMs\\emulation_or_maprom\\kick.a1200.46.143";

fn main() {
    println!("Begin emulation!");

    // Incorrect, but let's load the ROM to address 0x0 for now
    let rom_cheat = memrange::MemRange::from_file(0x000000, 512 * 1024, ROM_FILE_PATH).unwrap();
    let rom = memrange::MemRange::from_file(0xF80000, 512 * 1024, ROM_FILE_PATH).unwrap();

    let mut mem_ranges = Vec::new();
    mem_ranges.push(rom_cheat);
    mem_ranges.push(rom);
    let mem = mem::Mem::new(mem_ranges);

    let mut cpu = cpu::Cpu::new(mem);
    cpu.memory.print_range(0xf80000, 0xf800ff);

    cpu.print_registers();

    let mut disassembly_pc = cpu.register.reg_pc.clone();
    for i in 0..30 {
        let disassembly_result = cpu.get_disassembly(&mut disassembly_pc);
        match &disassembly_result {
            DisassemblyResult::Done {
                address,
                address_next: next_address,
                name,
                operands_format,
                // pc,
                // next_instr_address,
            } => {
                disassembly_pc = ProgramCounter::from_address(*address);
            }
            DisassemblyResult::PassOn => {}
        }
        cpu.print_disassembly(&disassembly_result);
    }

    loop {
        // cpu.print_registers();
        // let disassembly_result = cpu.get_next_disassembly();
        // cpu.print_disassembly(&disassembly_result);
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
fn instr_test_setup<'a>(code: Vec<u8>, mem_range: Option<memrange::MemRange>) -> cpu::Cpu {
    // TODO: Would be nice to not need the rom cheat
    let rom_cheat = memrange::MemRange::from_file(0x000000, 512 * 1024, ROM_FILE_PATH).unwrap();

    let mut mem_ranges = Vec::new();
    let code = memrange::MemRange::from_bytes(0xC00000, code);
    mem_ranges.push(rom_cheat);
    mem_ranges.push(code);
    if let Some(mem_range) = mem_range {
        mem_ranges.push(mem_range);
    }
    let mem = mem::Mem::new(mem_ranges);
    let mut cpu = cpu::Cpu::new(mem);
    cpu.register.reg_pc = ProgramCounter::from_address(0xC00000);
    cpu
}
