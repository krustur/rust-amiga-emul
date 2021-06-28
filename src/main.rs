mod cpu;
mod mem;

fn main() {
    println!("Begin emulation!");
    
    // Incorrect, but let's load the ROM to address 0x0 for now
    let rom_result = mem::Mem::from_file(0x0, "D:\\Amiga\\ROM\\Kickstart 3.1.rom");
    let rom = rom_result.unwrap();

    println!("Bytes");
    println!("=====");
    println!("0x00: {:#04x}", rom.memory[0]);
    println!("0x01: {:#04x}", rom.memory[1]);
    println!("0x02: {:#04x}", rom.memory[2]);
    println!("0x03: {:#04x}", rom.memory[3]);
    
    println!("Longwords");
    println!("=========");
    println!("0x00: {:#010x}", rom.get_longword_unsigned(0));
    println!("0x04: {:#010x}", rom.get_longword_unsigned(4));
    println!("0x08: {:#010x}", rom.get_longword_unsigned(8));
    println!("0x0c: {:#010x}", rom.get_longword_unsigned(12));

    // let cpu = cpu::Cpu::new(rom);

    println!("End emulation!")
}
