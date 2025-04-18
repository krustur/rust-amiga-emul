// Path: ..\src\cpu\instruction\gen_tests\addi.rs
// This file is autogenerated from tests\addi.tests

#![allow(unused_imports)]

use std::cell::RefCell;
use std::rc::Rc;
use crate::register::ProgramCounter;
use crate::mem::rammemory::RamMemory;
use crate::cpu::instruction::GetDisassemblyResult;
use crate::mem::memory::Memory;
use crate::mem::ciamemory::CiaMemory;
use crate::cpu::{Cpu, CpuSpeed};
use crate::mem::Mem;
use crate::modermodem::Modermodem;
use crate::register::STATUS_REGISTER_MASK_CARRY;
use crate::register::STATUS_REGISTER_MASK_EXTEND;
use crate::register::STATUS_REGISTER_MASK_NEGATIVE;
use crate::register::STATUS_REGISTER_MASK_OVERFLOW;
use crate::register::STATUS_REGISTER_MASK_ZERO;


#[test]
fn addi_byte_immediate_data_to_data_register_direct() {
    // arrange - code
    // ADDI.B #$23,D7
    let code = [0x06, 0x07, 0x00, 0x23].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004321);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040004,
            String::from("ADDI.B"),
            String::from("#$23,D7"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn addi_byte_immediate_data_to_absolute_short() {
    // arrange - code
    // ADDI.B #$38,($4000).W
    let code = [0x06, 0x38, 0x00, 0x38, 0x40, 0x00].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0x4C].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00004000)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040006,
            String::from("ADDI.B"),
            String::from("#$38,($4000).W"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    assert_eq!(0x84, modermodem.mem.get_byte_no_log(0x00004000));
}

#[test]
fn addi_word_immediate_data_to_data_register_direct() {
    // arrange - code
    // ADDI.W #$1234,D7
    let code = [0x06, 0x47, 0x12, 0x34].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004321);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040004,
            String::from("ADDI.W"),
            String::from("#$1234,D7"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00005555);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn addi_word_immediate_data_to_absolute_long() {
    // arrange - code
    // ADDI.W #$3878,($00040000).L
    let code = [0x06, 0x79, 0x38, 0x78, 0x00, 0x04, 0x00, 0x00].to_vec();
    let code_memory = RamMemory::from_bytes(0x00030000, code);

    // arrange - mem
    let arrange_mem_bytes_00040000 = [0x3C, 0x09].to_vec();
    let arrange_mem_00040000 = RamMemory::from_bytes(0x00040000, arrange_mem_bytes_00040000);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00040000)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00030000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00030000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00030000,
            0x00030008,
            String::from("ADDI.W"),
            String::from("#$3878,($00040000).L"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x74, modermodem.mem.get_byte_no_log(0x00040000));
    assert_eq!(0x81, modermodem.mem.get_byte_no_log(0x00040001));
}

#[test]
fn addi_long_immediate_data_to_data_register_direct() {
    // arrange - code
    // ADDI.L #$76857685,D0
    let code = [0x06, 0x80, 0x76, 0x85, 0x76, 0x85].to_vec();
    let code_memory = RamMemory::from_bytes(0x00030000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00030000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x10101010, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00030000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00030000,
            0x00030006,
            String::from("ADDI.L"),
            String::from("#$76857685,D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x86958695, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    // -nothing-
}

#[test]
fn addi_long_immediate_data_to_absolute_long() {
    // arrange - code
    // ADDI.L #$38784545,($00040000).L
    let code = [0x06, 0xB9, 0x38, 0x78, 0x45, 0x45, 0x00, 0x04, 0x00, 0x00].to_vec();
    let code_memory = RamMemory::from_bytes(0x00030000, code);

    // arrange - mem
    let arrange_mem_bytes_00040000 = [0xEC, 0x09, 0x00, 0x01].to_vec();
    let arrange_mem_00040000 = RamMemory::from_bytes(0x00040000, arrange_mem_bytes_00040000);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00040000)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00030000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x86958695, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00030000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00030000,
            0x0003000a,
            String::from("ADDI.L"),
            String::from("#$38784545,($00040000).L"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x86958695, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0xdddd5555);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_CARRY
    );

    // assert - mem
    assert_eq!(0x24, modermodem.mem.get_byte_no_log(0x00040000));
    assert_eq!(0x81, modermodem.mem.get_byte_no_log(0x00040001));
    assert_eq!(0x45, modermodem.mem.get_byte_no_log(0x00040002));
    assert_eq!(0x46, modermodem.mem.get_byte_no_log(0x00040003));
}
