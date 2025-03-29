// Path: ..\src\cpu\instruction\gen_tests\addq.rs
// This file is autogenerated from tests\addq.tests

#![allow(unused_imports)]

use crate::register::ProgramCounter;
use crate::mem::rammemory::RamMemory;
use crate::cpu::instruction::GetDisassemblyResult;
use crate::mem::memory::Memory;
use crate::mem::ciamemory::CiaMemory;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::STATUS_REGISTER_MASK_CARRY;
use crate::register::STATUS_REGISTER_MASK_EXTEND;
use crate::register::STATUS_REGISTER_MASK_NEGATIVE;
use crate::register::STATUS_REGISTER_MASK_OVERFLOW;
use crate::register::STATUS_REGISTER_MASK_ZERO;


#[test]
fn addq_data_to_data_register_direct_byte() {
    // arrange - code
    // ADDQ.B #$5,(A0)+
    let code = [0x5A, 0x18].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0x10].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.B"),
            String::from("#$5,(A0)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00004001, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x15, cpu.memory.get_byte_no_log(0x00004000));
}

#[test]
fn addq_data_to_data_register_direct_byte_overflow() {
    // arrange - code
    // ADDQ.B #$5,(A0)+
    let code = [0x5A, 0x18].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0x7E].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.B"),
            String::from("#$5,(A0)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00004001, 0x000000a1, 0x000000a2, 0x000000a3, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
    );

    // assert - mem
    assert_eq!(0x83, cpu.memory.get_byte_no_log(0x00004000));
}

#[test]
fn addq_data_to_data_register_direct_word() {
    // arrange - code
    // ADDQ.W #$8,(A3)+
    let code = [0x50, 0x5B].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0x60, 0x20].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004000, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.W"),
            String::from("#$8,(A3)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004002, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x60, cpu.memory.get_byte_no_log(0x00004000));
    assert_eq!(0x28, cpu.memory.get_byte_no_log(0x00004001));
}

#[test]
fn addq_data_to_data_register_direct_word_carry() {
    // arrange - code
    // ADDQ.W #$3,(A3)+
    let code = [0x56, 0x5B].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0xFF, 0xFE].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004000, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.W"),
            String::from("#$3,(A3)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004002, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_CARRY
    );

    // assert - mem
    assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00004000));
    assert_eq!(0x01, cpu.memory.get_byte_no_log(0x00004001));
}

#[test]
fn addq_data_to_data_register_direct_word_negative() {
    // arrange - code
    // ADDQ.W #$3,(A3)+
    let code = [0x56, 0x5B].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0xFF, 0xF0].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004000, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.W"),
            String::from("#$3,(A3)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00004000, 0x000000a1, 0x000000a2, 0x00004002, 0x000000a4, 0x000000a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_NEGATIVE
    );

    // assert - mem
    assert_eq!(0xff, cpu.memory.get_byte_no_log(0x00004000));
    assert_eq!(0xf3, cpu.memory.get_byte_no_log(0x00004001));
}

#[test]
fn addq_data_to_data_register_direct_long() {
    // arrange - code
    // ADDQ.L #$1,(A5)+
    let code = [0x52, 0x9D].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0x60, 0x70, 0x80, 0x20].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0xa0a0a0a0, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0x00004000, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.L"),
            String::from("#$1,(A5)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0xa0a0a0a0, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0x00004004, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    assert_eq!(0x60, cpu.memory.get_byte_no_log(0x00004000));
    assert_eq!(0x70, cpu.memory.get_byte_no_log(0x00004001));
    assert_eq!(0x80, cpu.memory.get_byte_no_log(0x00004002));
    assert_eq!(0x21, cpu.memory.get_byte_no_log(0x00004003));
}

#[test]
fn addq_data_to_data_register_direct_long_zero() {
    // arrange - code
    // ADDQ.L #$8,(A5)+
    let code = [0x50, 0x9D].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    let arrange_mem_bytes_00004000 = [0xFF, 0xFF, 0xFF, 0xF8].to_vec();
    let arrange_mem_00004000 = RamMemory::from_bytes(0x00004000, arrange_mem_bytes_00004000);

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    mem_ranges.push(Box::new(arrange_mem_00004000));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0xa0a0a0a0, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0x00004000, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_NEGATIVE
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_OVERFLOW
       | STATUS_REGISTER_MASK_CARRY
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.L"),
            String::from("#$8,(A5)+"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0xa0a0a0a0, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0x00004004, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       STATUS_REGISTER_MASK_EXTEND
       | STATUS_REGISTER_MASK_ZERO
       | STATUS_REGISTER_MASK_CARRY
    );

    // assert - mem
    assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00004000));
    assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00004001));
    assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00004002));
    assert_eq!(0x00, cpu.memory.get_byte_no_log(0x00004003));
}

#[test]
fn addq_data_to_address_register_direct_word() {
    // arrange - code
    // ADDQ.W #$8,A0
    let code = [0x50, 0x48].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0xfffffffe, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0xa5a5a5a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       0x0000
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.W"),
            String::from("#$8,A0"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0x00000006, 0xa1a1a1a1, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0xa5a5a5a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}

#[test]
fn addq_data_to_address_register_direct_long() {
    // arrange - code
    // ADDQ.L #$8,A1
    let code = [0x50, 0x89].to_vec();
    let code_memory = RamMemory::from_bytes(0x00040000, code);

    // arrange - mem
    // -nothing-

    // arrange - common
    let stack = RamMemory::from_range(0x01000000, 0x010003ff);
    let vectors = RamMemory::from_range(0x00000000, 0x000003ff);
    let cia_memory = CiaMemory::new();
    let mut mem_ranges: Vec<Box<dyn Memory>> = Vec::new();
    mem_ranges.push(Box::new(code_memory));
    mem_ranges.push(Box::new(stack));
    mem_ranges.push(Box::new(vectors));
    mem_ranges.push(Box::new(cia_memory));
    let overlay_hack = Box::new(RamMemory::from_range(0xffffffff, 0xffffffff));
    let mem = Mem::new(mem_ranges, overlay_hack);
    let mut cpu = Cpu::new(mem);

    // arrange - regs
    cpu.register.set_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.set_all_a_reg_long_no_log(0xa0a0a0a0, 0xfffffffe, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0xa5a5a5a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_pc = ProgramCounter::from_address(0x00040000);
    cpu.register.set_ssp_reg(0x01000400);
    cpu.register.reg_sr.set_sr_reg_flags_abcde(
       0x0000
    );

    // act/assert - disassembly
    let get_disassembly_result = cpu.get_next_disassembly_no_log();
    assert_eq!(
        GetDisassemblyResult::from_address_and_address_next(
            0x00040000,
            0x00040002,
            String::from("ADDQ.L"),
            String::from("#$8,A1"),
            ),
            get_disassembly_result
        );

    // act
    cpu.execute_next_instruction();

    // assert - regs
    cpu.register.assert_all_d_reg_long_no_log(0x000000d0, 0x000000d1, 0x000000d2, 0x000000d3, 0x000000d4, 0x000000d5, 0x000000d6, 0x00004344);
    cpu.register.assert_all_a_reg_long_no_log(0xa0a0a0a0, 0x00000006, 0x000000a2, 0xa3a3a3a3, 0xa4a4a4a4, 0xa5a5a5a5, 0x000000a6, 0x000000a7);
    cpu.register.reg_sr.assert_sr_reg_flags_abcde(
       0x0000
    );

    // assert - mem
    // -nothing-
}
