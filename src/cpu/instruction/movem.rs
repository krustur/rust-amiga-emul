use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, OperationSize,
    StepError,
};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};
use std::collections::BTreeMap;

// Instruction State
// =================
// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

#[derive(Debug, Clone)]
enum MovemDirection {
    MemoryToRegister,
    RegisterToMemory,
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let mut register_list_mask = pc.fetch_next_word(mem);

    let ea_data = pc.get_effective_addressing_data_from_bit_pos(
        instr_word,
        reg,
        mem,
        |instr_word| match instr_word & 0x0040 {
            0x0040 => Ok(OperationSize::Long),
            _ => Ok(OperationSize::Word),
        },
        3,
        0,
    )?;

    let direction = match ea_data.instr_word & 0x0400 {
        0x0400 => MovemDirection::MemoryToRegister,
        _ => MovemDirection::RegisterToMemory,
    };

    match ea_data.ea_mode {
        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
            operation_size,
            ea_register,
        } => {
            // A7 to A0 then D7 to D0
            let a_regs = get_reverse_reg_list_from_mask(&mut register_list_mask);
            let d_regs = get_reverse_reg_list_from_mask(&mut register_list_mask);

            for a in a_regs {
                match ea_data.operation_size {
                    OperationSize::Word => {
                        let value = reg.get_a_reg_word(a);
                        ea_data.set_value_word(pc, reg, mem, value, true);
                    }
                    OperationSize::Long => {
                        let value = reg.get_a_reg_long(a);
                        ea_data.set_value_long(pc, reg, mem, value, true);
                    }
                    _ => panic!(),
                }
            }
            for d in d_regs {
                match ea_data.operation_size {
                    OperationSize::Word => {
                        let value = reg.get_d_reg_word(d);
                        ea_data.set_value_word(pc, reg, mem, value, true);
                    }
                    OperationSize::Long => {
                        let value = reg.get_d_reg_long(d);
                        ea_data.set_value_long(pc, reg, mem, value, true);
                    }
                    _ => panic!(),
                }
            }
        }
        EffectiveAddressingMode::ARegIndirectWithPostIncrement {
            operation_size,
            ea_register,
        } => {
            // D0 to D7 then A0 to A7
            let d_regs = get_reg_list_from_mask(&mut register_list_mask);
            let a_regs = get_reg_list_from_mask(&mut register_list_mask);

            for d in d_regs {
                match ea_data.operation_size {
                    OperationSize::Word => {
                        let value =
                            Cpu::sign_extend_word(ea_data.get_value_word(pc, reg, mem, true));
                        // println!("d{}=${:08X}", reg.reg_d[d], value);
                        reg.set_d_reg_long(d, value);
                    }
                    OperationSize::Long => {
                        let value = ea_data.get_value_long(pc, reg, mem, true);
                        reg.set_d_reg_long(d, value);
                    }
                    _ => panic!(),
                }
            }
            for a in a_regs {
                match ea_data.operation_size {
                    OperationSize::Word => {
                        let value =
                            Cpu::sign_extend_word(ea_data.get_value_word(pc, reg, mem, true));
                        // println!("a{}=${:08X}", reg.reg_a[a], value);
                        reg.set_a_reg_long(a, value);
                    }
                    OperationSize::Long => {
                        let value = ea_data.get_value_long(pc, reg, mem, true);
                        reg.set_a_reg_long(a, value);
                    }
                    _ => panic!(),
                }
            }
        }
        _ => {
            // D0 to D7 then A0 to A7
            let d_regs = get_reg_list_from_mask(&mut register_list_mask);
            let a_regs = get_reg_list_from_mask(&mut register_list_mask);

            match direction {
                MovemDirection::RegisterToMemory => {
                    let mut address = ea_data.get_address(pc, reg, mem);
                    for a in a_regs {
                        match ea_data.operation_size {
                            OperationSize::Word => {
                                let value = reg.get_a_reg_word(a);
                                mem.set_word(address, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            OperationSize::Long => {
                                let value = reg.get_a_reg_long(a);
                                mem.set_long(address, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            _ => panic!(),
                        }
                    }
                    for d in d_regs {
                        match ea_data.operation_size {
                            OperationSize::Word => {
                                let value = reg.get_d_reg_word(d);
                                mem.set_word(address, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            OperationSize::Long => {
                                let value = reg.get_d_reg_long(d);
                                mem.set_long(address, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            _ => panic!(),
                        }
                    }
                }
                MovemDirection::MemoryToRegister => {
                    let mut address = ea_data.get_address(pc, reg, mem);
                    for a in a_regs {
                        match ea_data.operation_size {
                            OperationSize::Word => {
                                let value = Cpu::sign_extend_word(mem.get_word(address));
                                // println!("a{}=${:08X}", reg.reg_a[a], value);
                                reg.set_a_reg_long(a, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            OperationSize::Long => {
                                let value = mem.get_long(address);
                                reg.set_a_reg_long(a, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            _ => panic!(),
                        }
                    }
                    for d in d_regs {
                        match ea_data.operation_size {
                            OperationSize::Word => {
                                let value = Cpu::sign_extend_word(mem.get_word(address));
                                // println!("d{}=${:08X}", reg.reg_d[d], value);
                                reg.set_d_reg_long(d, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            OperationSize::Long => {
                                let value = mem.get_long(address);
                                reg.set_d_reg_long(d, value);
                                (address, _) =
                                    address.overflowing_add(ea_data.operation_size.size_in_bytes());
                            }
                            _ => panic!(),
                        }
                    }
                }
            }
        }
    };

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let mut register_list_mask = pc.fetch_next_word(mem);

    let ea_data = pc.get_effective_addressing_data_from_bit_pos(
        instr_word,
        reg,
        mem,
        |instr_word| match instr_word & 0x0040 {
            0x0040 => Ok(OperationSize::Long),
            _ => Ok(OperationSize::Word),
        },
        3,
        0,
    )?;

    let ea_debug = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), mem);

    let direction = match ea_data.instr_word & 0x0400 {
        0x0400 => MovemDirection::MemoryToRegister,
        _ => MovemDirection::RegisterToMemory,
    };

    // println!("ea_data.ea_mode: {:?}", ea_data.ea_mode);
    // println!("ea_data.operation_size: {:?}", ea_data.operation_size);
    // println!("direction: {:?}", direction);

    let (d_regs, a_regs) = match ea_data.ea_mode {
        EffectiveAddressingMode::ARegIndirectWithPreDecrement {
            operation_size,
            ea_register,
            // ea_address,
        } => {
            // A7 to A0 then D7 to D0
            // println!("2) register_list_mask Ax: ${:04X}", register_list_mask);
            let a_regs = get_reverse_reg_list_from_mask(&mut register_list_mask);
            // println!("2) register_list_mask Dx: ${:04X}", register_list_mask);
            let d_regs = get_reverse_reg_list_from_mask(&mut register_list_mask);
            (d_regs, a_regs)
        }
        _ => {
            // D0 to D7 then A0 to A7
            // println!("3) register_list_mask Dx: ${:04X}", register_list_mask);
            let d_regs = get_reg_list_from_mask(&mut register_list_mask);
            // println!("3) register_list_mask Ax: ${:04X}", register_list_mask);
            let a_regs = get_reg_list_from_mask(&mut register_list_mask);
            (d_regs, a_regs)
        }
    };

    let reg_format = get_reg_format(&d_regs, &a_regs);

    let name = match ea_data.operation_size {
        OperationSize::Long => String::from("MOVEM.L"),
        _ => String::from("MOVEM.W"),
    };

    let operands_format = match direction {
        MovemDirection::MemoryToRegister => format!("{},{}", ea_debug.format, reg_format),
        MovemDirection::RegisterToMemory => format!("{},{}", reg_format, ea_debug.format,),
    };

    Ok(GetDisassemblyResult::from_pc(pc, name, operands_format))
}

fn get_reg_list_from_mask(register_list_mask: &mut u16) -> Vec<usize> {
    let mut res = Vec::new();
    for i in 0..8 {
        let q = (*register_list_mask & 0x0001) != 0;
        if q {
            res.push(i)
        }
        *register_list_mask = (*register_list_mask) >> 1;
    }
    res
}

fn get_reverse_reg_list_from_mask(register_list_mask: &mut u16) -> Vec<usize> {
    let mut res = Vec::new();
    for i in (0..8).rev() {
        let q = (*register_list_mask & 0x0001) != 0;
        if q {
            res.push(i)
        }
        *register_list_mask = (*register_list_mask) >> 1;
    }
    res
}

fn get_reg_format(d_regs: &Vec<usize>, a_regs: &Vec<usize>) -> String {
    let mut d_regs = d_regs.clone();
    let mut a_regs = a_regs.clone();
    d_regs.sort();
    a_regs.sort();

    let d_reg_groups = get_reg_groups(&d_regs);
    let a_reg_groups = get_reg_groups(&a_regs);

    let mut result = String::new();
    let mut add_separator = false;
    for (reg_start, reg_end) in d_reg_groups {
        if add_separator {
            result.push_str("/");
        }
        if reg_start == reg_end {
            result.push_str(&format!("D{}", reg_start));
        } else {
            result.push_str(&format!("D{}-D{}", reg_start, reg_end));
        }
        add_separator = true;
    }

    for (reg_start, reg_end) in a_reg_groups {
        if add_separator {
            result.push_str("/");
        }
        if reg_start == reg_end {
            result.push_str(&format!("A{}", reg_start));
        } else {
            result.push_str(&format!("A{}-A{}", reg_start, reg_end));
        }
        add_separator = true;
    }

    result
}

fn get_reg_groups(regs: &Vec<usize>) -> BTreeMap<usize, usize> {
    let regs_len = regs.len();

    let mut reg_groups = BTreeMap::new();
    let no_prev = 9999;
    let mut grp_start = no_prev;
    let mut prev = no_prev;
    for i in 0..regs_len {
        let this = regs[i];
        if i == 0 || this != prev + 1 {
            grp_start = this;
        }

        if i == regs_len - 1 || (i < regs_len - 1 && regs[i + 1] - 1 != this) {
            let grp_end = this;
            reg_groups.insert(grp_start, grp_end);
        }
        prev = this;
    }

    reg_groups
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        mem::rammemory::RamMemory,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    // long

    #[test]
    fn movem_long_ff00_register_to_memory_address_register_indirect_with_pre_decrement() {
        // arrange
        let code = [0x48, 0xe7, 0xff, 0x00].to_vec(); // MOVEM.L D0-D7,-(A7)
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0x00000000);
        cpu.register.set_d_reg_long(1, 0x11111111);
        cpu.register.set_d_reg_long(7, 0x77777777);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.L"),
                String::from("D0-D7,-(A7)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00000000, cpu.memory.get_long(0x010003e0));
        assert_eq!(0x11111111, cpu.memory.get_long(0x010003e4));
        assert_eq!(0x77777777, cpu.memory.get_long(0x010003fc));
        assert_eq!(0x010003e0, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_long_8182_register_to_memory_address_register_indirect_with_pre_decrement() {
        // arrange
        let code = [0x48, 0xe7, 0x81, 0x82].to_vec(); // MOVEM.L D0/D7/A0/A6,-(A7)
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0xd0d0d0d0);
        cpu.register.set_d_reg_long(7, 0xd7d7d7d7);
        cpu.register.set_a_reg_long(0, 0xa0a0a0a0);
        cpu.register.set_a_reg_long(6, 0xa6a6a6a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.L"),
                String::from("D0/D7/A0/A6,-(A7)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xa6a6a6a6, cpu.memory.get_long(0x010003fc));
        assert_eq!(0xa0a0a0a0, cpu.memory.get_long(0x010003f8));
        assert_eq!(0xd7d7d7d7, cpu.memory.get_long(0x010003f4));
        assert_eq!(0xd0d0d0d0, cpu.memory.get_long(0x010003f0));
        assert_eq!(0x010003f0, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_long_7e7e_memory_to_register_address_register_indirect_with_post_increment() {
        // arrange
        let code = [0x4c, 0xdf, 0x7e, 0x7e].to_vec(); // MOVEM.L (A7)+,D1-D6/A1-A6
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(7, 0x010003d0);
        cpu.memory.set_long(0x010003d0, 0xd1d1d1d1);
        cpu.memory.set_long(0x010003e4, 0xd6d6d6d6);
        cpu.memory.set_long(0x010003e8, 0xa1a1a1a1);
        cpu.memory.set_long(0x010003fc, 0xa6a6a6a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.L"),
                String::from("(A7)+,D1-D6/A1-A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xd1d1d1d1, cpu.register.get_d_reg_long(1));
        assert_eq!(0xd6d6d6d6, cpu.register.get_d_reg_long(6));
        assert_eq!(0xa1a1a1a1, cpu.register.get_a_reg_long(1));
        assert_eq!(0xa6a6a6a6, cpu.register.get_a_reg_long(6));
        assert_eq!(0x01000400, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_long_5bdb_memory_to_register_address_register_indirect_with_post_increment() {
        // arrange
        let code = [0x4c, 0xdf, 0x5b, 0xdb].to_vec(); // MOVEM.L (A7)+,D0-D1/D3-D4/D6-D7/A0-A1/A3-A4/A6
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(7, 0x010003d4);
        cpu.memory.set_long(0x010003d4, 0xd0d0d0d0);
        cpu.memory.set_long(0x010003dc, 0xd3d3d3d3);
        cpu.memory.set_long(0x010003e4, 0xd6d6d6d6);
        cpu.memory.set_long(0x010003ec, 0xa0a0a0a0);
        cpu.memory.set_long(0x010003f4, 0xa3a3a3a3);
        cpu.memory.set_long(0x010003fc, 0xa6a6a6a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.L"),
                String::from("(A7)+,D0-D1/D3-D4/D6-D7/A0-A1/A3-A4/A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xd0d0d0d0, cpu.register.get_d_reg_long(0));
        assert_eq!(0xd3d3d3d3, cpu.register.get_d_reg_long(3));
        assert_eq!(0xd6d6d6d6, cpu.register.get_d_reg_long(6));
        assert_eq!(0xa0a0a0a0, cpu.register.get_a_reg_long(0));
        assert_eq!(0xa3a3a3a3, cpu.register.get_a_reg_long(3));
        assert_eq!(0xa6a6a6a6, cpu.register.get_a_reg_long(6));
        assert_eq!(0x01000400, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_long_0006_register_to_memory_absolut_long() {
        // arrange
        let code = [
            0x48, 0xf9, 0x00, 0x06, 0x00, 0xC0, 0x00, 0x08, /* DC */ 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
        .to_vec(); // MOVEM.L D1-D2,($00C00008).L
                   // DC.L $00000000
                   // DC.L $00000000
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(1, 0xd1d1d1d1);
        cpu.register.set_d_reg_long(2, 0xd2d2d2d2);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("MOVEM.L"),
                String::from("D1-D2,($00C00008).L")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xd1d1d1d1, cpu.memory.get_long(0x00c00008));
        assert_eq!(0xd2d2d2d2, cpu.memory.get_long(0x00c0000c));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_long_0018_memory_to_register_absolut_long() {
        // arrange
        let code = [
            0x4c, 0xf9, 0x00, 0x18, 0x00, 0xC0, 0x00, 0x08, /* DC */ 0xd3, 0xd3, 0xd3, 0xd3,
            0xd4, 0xd4, 0xd4, 0xd4,
        ]
        .to_vec(); // MOVEM.L ($00C00008).L,D3-D4
                   // DC.L $d3d3d3d3
                   // DC.L $d4d4d4d4
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("MOVEM.L"),
                String::from("($00C00008).L,D3-D4")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xd3d3d3d3, cpu.register.get_d_reg_long(3));
        assert_eq!(0xd4d4d4d4, cpu.register.get_d_reg_long(4));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    // word

    #[test]
    fn movem_word_ff00_register_to_memory_address_register_indirect_with_pre_decrement() {
        // arrange
        let code = [0x48, 0xa7, 0xff, 0x00].to_vec(); // MOVEM.W D0-D7,-(A7)
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0x12340000);
        cpu.register.set_d_reg_long(1, 0x87651111);
        cpu.register.set_d_reg_long(7, 0xffff8888);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.W"),
                String::from("D0-D7,-(A7)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000, cpu.memory.get_word(0x010003f0));
        assert_eq!(0x1111, cpu.memory.get_word(0x010003f2));
        assert_eq!(0x8888, cpu.memory.get_word(0x010003fe));
        assert_eq!(0x010003f0, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_word_8182_register_to_memory_address_register_indirect_with_pre_decrement() {
        // arrange
        let code = [0x48, 0xa7, 0x81, 0x82].to_vec(); // MOVEM.W D0/D7/A0/A6,-(A7)
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0xd0d0d0d0);
        cpu.register.set_d_reg_long(7, 0xd7d7d7d7);
        cpu.register.set_a_reg_long(0, 0xa0a0a0a0);
        cpu.register.set_a_reg_long(6, 0xa6a6a6a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.W"),
                String::from("D0/D7/A0/A6,-(A7)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xa6a6, cpu.memory.get_word(0x010003fe));
        assert_eq!(0xa0a0, cpu.memory.get_word(0x010003fc));
        assert_eq!(0xd7d7, cpu.memory.get_word(0x010003fa));
        assert_eq!(0xd0d0, cpu.memory.get_word(0x010003f8));
        assert_eq!(0x010003f8, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_word_7e7e_memory_to_register_address_register_indirect_with_post_increment() {
        // arrange
        let code = [0x4c, 0x9f, 0x7e, 0x7e].to_vec(); // MOVEM.W (A7)+,D1-D6/A1-A6
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(7, 0x010003d0);
        cpu.register.set_d_reg_long(1, 0xffffffff);
        cpu.register.set_d_reg_long(6, 0xffffffff);
        cpu.register.set_a_reg_long(1, 0xffffffff);
        cpu.register.set_a_reg_long(6, 0xffffffff);
        cpu.memory.set_word(0x010003d0, 0xd1d1);
        cpu.memory.set_word(0x010003da, 0x66d6);
        cpu.memory.set_word(0x010003dc, 0x11a1);
        cpu.memory.set_word(0x010003e6, 0xa6a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.W"),
                String::from("(A7)+,D1-D6/A1-A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffd1d1, cpu.register.get_d_reg_long(1));
        assert_eq!(0x000066d6, cpu.register.get_d_reg_long(6));
        assert_eq!(0x000011a1, cpu.register.get_a_reg_long(1));
        assert_eq!(0xffffa6a6, cpu.register.get_a_reg_long(6));
        assert_eq!(0x010003e8, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_word_5bdb_memory_to_register_address_register_indirect_with_post_increment() {
        // arrange
        let code = [0x4c, 0x9f, 0x5b, 0xdb].to_vec(); // MOVEM.W (A7)+,D0-D1/D3-D4/D6-D7/A0-A1/A3-A4/A6
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(7, 0x010003d4);

        cpu.register.set_d_reg_long(0, 0xffffffff);
        cpu.register.set_d_reg_long(3, 0xffffffff);
        cpu.register.set_d_reg_long(6, 0xffffffff);
        cpu.register.set_a_reg_long(0, 0xffffffff);
        cpu.register.set_a_reg_long(3, 0xffffffff);
        cpu.register.set_a_reg_long(6, 0xffffffff);

        cpu.memory.set_word(0x010003d4, 0xd0d0);
        cpu.memory.set_word(0x010003d8, 0x33d3);
        cpu.memory.set_word(0x010003dc, 0xd6d6);
        cpu.memory.set_word(0x010003e0, 0x00a0);
        cpu.memory.set_word(0x010003e4, 0xa3a3);
        cpu.memory.set_word(0x010003e8, 0x06a6);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEM.W"),
                String::from("(A7)+,D0-D1/D3-D4/D6-D7/A0-A1/A3-A4/A6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffd0d0, cpu.register.get_d_reg_long(0));
        assert_eq!(0x000033d3, cpu.register.get_d_reg_long(3));
        assert_eq!(0xffffd6d6, cpu.register.get_d_reg_long(6));
        assert_eq!(0x000000a0, cpu.register.get_a_reg_long(0));
        assert_eq!(0xffffa3a3, cpu.register.get_a_reg_long(3));
        assert_eq!(0x000006a6, cpu.register.get_a_reg_long(6));
        assert_eq!(0x010003ea, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_word_0006_register_to_memory_absolut_long() {
        // arrange
        let code = [
            0x48, 0xb9, 0x00, 0x06, 0x00, 0xC0, 0x00, 0x08, /* DC */ 0x00, 0x00, 0x00, 0x00,
        ]
        .to_vec(); // MOVEM.W D1-D2,($00C00008).L
                   // DC.W $0000
                   // DC.W $0000
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(1, 0xffffd1d1);
        cpu.register.set_d_reg_long(2, 0xffffd2d2);

        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("MOVEM.W"),
                String::from("D1-D2,($00C00008).L")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xd1d1, cpu.memory.get_word(0x00c00008));
        assert_eq!(0xd2d2, cpu.memory.get_word(0x00c0000a));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn movem_word_0018_memory_to_register_absolut_long() {
        // arrange
        let code = [
            0x4c, 0xb9, 0x00, 0x18, 0x00, 0xC0, 0x00, 0x08, /* DC */ 0xd3, 0xd3, 0x44, 0xd4,
        ]
        .to_vec(); // MOVEM.L ($00C00008).L,D3-D4
                   // DC.W $D3D3
                   // DC.L $44D4
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(3, 0xffffffff);
        cpu.register.set_d_reg_long(4, 0xffffffff);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("MOVEM.W"),
                String::from("($00C00008).L,D3-D4")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffd3d3, cpu.register.get_d_reg_long(3));
        assert_eq!(0x000044d4, cpu.register.get_d_reg_long(4));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
