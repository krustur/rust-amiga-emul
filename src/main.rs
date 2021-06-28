mod cpu;
mod mem;

use cpu::Cpu;
use mem::Mem;

fn main() {
    let width: u32 = 2560;

    println!("Begin emulation!");
    
    let rom_result = mem::Mem::from_file("D:\\Amiga\\ROM\\Kickstart 3.1.rom");
    let rom = rom_result.unwrap();

    println!("0: {:#04x}", rom.memory[0]);
    println!("1: {:#04x}", rom.memory[1]);
    println!("2: {:#04x}", rom.memory[2]);
    println!("3: {:#04x}", rom.memory[3]);
    
    let cpu = cpu::Cpu::new(rom);

    println!("End emulation!")
}
