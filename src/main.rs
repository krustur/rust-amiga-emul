#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use mem::memory::Memory;

use crate::{mem::custommemory::CustomMemory, register::{ProgramCounter}, cpu::step_log::{StepLog}};
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

    // NTSC has 262/263 scan lines
    // PAL has 312/313 scan lines
    let mut exec_base :u32= 0xffffffff;
    let mut step_log = StepLog::new();
    // step_log.add_log(StepLogEntry::ReadRegisterLong{register_type: RegisterType::Data, register_index:1, value: 0xdddddd11});
    // step_log.add_log(StepLogEntry::WriteRegisterLong{register_type:RegisterType::Address,register_index:2, value:  0xaaaaaa22});
    // step_log.print_logs();
    // step_log.reset_log();
    // step_log.add_log(StepLogEntry::WriteRegisterLong{register_type:RegisterType::Address,register_index:7, value:  0xaaaaaa77});
    // step_log.print_logs();

    loop {
        let pc_address = cpu.register.reg_pc.get_address();

        let comment = get_3_14_comment(pc_address);
        let no_print_disassembly_before_step = get_3_14_no_print_disassembly_before_step(pc_address);
        let print_registers_after_step = get_3_14_print_registers_after_step(pc_address);
        let (dump_memory_after_step, dump_memory_start, dump_memory_end) = get_3_14_dump_memory_after_step(pc_address);
        let (print_disassembly_after_step, disasm_memory_start, disasm_memory_end) = get_3_14_print_disassembly_after_step(pc_address);

        if let Some(comment) = comment {
            println!("                              ; {}", comment);
        }

        if !no_print_disassembly_before_step {
            let disassembly_result = cpu.get_next_disassembly(&mut step_log);
            cpu.print_disassembly(&disassembly_result, false);
        }
        step_log.reset_log();
        cpu.execute_next_instruction_step_log(&mut step_log);
        if !no_print_disassembly_before_step {
            print!(";");
            step_log.print_logs();
        }
        if cpu.memory.overlay == false {
            let new_exec_base = cpu.memory.get_long_no_log(0x00000004);
            if exec_base != new_exec_base {
                println!("ExecBase changed from ${:08X} to ${:08X}", exec_base, new_exec_base);
                exec_base = new_exec_base;
            }
        }

        if print_registers_after_step {
            cpu.register.print_registers();
        }
        if dump_memory_after_step{
            cpu.memory.print_hex_dump(dump_memory_start, dump_memory_end);
        }
        if print_disassembly_after_step {

            let mut disassembly_pc = ProgramCounter::from_address(disasm_memory_start);
            while disassembly_pc.get_address() <= disasm_memory_end {
                step_log.reset_log();
                let disassembly_result = cpu.get_disassembly(&mut disassembly_pc, &mut step_log);
                cpu.print_disassembly(&disassembly_result, true);
                disassembly_pc = ProgramCounter::from_address(disassembly_result.address_next);
            }
        }
    }
}

fn get_3_14_comment(pc_address: u32) -> Option<String> {
    match pc_address {
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
        0x00F802A8 => Some(String::from("D0=length of memory area")),
        0x00F802AA => Some(String::from("$303=MEMF_PUBLIC|MEMF_CHIP|MEMF_LOCAL|MEMF_24BITDMA")),
        0x00F82214 => Some(String::from("MemHeader.Node.ln_Succ/ln_Pred")),
        0x00F8221A => Some(String::from("MemHeader.Node.ln_Type=10 (NT_MEMORY)")),
        0x00F82220 => Some(String::from("MemHeader.Node.ln_Pri")),
        0x00F82224 => Some(String::from("MemHeader.Node.ln_Name ('chip memory')")),
        0x00F82228 => Some(String::from("MemHeader.mh_Attributes")),
        0x00F8222C => Some(String::from("A1=First MemChunk")),
        0x00F82248 => Some(String::from("MemHeader.mh_First")),
        0x00F8224C => Some(String::from("MemHeader.mh_Lower")),
        0x00F82254 => Some(String::from("MemHeader.mh_Upper")),
        0x00F82258 => Some(String::from("MemHeader.mh_Free")),

        0x00F8225C => Some(String::from("MemChunk.mc_Next")),
        0x00F8225E => Some(String::from("MemChunk.mc_Bytes")),

        0x00F80F16 => Some(String::from("Scan for RomTag structs A4=start address D4=end address")),
        0x00F80F1A => Some(String::from("RomTag matchword ($4AFC)")),

        0x00F80F3A => Some(String::from("RomTag Match found? Check if actually RomTag ..")),
        0x00F80F42 => Some(String::from("Yes, was a RomTag!")),

        0x00F80346 => Some(String::from("Check for expansion RAM in 00C00000 => 00DA0000")),
        0x00F803BA => Some(String::from("RAM expansion check! Will mess with INTENA and UNMAPPED memory if no RAM expansion exists")),
        0x00F80458 => Some(String::from("A4=0, no RAM expansion found!")),

        0x00F83EC6 => Some(String::from("Check if the stack is working by pushing a signature and trying to pop it off again.")),
        0x00F83ED2 => Some(String::from("Test if unrecoverable alter number (high bit)")),
        
        0x00F83F10 => Some(String::from("Unrecoverable crash (stack not working)")),
        0x00F83F1C => Some(String::from("Unrecoverable crash (stack working entry point)")),
        0x00F83F46 => Some(String::from("Flashing LEDs on and off")),
        0x00F83F66 => Some(String::from("Read Serial")),
        0x00F83F6E => Some(String::from("Check if DEL ascii ($7F) received?")),
       
        0x00F84666 => Some(String::from("Call to expansion.library!")),
        
        0x00F81D0E => Some(String::from("ExecLibrary.MakeLibrary done!")),
        0x00F83E8E => Some(String::from("ExecLibrary.Alertish")),

        // ExecLibrary
        0x00F82D2C => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -6  ")),
        0x00F82D34 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -12 ")),
        0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -18 ")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -24 ")),
        // misc
        0x00F80C46 => Some(String::from("ExecLibrary.Supervisor -30 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0386.html")),
        0x00F80C68 => Some(String::from("ExecLibrary.Supervisor (60010+) -30 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0386.html")),
        // special patchable hooks to internal exec activity
        0x00F81456 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -36 ")),
        0x00F81478 => Some(String::from("ExecLibrary._Internal_Scheduling??? -42 ")),
        0x00F829EE => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -48 ")),
        0x00F814D2 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -54 ")),
        0x00F81520 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -60 ")),
        0x00F815C4 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -66 ")),
        // module creation
        0x00F810BA => Some(String::from("ExecLibrary.InitCode -72 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node035B.html")),
        0x00F811B8 => Some(String::from("ExecLibrary.InitStruct -78 ")),
        0x00F81C88 => Some(String::from("ExecLibrary.MakeLibrary -84 library D0= MakeLibrary(vectors A0, structure A1, init A2, dSize D0, segList D1) http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0361.html")),
        0x00F81D10 => Some(String::from("ExecLibrary.MakeFunctions -90 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0360.html")),
        0x00F8108A => Some(String::from("ExecLibrary.FindResident -96 ")),
        0x00F810F2 => Some(String::from("ExecLibrary.InitResident -102 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node035C.html")),
        // diagnostics
        0x00F83E80 => Some(String::from("ExecLibrary.Alert -108")),
        0x00F83248 => Some(String::from("ExecLibrary.Debug -114")),
        // interrupts
        0x00F8196A => Some(String::from("ExecLibrary.Disable -120")),
        0x00F81978 => Some(String::from("ExecLibrary.Enable -126")),
        0x00F82A10 => Some(String::from("ExecLibrary.Forbid -132 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0353.html")),
        0x00F82A18 => Some(String::from("ExecLibrary.Permit -138 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node0224.html")),
        0x00F81706 => Some(String::from("ExecLibrary.SetSR -144")),
        0x00F81722 => Some(String::from("ExecLibrary.SuperState -150")),
        0x00F8174C => Some(String::from("ExecLibrary.UserState -156")),
        0x00F81758 => Some(String::from("ExecLibrary.SetIntVector -162")),
        0x00F8179A => Some(String::from("ExecLibrary.AddIntServer -168")),
        0x00F817D8 => Some(String::from("ExecLibrary.RemIntServer -174")),
        0x00F818AA => Some(String::from("ExecLibrary.Cause -180")),
        // memory allocation
        0x00F81E5A => Some(String::from("ExecLibrary.Allocate -186 memoryBlock D0=Allocate(memHeader A0, byteSize D0) http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node01E5.html")),
        0x00F81D74 => Some(String::from("ExecLibrary.Deallocate -192")),
        0x00F81F5C => Some(String::from("ExecLibrary.AllocMem -198 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0332.html")),
        0x00F8202C => Some(String::from("ExecLibrary.AllocAbs -204 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node01E4.html")),
        0x00F81E1C => Some(String::from("ExecLibrary.FreeMem -210 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0355.html")),
        0x00F820CC => Some(String::from("ExecLibrary.AvailMem -216")),
        0x00F82146 => Some(String::from("ExecLibrary.AllocEntry -222 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node01E6.html")),
        0x00F821CE => Some(String::from("ExecLibrary.FreeEntry -228")),
        // lists
        0x00F81A04 => Some(String::from("ExecLibrary.Insert -234")),
        0x00F81A2C => Some(String::from("ExecLibrary.AddHead -240 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0325.html")),
        0x00F81A3C => Some(String::from("ExecLibrary.AddTail -246")),
        0x00F81A62 => Some(String::from("ExecLibrary.Remove -252 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node022F.html")),
        0x00F81A6E => Some(String::from("ExecLibrary.RemHead -258")),
        0x00F81A7E => Some(String::from("ExecLibrary.RemTail -264")),
        0x00F81A9E => Some(String::from("ExecLibrary.Enqueue -270 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node034D.html")),
        0x00F81ACE => Some(String::from("ExecLibrary.FindName -276")),
        // tasks
        0x00F826C8 => Some(String::from("ExecLibrary.AddTask -282 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node01E2.html")),
        0x00F8277C => Some(String::from("ExecLibrary.RemTask -288")),
        0x00F8280E => Some(String::from("ExecLibrary.FindTask -294")),
        0x00F8286A => Some(String::from("ExecLibrary.SetTaskPri -300")),
        0x00F828C4 => Some(String::from("ExecLibrary.SetSignal -306")),
        0x00F828BA => Some(String::from("ExecLibrary.SetExcept -312")),
        0x00F8296C => Some(String::from("ExecLibrary.Wait -318")),
        0x00F828EA => Some(String::from("ExecLibrary.Signal -324")),
        0x00F82AA0 => Some(String::from("ExecLibrary.AllocSignal -330")),
        0x00F82AD8 => Some(String::from("ExecLibrary.FreeSignal -336")),
        0x00F82A76 => Some(String::from("ExecLibrary.AllocTrap -342")),
        0x00F82A96 => Some(String::from("ExecLibrary.FreeTrap -348")),
        // messages
        0x00F825A4 => Some(String::from("ExecLibrary.AddPort -354")),
        0x00F81A50 => Some(String::from("ExecLibrary.RemPort -360")),
        0x00F825C6 => Some(String::from("ExecLibrary.PutMsg -366")),
        0x00F82650 => Some(String::from("ExecLibrary.GetMsg -372")),
        0x00F825B4 => Some(String::from("ExecLibrary.ReplyMsg -378")),
        0x00F82688 => Some(String::from("ExecLibrary.WaitPort -384")),
        0x00F826B0 => Some(String::from("ExecLibrary.FindPort -390")),
        // libraries
        0x00F81C30 => Some(String::from("ExecLibrary.AddLibrary -396")),
        0x00F81AFC => Some(String::from("ExecLibrary.RemLibrary -402")),
        0x00F81BB6 => Some(String::from("ExecLibrary.OldOpenLibrary -408")),
        0x00F81BEC => Some(String::from("ExecLibrary.CloseLibrary -414")),
        0x00F81C02 => Some(String::from("ExecLibrary.SetFunction -420")),
        0x00F81C3C => Some(String::from("ExecLibrary.SumLibrary -426 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_2._guide/node0384.html")),
        // devices
        0x00F808B4 => Some(String::from("ExecLibrary.AddDevice -432")),
        // 0x00F81AFC => Some(String::from("ExecLibrary.RemDevice -438")),
        0x00F808C0 => Some(String::from("ExecLibrary.OpenDevice -444")),
        0x00F80944 => Some(String::from("ExecLibrary.CloseDevice -450")),
        0x00F8096E => Some(String::from("ExecLibrary.DoIO -456")),
        0x00F8095C => Some(String::from("ExecLibrary.SendIO -462")),
        0x00F809DE => Some(String::from("ExecLibrary.CheckIO -468")),
        0x00F80984 => Some(String::from("ExecLibrary.WaitIO -474")),
        0x00F80A0A => Some(String::from("ExecLibrary.AbortIO -480")),
        // resources
        0x00F826B8 => Some(String::from("ExecLibrary.AddResource -486")),
        // 0x00F81A50 => Some(String::from("ExecLibrary.RemResource -492")),
        0x00F826C0 => Some(String::from("ExecLibrary.OpenResource -498")),
        // private diagnostic support
        0x00F8312C => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -504")),
        0x00F83136 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -510")),
        0x00F8315E => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -516")),
        // misc
        0x00F82BC0 => Some(String::from("ExecLibrary.RawDoFmt -522")),
        //0xXXXXXXXX => Some(String::from("ExecLibrary.GetCC -528")),
        0x00F82008 => Some(String::from("ExecLibrary.TypeOfMem -534")),
        0x00F82DB0 => Some(String::from("ExecLibrary.Procure -540")),
        0x00F82E0E => Some(String::from("ExecLibrary.Vacate -546")),
        0x00F81BBC => Some(String::from("ExecLibrary.OpenLibrary -552")),
        // functions in V33 or higher (Release 1.2)
        // signal semaphores (note funny registers)
        0x00F82E60 => Some(String::from("ExecLibrary.InitSemaphore -558")),
        0x00F82E7C => Some(String::from("ExecLibrary.ObtainSemaphore -564")),
        0x00F82ED2 => Some(String::from("ExecLibrary.ReleaseSemaphore -570")),
        0x00F82FDC => Some(String::from("ExecLibrary.AttemptSemaphore -576")),
        0x00F83008 => Some(String::from("ExecLibrary.ObtainSemaphoreList -582")),
        0x00F8307A => Some(String::from("ExecLibrary.ReleaseSemaphoreList -588")),
        0x00F830A0 => Some(String::from("ExecLibrary.FindSemaphore -594")),
        0x00F8308E => Some(String::from("ExecLibrary.AddSemaphore -600")),
        // 0x00F81A50 => Some(String::from("ExecLibrary.RemSemaphore -606")),
        // kickmem support
        0x00F81008 => Some(String::from("ExecLibrary.SumKickData -612")),
        // more memory support
        0x00F82208 => Some(String::from("ExecLibrary.AddMemList -618")),
        0x00F82D40 => Some(String::from("ExecLibrary.CopyMem -624")),
        0x00F82D3C => Some(String::from("ExecLibrary.CopyMemQuick -630")),
        // cache
        // functions in V36 or higher (Release 2.0)
        0x00F80D60 => Some(String::from("ExecLibrary.CacheClearU -636 http://amigadev.elowar.com/read/ADCD_2.1/Includes_and_Autodocs_3._guide/node01F0.html")),
        // 0x00F80D60 => Some(String::from("ExecLibrary.CacheClearE -642")),
        0x00F80DC6 => Some(String::from("ExecLibrary.CacheControl -648")),
        // misc
        0x00F80A18 => Some(String::from("ExecLibrary.CreateIORequest -654")),
        0x00F80A48 => Some(String::from("ExecLibrary.DeleteIORequest -660")),
        0x00F80A62 => Some(String::from("ExecLibrary.CreateMsgPort -666")),
        0x00F80AB0 => Some(String::from("ExecLibrary.DeleteMsgPort -672")),
        0x00F830D2 => Some(String::from("ExecLibrary.ObtainSemaphoreShared -678")),
        // even more memory support
        0x00F81EBE => Some(String::from("ExecLibrary.AllocVec -684")),
        0x00F81E12 => Some(String::from("ExecLibrary.FreeVec -690")),
        // V39 Pool LVOs
        0x00F82264 => Some(String::from("ExecLibrary.CreatePool -696")),
        0x00F8229A => Some(String::from("ExecLibrary.DeletePool -702")),
        0x00F822C2 => Some(String::from("ExecLibrary.AllocPooled -708")),
        0x00F823C2 => Some(String::from("ExecLibrary.FreePooled -714")),
        // misc
        0x00F830A8 => Some(String::from("ExecLibrary.AttemptSemaphoreShared -720")),
        0x00F80E2E => Some(String::from("ExecLibrary.ColdReboot -726")),
        0x00F81988 => Some(String::from("ExecLibrary.StackSwap -732")),
        
        0x00F82AEE => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -738")),
        // 0x00F82AEE => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -744")),
        // 0x00F82AEE => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -750")),
        // 0x00F82AEE => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -756")),
        // future expansion
        0x00F80DBC => Some(String::from("ExecLibrary.CachePreDMA -762")),
        // 0x00F80D60 => Some(String::from("ExecLibrary.CachePostDMA -768")),
        // functions in V39 or higher (Release 3)
        // Low memory handler functions
        0x00F81D50 => Some(String::from("ExecLibrary.AddMemHandler -774")),
        0x00F81D58 => Some(String::from("ExecLibrary.RemMemHandler -780")),
        // Function to attempt to obtain a Quick Interrupt Vector
        0x00F819DC => Some(String::from("ExecLibrary.ObtainQuickVector -786")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -792")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -798")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -804")),
        0x00F81B62 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -810")),
        0x00F804BA => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -816")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -822")),
        // functions in V45 or higher
        // Finally the list functions are complete
        0x00F819F8 => Some(String::from("ExecLibrary.NewMinList -828")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -834")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -840")),
        // 0x00F82D38 => Some(String::from("ExecLibrary.XXXXXXXXXXXXXXXXXXXXX -846")),
        // New AVL tree support for V45. Yes, this is intentionally part of Exec!
        0x00F82462 => Some(String::from("ExecLibrary.AVL_AddNode -852")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_RemNodeByAddress -858")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_RemNodeByKey -864")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindNode -870")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindPrevNodeByAddress -876")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindPrevNodeByKey -882")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindNextNodeByAddress -888")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindNextNodeByKey -894")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindFirstNode -900")),
        // 0x00F82462 => Some(String::from("ExecLibrary.AVL_FindLastNode -906")),
        // (10 function slots reserved here)

        // expansion.library
        
        0x00F8488C => Some(String::from("ExpansionLibrary.XXXXXXX -162")),
        0x00F8469E => Some(String::from("ExpansionLibrary.XXXXXXX -156")),
        0x00F84AA8 => Some(String::from("ExpansionLibrary.AddDosNode -150")),
        0x00F84998 => Some(String::from("ExpansionLibrary.MakeDosNode -144")),
        0x00F84972 => Some(String::from("ExpansionLibrary.GetCurrentBinding -138")),
        0x00F8496C => Some(String::from("ExpansionLibrary.SetCurrentBinding -132")),
        0x00F8495A => Some(String::from("ExpansionLibrary.ReleaseConfigBinding -126")),
        0x00F84948 => Some(String::from("ExpansionLibrary.ObtainConfigBinding -120")),
        0x00F84866 => Some(String::from("ExpansionLibrary.WriteExpansionByte -114")),
        0x00F847EC => Some(String::from("ExpansionLibrary.RemConfigDev -108")),
        0x00F848AA => Some(String::from("ExpansionLibrary.ReadExpansionRom -102")),
        0x00F8483E => Some(String::from("ExpansionLibrary.ReadExpansionByte -96")),
        0x00F8479A => Some(String::from("ExpansionLibrary.FreeExpansionMem -90")),
        0x00F846BA => Some(String::from("ExpansionLibrary.FreeConfigDev -84")),
        0x00F84780 => Some(String::from("ExpansionLibrary.FreeBoardMem -78")),
        0x00F8480A => Some(String::from("ExpansionLibrary.FindConfigDev -72")),
        0x00F84634 => Some(String::from("ExpansionLibrary.ConfigChain -66")),
        0x00F84360 => Some(String::from("ExpansionLibrary.ConfigBoard -60")),
        0x00F846CC => Some(String::from("ExpansionLibrary.AllocExpansionMem -54")),
        0x00F846A4 => Some(String::from("ExpansionLibrary.AllocConfigDev -48")),
        0x00F8476A => Some(String::from("ExpansionLibrary.AllocBoardMem -42")),
        0x00F84AAA => Some(String::from("ExpansionLibrary.AddBootNode -36")),
        0x00F847D0 => Some(String::from("ExpansionLibrary.AddConfigDev -30")),
        0x00F841B0 => Some(String::from("ExpansionLibrary.XXXXXXX -24")),
        // 0x00F841B0 => Some(String::from("ExpansionLibrary.XXXXXXX -18")),
        0x00F841AC => Some(String::from("ExpansionLibrary.XXXXXXX -12")),
        0x00F841A4 => Some(String::from("ExpansionLibrary.XXXXXXX -6")),

        // utility.library

        0x00F80C2E => Some(String::from("UtilityLibrary.XXXXXXX -312")),             
        0x00F80C2C => Some(String::from("UtilityLibrary.XXXXXXX -306")),             
        0x00F80C2A => Some(String::from("UtilityLibrary.XXXXXXX -300")),             
        0x00F80C28 => Some(String::from("UtilityLibrary.XXXXXXX -294")),             
        0x00F80C26 => Some(String::from("UtilityLibrary.XXXXXXX -288")),             
        0x00F80C24 => Some(String::from("UtilityLibrary.XXXXXXX -282")),             
        0x00F80C22 => Some(String::from("UtilityLibrary.XXXXXXX -276")),             
        0x00F80C20 => Some(String::from("UtilityLibrary.GetUniqueID -270")),             
        0x00F80C1E => Some(String::from("UtilityLibrary.RemNamedObject -264")),             
        0x00F80C1C => Some(String::from("UtilityLibrary.ReleaseNamedObject -258")),             
        0x00F80C1A => Some(String::from("UtilityLibrary.NamedObjectName -252")),             
        0x00F80C18 => Some(String::from("UtilityLibrary.FreeNamedObject -246")),             
        0x00F80C16 => Some(String::from("UtilityLibrary.FindNamedObject -240")),             
        0x00F80C14 => Some(String::from("UtilityLibrary.AttemptRemNamedObject -234")),             
        0x00F80C12 => Some(String::from("UtilityLibrary.AllocNamedObject -228")),
        0x00F80C10 => Some(String::from("UtilityLibrary.AddNamedObject -222")),             
        0x00F80C0E => Some(String::from("UtilityLibrary.UnpackStructureTags -216")),             
        0x00F80C0C => Some(String::from("UtilityLibrary.PackStructureTags -210")),             
        0x00F80C0A => Some(String::from("UtilityLibrary.UMult64 -204")),             
        0x00F80C08 => Some(String::from("UtilityLibrary.SMult64 -198")),             
        0x00F80C06 => Some(String::from("UtilityLibrary.XXXXXXX -192")),             
        0x00F80C04 => Some(String::from("UtilityLibrary.ApplyTagChanges -186")),             
        0x00F80C02 => Some(String::from("UtilityLibrary.ToLower -180")),             
        0x00F80C00 => Some(String::from("UtilityLibrary.ToUpper -174")),             
        0x00F80BFE => Some(String::from("UtilityLibrary.Strnicmp -168")),             
        0x00F80BFC => Some(String::from("UtilityLibrary.Stricmp -162")),             
        0x00F80BFA => Some(String::from("UtilityLibrary.UDivMod32 -156")),             
        0x00F80BF8 => Some(String::from("UtilityLibrary.SDivMod32 -150")),             
        0x00F80BF6 => Some(String::from("UtilityLibrary.UMult32 -144")),             
        0x00F80BF4 => Some(String::from("UtilityLibrary.SMult32 -138")),             
        0x00F80BF2 => Some(String::from("UtilityLibrary.CheckDate -132")),             
        0x00F80BF0 => Some(String::from("UtilityLibrary.Date2Amiga -126")),             
        0x00F80EA6 => Some(String::from("UtilityLibrary.Amiga2Date -120")),             
        0x00F81412 => Some(String::from("UtilityLibrary.XXXXXXX -114")),             
        0x00F813C8 => Some(String::from("UtilityLibrary.XXXXXXX -108")),             
        0x00F81328 => Some(String::from("UtilityLibrary.CallHookPkt -102")),             
        0x00F812D0 => Some(String::from("UtilityLibrary.FilterTagItems -96")),             
        0x00F81290 => Some(String::from("UtilityLibrary.TagInArray -90")),             
        0x00F81238 => Some(String::from("UtilityLibrary.RefreshTagItemClones -84")),             
        0x00F80B82 => Some(String::from("UtilityLibrary.FreeTagItems -78")),             
        0x00F80B80 => Some(String::from("UtilityLibrary.CloneTagItems -72")),             
        0x00F80B7E => Some(String::from("UtilityLibrary.AllocateTagItems -66")),             
        0x00F80B7C => Some(String::from("UtilityLibrary.MapTags -60")),             
        0x00F80B7A => Some(String::from("UtilityLibrary.FilterTagChanges -54")),             
        0x00F80B78 => Some(String::from("UtilityLibrary.NextTagItem -48")),             
        0x00F80B76 => Some(String::from("UtilityLibrary.PackBoolTags -42")),             
        0x00F80B74 => Some(String::from("UtilityLibrary.GetTagData -36")),             
        0x00F80B72 => Some(String::from("UtilityLibrary.FindTagItem -30")),             
        0x00F80B70 => Some(String::from("UtilityLibrary.XXXXXXX -24")),             
        0x00F80B6E => Some(String::from("UtilityLibrary.XXXXXXX -18")),             
        0x00F80B6C => Some(String::from("UtilityLibrary.XXXXXXX -12")),             
        0x00F80B6A => Some(String::from("UtilityLibrary.XXXXXXX -6")),  

        _ => None,
    }
}

fn get_3_14_no_print_disassembly_before_step(pc_address: u32) -> bool {
    match pc_address {
        0x00F800E2..=0x00F800E8 => true, // calculate check sum
        0x00F80F2E..=0x00F80F30 => true, // scan for RomTag
        0x00F81CAE..=0x00F81CB0 => true, // ExecLibrary.MakeLibrary count vectors
        0x00F81FBC..=0x00F81FC4 => true, // ExecLibrary.MakeLibrary clear memory loop
        0x00F83F48..=0x00F83F5c => true, // Flash LEDs loops
        0x00F81D18..=0x00F81D40 => true, // ExecLibrary.MakeFunctions loop
        0x00F804A2..=0x00F804AC => true, // Reset black screen loop
        0x00F80AD8..=0x00F80AF2 => true, // ExecLibrary.FindName loop
        _ => false,
    }
}

fn get_3_14_print_registers_after_step(pc_address: u32) -> bool {
    match pc_address {
        // 0x00F80278 => true, // We now now chip mem (A0=start, A3=end)
        // 0x00F802A8 => true, // D0=length of memory area
        // 0x00F80F16 => true, // scan for RomTag

        // 0x00FC087E => true, // A6=What Library?
        // 0x00FC087E => true, // Library mapping! => 0x0000D09C

        0x00F810F2 => true, // ExecLibrary.InitResident call

        _ => false,
    }
}

fn get_3_14_dump_memory_after_step(pc_address: u32) -> (bool, u32, u32) {
    match pc_address {
        // 0x00F8060C => (true, 0x00f8008d, 0x00f800ad),
        // 0x00F82002 => (true, 0x0000515C, 0x0000516C),
        // 0x00F82002 => (true, 0x00f8008d, 0x00f800ad),

        // 0x00FC087E => (true, 0x0000D09C, 0x0000D09C + 16), // A6=What Library (libbase)?
        // 0x00FC087E => (true, 0x00fc077a, 0x00fc077a + 32), // A6=What Library (name)?
        
        // 0x00F810F2 => (true, 0x00FC0760, 0x00FC0760 + 32), // ExecLibrary.InitResident call A1 mem dump
        0x00F810F2 => (true, 0x00fc077a, 0x00fc077a + 32), // ExecLibrary.InitResident call A1->rt_Name mem dump
        _ => (false, 0, 0)
    }
}

fn get_3_14_print_disassembly_after_step(pc_address: u32) -> (bool, u32, u32) {
    match pc_address {
        // 0x00F82A32 => (true, 0x00004dcc, 0x0000515C),
        // 0x00FC087E => (true, (0x0000D09C-270)-42, 0x0000D09C-6), // Library mapping! => 0x00F8420E
        _ => (false, 0, 0)
    }
}

#[cfg(test)]

fn instr_test_setup(code: Vec<u8>, mem_ranges: Option<Vec<RamMemory>>) -> cpu::Cpu {
    let mut mem_ranges_internal: Vec<Box<dyn Memory>> = Vec::new();
    let code = RamMemory::from_bytes(0x00C00000, code);
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem_ranges_internal.push(Box::new(code));
    mem_ranges_internal.push(Box::new(stack));
    mem_ranges_internal.push(Box::new(vectors));
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
    cpu.register.set_ssp_reg(0x01000400);
    cpu
}
