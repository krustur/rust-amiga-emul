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
// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 1.2.rom";
// static ROM_FILE_PATH: &str = "D:\\Amiga\\ROM\\Kickstart 2.0.rom";

fn main() {
    println!("Begin emulation!");

    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();

    let rom = RomMemory::from_file(0xF80000, ROM_FILE_PATH).unwrap();
    mem_ranges.push(Box::new(rom));

    // Hack "CDTV & CD32 Extended ROM / A4000 Diagnostics ROM" as RAM
    // ROM code checks for $1111 at 0F00000 ()
    let no_extended_rom_hack = RamMemory::from_range(0x00f00000, 0x00F7FFFF);
    mem_ranges.push(Box::new(no_extended_rom_hack));

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
    println!("Chip memory-string:");
    cpu.memory.print_hex_dump(0x00F803AE, 0x00F803B9);

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
            0x00F8010C => Some(String::from(
                "We're running ROM code at correct location. Access A600 & A1200 IDE controller",
            )),
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
            0x00F801E4 => Some(String::from("Exec base => D1/A6")),
            0x00F801EA => Some(String::from(
                "If Exec Base is at an odd address, go reconfigure memory",
            )),
            0x00F801F0 => Some(String::from(
                "Check ExecBase->ChkBase (system base pointer complement). If not ok, go reconfigure memory",
            )),
            0x00F80232 => Some(String::from("Reconfigure memory")),
            0x00F80234 => Some(String::from("Reconfigure memory")),
            0x00F8023A => Some(String::from("... continue reconfigure memory")),
            0x00F80D50 => Some(String::from("We now know CPU model and if FPU is present")),
            
            
            0x00F8024e => Some(String::from("Check chunks of 16k")),
            0x00F80252 => Some(String::from("Max 2MB of chip memory available")),
            0x00F8025a => Some(String::from("I don't really get these cmp's")),
            0x00F80270 => Some(String::from("Did we 'wrap around' (by writing in shadow memory!)?")),
            0x00F80274 => Some(String::from("Can we read back the value? If not, we didn't write to ram!")),
            0x00F80278 => Some(String::from("We now now chip mem (A0=start, A3=end)")),

            0x00F80282 => Some(String::from("'LOWM' in .$0000.W?")),
            0x00F802FE => Some(String::from("'HELP' in .$0000.W?")),
            
            0x00F802A0 => Some(String::from("Setting up MemHeader for chip memory")),

            // ExecLibrary
            
            0x00004364 => Some(String::from("ExecLibrary.InitCode -72")),
            0x00F810BA => Some(String::from("ExecLibrary.InitCode [code]")),
            0x00004358 => Some(String::from("ExecLibrary.MakeLibrary -84")),
            0x00F81C88 => Some(String::from("ExecLibrary.MakeLibrary [code]")),
            0x00004352 => Some(String::from("ExecLibrary.MakeFunctions -90")),
            0x00F81D10 => Some(String::from("ExecLibrary.MakeFunctions [code]")),
            0x00004346 => Some(String::from("ExecLibrary.InitResident -102")),
            0x00F810F2 => Some(String::from("ExecLibrary.InitResident [code]")),
            0x000042E6 => Some(String::from("ExecLibrary.AllocMem -198")),
            0x00F8060C => Some(String::from("ExecLibrary.AllocMem -198 (B)")),
            0x00F81F5C => Some(String::from("ExecLibrary.AllocMem [code]")),

            0x00005090 => Some(String::from("ExecLibrary.AllocAbs -204")),
            0x00F8202C => Some(String::from("ExecLibrary.AllocAbs [code]")),

            0x000042DA => Some(String::from("ExecLibrary.FreeMem -210")),
            0x00F81E1C => Some(String::from("ExecLibrary.FreeMem [code]")),
            0x00004130 => Some(String::from("ExecLibrary.CacheClearU -636")),
            0x00F80D60 => Some(String::from("ExecLibrary.CacheClearU [code]")),
            
            _ => None,
        };
        if let Some(comment) = comment {
            println!("                              ; {}", comment);
        }

        let print_disassembly_before_step = match pc_address {
            0x00F800E2..=0x00F800E8 => false,
            0x00F80F2E..=0x00F80F30 => false,
            _ => true,
        };
        let print_registers_after_step = match pc_address {
            // 0x00F800EC => true,
            // 0x00F80D50 => true,
            // 0x00F80D52 => true,
            // 0x00F80D5A => true,
            // 0x00F80D5E => true,
            // 0x00F801EA => true,
            // 0x00F80F22 => true,
            // 0x00F80F24 => true,
            // 0x00F80F26 => true,
            0x00F8060C => true,
            0x00F82002=> true,
            _ => false,
        };
        let (dump_memory_after_step, dump_memory_start, dump_memory_end) = match pc_address {
            0x00F8060C => (true, 0x00f8008d, 0x00f800ad),
            // 0x00F82002 => (true, 0x0000515C, 0x0000516C),
            0x00F82002 => (true, 0x00f8008d, 0x00f800ad),
            _ => (false, 0, 0)
        };
        if print_disassembly_before_step {
            let disassembly_result = cpu.get_next_disassembly();
            cpu.print_disassembly(&disassembly_result);
        }
        cpu.execute_next_instruction();
        if print_registers_after_step {
            cpu.register.print_registers();
        }
        if dump_memory_after_step{
            cpu.memory.print_hex_dump(dump_memory_start, dump_memory_end);
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
