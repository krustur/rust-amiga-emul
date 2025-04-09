// Path: ..\src\cpu\instruction\gen_tests\divu.rs
// This file is autogenerated from tests\divu.tests

#![allow(unused_imports)]

use std::cell::RefCell;
use std::rc::Rc;
use crate::register::ProgramCounter;
use crate::mem::rammemory::RamMemory;
use crate::cpu::instruction::GetDisassemblyResult;
use crate::mem::memory::Memory;
use crate::mem::ciamemory::CiaMemory;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::modermodem::Modermodem;
use crate::register::STATUS_REGISTER_MASK_CARRY;
use crate::register::STATUS_REGISTER_MASK_EXTEND;
use crate::register::STATUS_REGISTER_MASK_NEGATIVE;
use crate::register::STATUS_REGISTER_MASK_OVERFLOW;
use crate::register::STATUS_REGISTER_MASK_ZERO;


#[test]
fn divu_w_x_not_affected_still_set() {
    // arrange - code
    // DIVU.W (A5),D5
    let code = [0x8A, 0xD5].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00030000 = [0x00, 0x05].to_vec();
    let arrange_mem_00030000 = RamMemory::from_bytes(0x00030000, arrange_mem_bytes_00030000);

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00030000)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00000100, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
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
            String::from("DIVU.W"),
            String::from("(A5),D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00010033, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_x_not_affected_still_clear() {
    // arrange - code
    // DIVU.W (A5),D5
    let code = [0x8A, 0xD5].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00030000 = [0x00, 0x05].to_vec();
    let arrange_mem_00030000 = RamMemory::from_bytes(0x00030000, arrange_mem_bytes_00030000);

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    mem.add_range(Rc::new(RefCell::new(arrange_mem_00030000)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00000100, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
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
            String::from("DIVU.W"),
            String::from("(A5),D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00010033, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_n_set() {
    // arrange - code
    // DIVU.W #$0002,D5
    let code = [0x8A, 0xFC, 0x00, 0x02].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00010000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040004,
            String::from("DIVU.W"),
            String::from("#$0002,D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00008000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_n_not_set() {
    // arrange - code
    // DIVU.W #$0002,D5
    let code = [0x8A, 0xFC, 0x00, 0x02].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x0000ffff, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
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
            String::from("DIVU.W"),
            String::from("#$0002,D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00017fff, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_z_set() {
    // arrange - code
    // DIVU.W #$0002,D5
    let code = [0x8A, 0xFC, 0x00, 0x02].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00000000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040004,
            String::from("DIVU.W"),
            String::from("#$0002,D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00000000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_ZERO
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_z_not_set() {
    // arrange - code
    // DIVU.W #$0002,D5
    let code = [0x8A, 0xFC, 0x00, 0x02].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00000003, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
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
            String::from("DIVU.W"),
            String::from("#$0002,D5"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x000000d3, 0x000000d4, 0x00010001, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_v_set() {
    // arrange - code
    // DIVU.W D0,D3
    let code = [0x86, 0xC0].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x00400000, 0x000000d4, 0x00000000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       0x0000
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("DIVU.W"),
            String::from("D0,D3"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000040, 0x000000d1, 0x555555d2, 0x00400000, 0x000000d4, 0x00000000, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_v_not_set() {
    // arrange - code
    // DIVU.W D0,D3
    let code = [0x86, 0xC0].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000041, 0x000000d1, 0x555555d2, 0x00400000, 0x000000d4, 0x00010001, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    modermodem.cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = modermodem.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("DIVU.W"),
            String::from("D0,D3"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000041, 0x000000d1, 0x555555d2, 0x0031fc0f, 0x000000d4, 0x00010001, 0x000000d6, 0x000000d7);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
    );

    // assert - mem
    // -nothing-
}

#[test]
fn divu_w_c_not_set() {
    // arrange - code
    // DIVU.W D7,D7
    let code = [0x8E, 0xC7].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let mut mem = Mem::new();
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    mem.add_range(Rc::new(RefCell::new(code_memory)));
    mem.add_range(Rc::new(RefCell::new(vectors)));
    mem.add_range(Rc::new(RefCell::new(cia_memory)));
    let cpu = Cpu::new(&mem);
    let mut modermodem = Modermodem::new(None, cpu, mem, None);

    // arrange - regs
    modermodem.cpu.register.set_all_d_reg_long_no_log(0x00000041, 0x000000d1, 0x555555d2, 0x00400000, 0x000000d4, 0x00010001, 0x000000d6, 0x00000001);
    modermodem.cpu.register.set_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
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
            String::from("DIVU.W"),
            String::from("D7,D7"),
            ),
            get_disassembly_result
        );

    // act
    modermodem.step();

    // assert - regs
    modermodem.cpu.register.assert_all_d_reg_long_no_log(0x00000041, 0x000000d1, 0x555555d2, 0x00400000, 0x000000d4, 0x00010001, 0x000000d6, 0x00000001);
    modermodem.cpu.register.assert_all_a_reg_long_no_log(0x000000a0, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x00030000, 0x000000a6, 0x000000a7);
    modermodem.cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
    );

    // assert - mem
    // -nothing-
}
