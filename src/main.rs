#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

mod cpu;
mod mem;
mod instruction;
mod memrange;
mod register;


fn main() {
    println!("Begin emulation!");
    
    // Incorrect, but let's load the ROM to address 0x0 for now
    let rom_cheat = memrange::MemRange::from_file(0x000000, 512*1024, "D:\\Amiga\\ROM\\Kickstart 3.1.rom").unwrap();
    let rom = memrange::MemRange::from_file(0xF80000, 512*1024, "D:\\Amiga\\ROM\\Kickstart 3.1.rom").unwrap();

    let mut mem_ranges = Vec::new();
    mem_ranges.push(&rom_cheat);
    mem_ranges.push(&rom);
    let mem = mem::Mem::new(mem_ranges);

    // println!("Bytes");
    // println!("=====");
    // // println!("0x00: {:#04x}", rom.memory[0]);
    // // println!("0x01: {:#04x}", rom.memory[1]);
    // // println!("0x02: {:#04x}", rom.memory[2]);
    // // println!("0x03: {:#04x}", rom.memory[3]);
    
    // println!("Longwords");
    // println!("=========");
    // println!("0x000000: {:#010x}", rom_cheat.get_longword_unsigned(0x000000));
    // println!("0x000004: {:#010x}", rom_cheat.get_longword_unsigned(0x000004));
    // println!("0x000008: {:#010x}", rom_cheat.get_longword_unsigned(0x000008));
    // println!("0x00000c: {:#010x}", rom_cheat.get_longword_unsigned(0x00000c));
    // println!("0xf80000: {:#010x}", rom.get_longword_unsigned(0xf80000));
    // println!("0xf80004: {:#010x}", rom.get_longword_unsigned(0xf80004));
    // println!("0xf80008: {:#010x}", rom.get_longword_unsigned(0xf80008));
    // println!("0xf8000c: {:#010x}", rom.get_longword_unsigned(0xf8000c));

    let mut cpu = cpu::Cpu::new(mem);
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.print_registers();
    cpu.memory.print_range(0x3d0, 0x41f);
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.execute_instruction();
    cpu.print_registers();


    println!("End emulation!")
}
