// Path: ..\src\cpu\instruction\gen_tests\add.rs
// This file is autogenerated from tests\add.tests

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
fn address_register_indirect_to_data_register_direct_byte() {
    // arrange - code
    // ADD.B (A0),D0
    let code = [0xD0, 0x10].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00050002 = [0x01].to_vec();
    let arrange_mem_00050002 = RamMemory::from_bytes(0x00050002, arrange_mem_bytes_00050002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00050002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000001, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.B"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000002, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00050002));
}

#[test]
fn address_register_indirect_to_data_register_direct_byte_overflow() {
    // arrange - code
    // ADD.B (A0),D0
    let code = [0xD0, 0x10].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00050002 = [0x01].to_vec();
    let arrange_mem_00050002 = RamMemory::from_bytes(0x00050002, arrange_mem_bytes_00050002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00050002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x0000007f, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.B"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000080, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00050002));
}

#[test]
fn address_register_indirect_to_data_register_direct_byte_carry() {
    // arrange - code
    // ADD.B (A0),D0
    let code = [0xD0, 0x10].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00050002 = [0x01].to_vec();
    let arrange_mem_00050002 = RamMemory::from_bytes(0x00050002, arrange_mem_bytes_00050002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00050002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x000000ff, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.B"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000000, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00050002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_CARRY
    );

    // assert - mem
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00050002));
}

#[test]
fn address_register_indirect_to_data_register_direct_word() {
    // arrange - code
    // ADD.W (A0),D0
    let code = [0xD0, 0x50].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x00, 0x01].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000001, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.W"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000002, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x00, modermodem.mem.get_byte_no_log(0x00060002));
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00060003));
}

#[test]
fn address_register_indirect_to_data_register_direct_word_overflow() {
    // arrange - code
    // ADD.W (A0),D0
    let code = [0xD0, 0x50].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x00, 0x01].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00007fff, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.W"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00008000, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    assert_eq!(0x00, modermodem.mem.get_byte_no_log(0x00060002));
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00060003));
}

#[test]
fn address_register_indirect_to_data_register_direct_word_carry() {
    // arrange - code
    // ADD.W (A0),D0
    let code = [0xD0, 0x50].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x00, 0x01].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x0000ffff, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.W"),
            String::from("(A0),D0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000000, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x0000d7d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_CARRY
    );

    // assert - mem
    assert_eq!(0x00, modermodem.mem.get_byte_no_log(0x00060002));
    assert_eq!(0x01, modermodem.mem.get_byte_no_log(0x00060003));
}

#[test]
fn data_register_direct_to_data_register_direct_long() {
    // arrange - code
    // ADD.L D0,D7
    let code = [0xDE, 0x80].to_vec();
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
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x54324321);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.L"),
            String::from("D0,D7"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x77775555);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn data_register_direct_to_address_register_indirect_long() {
    // arrange - code
    // ADD.L D7,(A1)+
    let code = [0xDF, 0x99].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x11, 0x22, 0x33, 0x44].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x12345678);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x00060002, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.L"),
            String::from("D7,(A1)+"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x12345678);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x23, modermodem.mem.get_byte_no_log(0x00060002));
    assert_eq!(0x56, modermodem.mem.get_byte_no_log(0x00060003));
    assert_eq!(0x89, modermodem.mem.get_byte_no_log(0x00060004));
    assert_eq!(0xbc, modermodem.mem.get_byte_no_log(0x00060005));
}

#[test]
fn data_register_direct_to_address_register_indirect_word() {
    // arrange - code
    // ADD.W D6,(A6)+
    let code = [0xDD, 0x5E].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x11, 0x44].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060002, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x00060002, 0x000000a7);
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
            0x00040002,
            String::from("ADD.W"),
            String::from("D6,(A6)+"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x00060004, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x35, modermodem.mem.get_byte_no_log(0x00060002));
    assert_eq!(0xac, modermodem.mem.get_byte_no_log(0x00060003));
}

#[test]
fn data_register_direct_to_address_register_indirect_byte() {
    // arrange - code
    // ADD.B D2,-(A0)
    let code = [0xD5, 0x20].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00060002 = [0x44].to_vec();
    let arrange_mem_00060002 = RamMemory::from_bytes(0x00060002, arrange_mem_bytes_00060002);

    // arrange - common
    let mut mem = Mem::new(None, None);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00060002)));
    let cpu = Cpu::new(CpuSpeed::NTSC_7_159090_MHz, 0x00000000, 00040000);
    let mut modermodem = Modermodem::new(None, cpu, mem, None, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x00060003, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0x000000a7);
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
            0x00040002,
            String::from("ADD.B"),
            String::from("D2,-(A0)"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00060002, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    assert_eq!(0x8a, modermodem.mem.get_byte_no_log(0x00060002));
}

#[test]
fn immediate_data_to_address_register_direct_word() {
    // arrange - code
    // ADDA.W #$4411,A0
    let code = [0xD0, 0xFC, 0x44, 0x11].to_vec();
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
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0xffffff00, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       0x0000
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040004,
            String::from("ADDA.W"),
            String::from("#$4411,A0"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x00004311, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn immediate_data_to_address_register_direct_long() {
    // arrange - code
    // ADDA.L #$88888888,A7
    let code = [0xDF, 0xFC, 0x88, 0x88, 0x88, 0x88].to_vec();
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
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0xa0a0a0a0, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0x22220000);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       0x0000
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040006,
            String::from("ADDA.L"),
            String::from("#$88888888,A7"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x23451234, 0x000000d1, 0x00000046, 0x000000d3, 0x000000d4, 0x000000d5, 0x00002468, 0x12345678);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0xa0a0a0a0, 0x00060006, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0xa6a6a6a6, 0xaaaa8888);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}
