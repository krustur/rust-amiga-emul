#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

mod cpu;
mod mem;
mod memrange;
mod register;

fn main() {
    // let rom_file_path = "D:\\Amiga\\ROM\\Kickstart 3.1.rom";
    println!("Begin emulation!");
    
    // Incorrect, but let's load the ROM to address 0x0 for now
    let rom_cheat =
        memrange::MemRange::from_file(0x000000, 512 * 1024, rom_file_path)
            .unwrap();
    let rom =
        memrange::MemRange::from_file(0xF80000, 512 * 1024, rom_file_path)
            .unwrap();

    let mut mem_ranges = Vec::new();
    mem_ranges.push(rom_cheat);
    mem_ranges.push(rom);
    let mem = mem::Mem::new(mem_ranges);

    let mut cpu = cpu::Cpu::new(mem);
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.print_registers();
    cpu.execute_next_instruction();
    cpu.print_registers();
    cpu.memory.print_range(0xf80000, 0xf800ff);
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();
    cpu.print_registers();

    println!("End emulation!")
}

#[cfg(test)]
fn instr_test_setup<'a>(code: Vec<u8>) -> cpu::Cpu {
        // TODO: Would be nice to not need the rom cheat
        let rom_cheat =
        memrange::MemRange::from_file(0x000000, 512 * 1024, "D:\\Amiga\\ROM\\Kickstart 3.1.rom")
            .unwrap();
        
        
        let mut mem_ranges = Vec::new();
        let code = memrange::MemRange::from_bytes(0x080000, code);
        mem_ranges.push(rom_cheat);
        mem_ranges.push(code);
        let mem = mem::Mem::new(mem_ranges);
        let mut cpu = cpu::Cpu::new(mem);
        cpu.register.reg_pc = 0x080000;
        cpu
}