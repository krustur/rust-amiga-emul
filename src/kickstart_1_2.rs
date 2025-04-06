use crate::kickstart::Kickstart;
use crate::mem::rommemory::RomMemory;
use crate::mem::Mem;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(non_camel_case_types)]
pub struct Kickstart_1_2 {
    rom_memory: Rc<RefCell<RomMemory>>,
}

impl Kickstart_1_2 {
    pub fn new(file_path: &str, mem: &mut Mem) -> Self {
        let rom_memory = Rc::new(RefCell::new(
            RomMemory::from_file(0xF80000, file_path).unwrap(),
        ));
        let rom_overlay = Rc::new(RefCell::new(RomMemory::from_file(0x000000, file_path).unwrap()));

        // let mut mem = mem.borrow_mut();
        mem.add_range(rom_memory.clone());
        mem.set_overlay(rom_overlay);
        Self { rom_memory }
    }
}

impl Kickstart for Kickstart_1_2 {
    // fn get_rom_memory_range(&self) -> Rc<RefCell<dyn Memory>> {
    //     self.rom_memory.clone()
    // }

    fn get_comment(&self, pc_address: u32) -> Option<String> {
        match pc_address {
            0x00FC00D2 => Some(String::from("We start running here")),
            0x00FC00E2 => Some(String::from("If the ROM is also visible at F00000, or if there is another ROM there, jump there")),
            0x00FC00FE => Some(String::from("Set up port A on the first CIA (8520-A)")),
            0x00FC010E => Some(String::from("Disable interrupts and DMA")),
            0x00FC0124 => Some(String::from("Set a blank, dark gray display")),
            0x00FC0136 => Some(String::from("Set up the Exception Vector Table")),
            0x00FC0148 => Some(String::from("See if the system wants a guru put up after reboot")),
            0x00FC014C => Some(String::from("Check whether there is already a valid ExecBase data structure")),
            0x00FC0172 => Some(String::from("If we get this far, we are reasonably confident that the ExecBase structure is OK, and run the cold start capture code if there is any")),
            0x00FC0184 => Some(String::from("We come here if the cold start capture vector was zero, or upon return from the cold-start capture code.  We continue to verify the ExecBase structure")),
            0x00FC01CE => Some(String::from("If we come here, it was decided that there is no valid ExecBase data structure")),
            0x00FC01D6 => Some(String::from("Now go and check for memory in the $C00000 - $DC0000 area")),
            0x00FC01EE => Some(String::from("The machine has expansion RAM at $C00000.  We put the ExecBase structure there to save chip memory.  This puts it at $C00276")),
            0x00FC01F8 => Some(String::from("Now we clear the expansion memory to zeros")),
            0x00FC0208 => Some(String::from("Having figured out the end address of expansion memory (in A4), and the value to use for ExecBase (in A6), we now check how much chip memory we have")),
            0x00FC0222 => Some(String::from("Clear chip memory")),
            0x00FC0238 => Some(String::from("Since we have found less than 256K of chip memory, some of it must not be working.  Turn the screen bright green, blink the power light, and reset")),
            0x00FC0240 => Some(String::from("We continue here after we've figured out where the chip memory ends (256K or greater) and where the $C00000 memory ends (0 if none present).  The two addresses are in A3 and A4, respectively.")),
            0x00FC025E => Some(String::from("Clear most of the ExecBase structure to zeros")),
            0x00FC027A => Some(String::from("Set up the ExecBase pointer at location 4, and its complement in the ExecBase structure")),
            0x00FC0286 => Some(String::from("Set up the system stack")),
            0x00FC029C => Some(String::from("Store the memory configuration.  Next reset will use this if still intact, and not clear memory")),
            0x00FC02A4 => Some(String::from("Part 2 of the deferred-guru procedure")),
            0x00FC02B0 => Some(String::from("Initialize the exec lists")),
            0x00FC033E => Some(String::from("0x00FC033E = after Initialize the exec lists")),
            0x00FC0384 => Some(String::from("Add expansion memory at $C00000 to the free memory lists")),
            0x00FC03A8 => Some(String::from("Add chip memory to free memory lists. Enter here if there is no expansion memory, and ExecBase therefore resides at the bottom of chip memory")),
            0x00FC03B2 => Some(String::from("Enter here if we do have expansion memory, with D0 and A0 set up as above")),
            0x00FC03CC => Some(String::from("Set the exception vector table up for actual system operation")),
            0x00FC03EC => Some(String::from("Special initialization for machines using a 68010/020")),
            0x00FC0400 => Some(String::from("Fix GetCC() for 68010/020 processors")),
            0x00FC0408 => Some(String::from("Check if we have a 68881 numeric coprocessor, and if so, fix up some more vectors")),
            0x00FC041E => Some(String::from("Regular 68000's continue here")),
            0x00FC0454 => Some(String::from("Now we are going to manufacture the very first task. We use AllocEntry() to obtain a block of memory.  This is then used to hold the MemList from AllocEntry(), the task's stack, and the task descriptor")),
            0x00FC045E => Some(String::from("It's assumed here that the allocated memory follows directly after the MemList.  A safe assumption, since we still have unfragmented memory.  We now create a task descriptor at the top of the allocated memory.  The stack pointer for the task is initialized below the task descriptor")),
            0x00FC048C => Some(String::from("We initialize the task's memory list to empty, then enqueue the MemList holding all this memory there.  This means that when the task dies, the memory will automatically be deallocated")),
            0x00FC04A4 => Some(String::from("Make this the current task, and make it ready to run. initialPC and finalPC are both initialized as zero, but no harm results, since the task can't start running yet")),
            0x00FC04BE => Some(String::from("A historic moment:  We turn the supervisor mode flag off. Starting right now, we are running as a task named 'exec.library', and the multitasking system is operational")),
            0x00FC0500 => Some(String::from("Scan for RomTags, process the KickMemPtr and KickTagPtr variables, and build a table of all the resident modules found. The address of the table of resident modules is stored in the ExecBase data structure")),
            0x00FC0514 => Some(String::from("Handle the 'cool start' capture vector.  Note that if we decided (much) earlier that ExecBase had been clobbered, it will have been rebuilt from scratch, and the cool start capture vector will be zero.  Thus, we don't have to verify it further")),
            0x00FC051E => Some(String::from("Another historic moment.  We call InitCode() to initialize the resident modules.  This is where all the other stuff in the ROMs, stuff in RAM which survived the reboot, etc. comes online.  We indicate that all those modules with the RTF_COLDSTART flag set should be initialized now.")),
            0x00FC0526 => Some(String::from("Yet another capture vector, this time the 'WarmCapture' one")),
            0x00FC0530 => Some(String::from("I assume that when the DOS came online, it took over.  This task looks like it's heading into a dead end. Clear all the CPU registers except for ExecBase and the stack pointer")),
            0x00FC053C => Some(String::from("This is the end of the road")),

            0x00FC0546 => Some(String::from("Determine CPU type and whether FPP is present")),
            0x00FC0592 => Some(String::from("Chip Memory Checking Routine")),
            0x00FC05B4 => Some(String::from("Error System Reset Routine")),
            0x00FC0602 => Some(String::from("Memory Clear Subroutine")),
            0x00FC061A => Some(String::from("$C00000 Expansion RAM Checker")),
            0x00FC0654 => Some(String::from("AddDevice( device ) A1")),

            0x00FC081A => Some(String::from("Handler for reserved exceptions #16-23 and spurious interrupts. These are dead ends (click for Guru)")),
            0x00FC0828 => Some(String::from("Another exception handler.  This one is a dead end also")),
            0x00FC083A => Some(String::from("Handler for bus and address errors (long stack frame). These two can be caught by the task if set up to do so")),
            0x00FC0850 => Some(String::from("Handler for miscellaneous other errors. These can be caught by the task")),
            0x00FC0866 => Some(String::from("Handler for TRAP instructions")),
            0x00FC087C => Some(String::from("Bus error handler for 68010/020 processors")),
            0x00FC0894 => Some(String::from("We get here if various exceptions occurred in user mode")),
            0x00FC0900 => Some(String::from("ROMTAG Scanner and 'KickMemPtr/KickTagPtr' Processor")),
            0x00FC09DE => Some(String::from("RomTag list to resident module table converter")),
            0x00FC0A14 => Some(String::from("KickTagPtr processor")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            // 0x00FC01EE => Some(String::from("XXXXXXXXXX")),
            0x00FC1794 => Some(String::from("ExecLibrary.AllocMem -198")),

            _ => None,
        }
    }

    fn get_no_print_disassembly_before_step(&self, pc_address: u32) -> bool {
        match pc_address {
            0x00FC00DE..=0x00FC00E0 => true, // Start delay loop
            0x00FC060E..=0x00FC0614 => true, // Memory clear loop
            0x00FC0960..=0x00FC0962 => true, // scan for RomTag
            0x00FC150A..=0x00FC1514 => true, // ExecLibrary.MakeLibrary count vectors
            0x00FC157E..=0x00FC15A6 => true, // ExecLibrary.MakeFunctions loop
            _ => false,
        }
    }

    fn get_print_registers_after_step(&self, pc_address: u32) -> bool {
        match pc_address {
            0x00FC0240 => true, // We continue here after we've figured out where the chip memory ends
            _ => false,
        }
    }

    fn get_dump_memory_after_step(&self, pc_address: u32) -> Option<(u32, u32)> {
        match pc_address {
            // 0x00F8060C => (true, 0x00f8008d, 0x00f800ad),
            _ => None
        }
    }

    fn get_dump_areg_memory_after_step(&self, pc_address: u32) -> Option<(usize, u32)> {
        match pc_address {
            // 0x00F81ACE => (true, 1, 32),
            _ => None
        }
    }

    fn get_print_disassembly_after_step(&self, pc_address: u32) -> Option<(u32, u32)> {
        match pc_address {
            // 0x00F82A32 => (true, 0x00004dcc, 0x0000515C),
            _ => None,
        }
    }
}
