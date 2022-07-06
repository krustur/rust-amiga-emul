use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, OperationSize,
    StepError, StepResult,
};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};
use std::collections::BTreeMap;

// Instruction State
// =================
// step: TODO
// step cc: DONE (not affected)
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

#[derive(Debug, Clone)]
enum MovemDirection {
    MemoryToRegister,
    RegisterToMemory,
}

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            match instr_word & 0x0040 {
                0x0040 => Ok(OperationSize::Long),
                _ => Ok(OperationSize::Word),
            }
        })?;

    let direction = match ea_data.instr_word & 0x0400 {
        0x0400 => MovemDirection::MemoryToRegister,
        _ => MovemDirection::RegisterToMemory,
    };

    let mut register_list_mask = pc.fetch_next_word(mem);

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

    for d in d_regs {
        let reg_d = reg.reg_d[d];

        println!("pushing {}={} to stack ${:08X}", d, reg_d, reg.reg_a[7] - 4);
        ea_data.set_value_long(pc, reg, mem, reg_d, true);
    }

    Ok(StepResult::Done {})

    // todo!();
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            match instr_word & 0x0040 {
                0x0040 => Ok(OperationSize::Long),
                _ => Ok(OperationSize::Word),
            }
        })?;

    let ea_debug = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), reg, mem);

    let direction = match ea_data.instr_word & 0x0400 {
        0x0400 => MovemDirection::MemoryToRegister,
        _ => MovemDirection::RegisterToMemory,
    };

    let mut register_list_mask = pc.fetch_next_word(mem);

    // println!("ea_data.ea_mode: {:?}", ea_data.ea_mode);
    // println!("ea_data.operation_size: {:?}", ea_data.operation_size);
    // println!("direction: {:?}", direction);

    let (d_regs, a_regs) = match ea_data.ea_mode {
        // EffectiveAddressingMode::ARegIndirectWithPostIncrement {
        //     ea_register,
        //     ea_address,
        // } => {
        //     // D0 to D7 then A0 to A7
        //     // println!("1) register_list_mask Dx: ${:04X}", register_list_mask);
        //     let d_regs = get_reg_list_from_mask(&mut register_list_mask);
        //     // println!("1) register_list_mask Ax: ${:04X}", register_list_mask);
        //     let a_regs = get_reg_list_from_mask(&mut register_list_mask);
        //     (d_regs, a_regs)
        // }
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
        // println!("*register_list_mask before: ${:04X}", *register_list_mask);
        let q = (*register_list_mask & 0x0001) != 0;
        if q {
            res.push(i)
        }
        *register_list_mask = (*register_list_mask) >> 1;
        // println!("*register_list_mask after: ${:04X}", *register_list_mask);
    }
    res
}

fn get_reg_format(d_regs: &Vec<usize>, a_regs: &Vec<usize>) -> String {
    let mut d_regs = d_regs.clone();
    let mut a_regs = a_regs.clone();
    d_regs.sort();
    a_regs.sort();

    for x in d_regs.clone() {
        println!("D{}", x);
    }
    for x in a_regs.clone() {
        println!("A{}", x);
    }
    let d_reg_groups = get_reg_groups(&d_regs);
    let a_reg_groups = get_reg_groups(&a_regs);
    // for (x, y) in d_reg_groups.clone() {
    //     println!("D{}-D{}", x, y);
    // }
    // for (x, y) in a_reg_groups.clone() {
    //     println!("A{}-A{}", x, y);
    // }

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

    // 0 1 2 3 7

    let mut reg_groups = BTreeMap::new();
    let no_prev = 9999;
    let mut grp_start = no_prev;
    let mut prev = no_prev;
    for i in 0..regs_len {
        let this = regs[i];
        if i == 0 || this != prev + 1 {
            grp_start = this;
            // println!("found new group start: {}", grp_start);
        }

        if i == regs_len - 1 || (i < regs_len - 1 && regs[i + 1] - 1 != this) {
            let grp_end = this;
            // println!("found new group end: {}", grp_end);
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
            ProgramCounter, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
            STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW,
            STATUS_REGISTER_MASK_ZERO,
        },
    };

    // TODO: tests
    // 8181
    // 8000
    // 0100
    // 0080
    // 0001
    // 7ffe
    // aaaa
    #[test]
    fn data_reg_direct_to_absolute_long_addressing_mode() {
        // arrange
        let code = [0x48, 0xe7, 0xff, 0x00].to_vec(); // MOVEM.L D0-D7,-(A7)
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.reg_d[0] = 0x00000000;
        cpu.register.reg_d[1] = 0x11111111;
        cpu.register.reg_d[7] = 0x77777777;

        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
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
        assert_eq!(0x010003e0, cpu.register.reg_a[7]);
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }
}
