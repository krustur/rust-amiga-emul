use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::{Cpu, RotateDirection};
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register
    ,
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Adjust syntax for memory to: LSd <ea>

enum AslrType {
    Register,
    Memory,
}

pub fn match_check_register(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_size000110_from_bit_pos_6(instr_word),
        false => false,
    }
}

pub fn match_check_memory(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_memory_alterable_addressing_modes_pos_0(instr_word),
        false => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let (direction, aslr_type, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Byte,
        ),
        0b001 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Word,
        ),
        0b010 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Long,
        ),
        0b011 => (RotateDirection::Right, AslrType::Memory, OperationSize::Word),
        0b100 => (RotateDirection::Left, AslrType::Register, OperationSize::Byte),
        0b101 => (RotateDirection::Left, AslrType::Register, OperationSize::Word),
        0b110 => (RotateDirection::Left, AslrType::Register, OperationSize::Long),
        _ => (RotateDirection::Left, AslrType::Memory, OperationSize::Word),
    };

    let status_register_result = match aslr_type {
        AslrType::Register => {
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            let shift_count = match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    let shift_count = reg.get_d_reg_long(source_register, step_log) % 64;
                    shift_count
                }
                _ => {
                    let shift_count = ((instr_word & 0x0e00) >> 9).into();
                    let shift_count = match shift_count {
                        1..=7 => shift_count,
                        _ => 8,
                    };
                    shift_count
                }
            };
            match operation_size {
                OperationSize::Byte => {
                    let value = reg.get_d_reg_byte(dest_register, step_log);
                    let (result, status_register_result) =
                        Cpu::shift_byte_arithmetic(value, direction, shift_count);
                    reg.set_d_reg_byte(step_log, dest_register, result);
                    status_register_result
                }
                OperationSize::Word => {
                    let value = reg.get_d_reg_word(dest_register, step_log);
                    let (result, status_register_result) =
                        Cpu::shift_word_arithmetic(value, direction, shift_count);
                    reg.set_d_reg_word(step_log, dest_register, result);
                    status_register_result
                }
                OperationSize::Long => {
                    let value = reg.get_d_reg_long(dest_register, step_log);
                    let (result, status_register_result) =
                        Cpu::shift_long_arithmetic(value, direction, shift_count);
                    reg.set_d_reg_long(step_log, dest_register, result);
                    status_register_result
                }
            }
        }
        AslrType::Memory => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
            )?;

            let value = ea_data.get_value_word(pc, reg, mem, step_log, false);
            let (result, status_register_result) =
                Cpu::shift_word_arithmetic(value, direction, 1);
            ea_data.set_value_word(pc, reg, mem, step_log, result, true);
            status_register_result
        }
    };

    // println!(
    //     "status_register_result: ${:04X}-${:04X}",
    //     status_register_result.status_register, status_register_result.status_register_mask
    // );
    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let (direction, aslr_type, operation_size) = match (instr_word & 0x01c0) >> 6 {
        0b000 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Byte,
        ),
        0b001 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Word,
        ),
        0b010 => (
            RotateDirection::Right,
            AslrType::Register,
            OperationSize::Long,
        ),
        0b011 => (RotateDirection::Right, AslrType::Memory, OperationSize::Word),
        0b100 => (RotateDirection::Left, AslrType::Register, OperationSize::Byte),
        0b101 => (RotateDirection::Left, AslrType::Register, OperationSize::Word),
        0b110 => (RotateDirection::Left, AslrType::Register, OperationSize::Long),
        _ => (RotateDirection::Left, AslrType::Memory, OperationSize::Word),
    };

    match aslr_type {
        AslrType::Register => {
            let dest_register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            match instr_word & 0x0020 {
                0x0020 => {
                    let source_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
                    Ok(GetDisassemblyResult::from_pc(
                        pc,
                        mem,
                        format!(
                            "AS{}.{}",
                            direction.get_format(),
                            operation_size.get_format()
                        ),
                        format!("D{},D{}", source_register, dest_register),
                    ))
                }
                _ => {
                    let count = (instr_word & 0x0e00) >> 9;
                    let count = match count {
                        1..=7 => count,
                        _ => 8,
                    };
                    Ok(GetDisassemblyResult::from_pc(
                        pc,
                        mem,
                        format!(
                            "AS{}.{}",
                            direction.get_format(),
                            operation_size.get_format()
                        ),
                        format!("#${:02X},D{}", count, dest_register),
                    ))
                }
            }
        }
        AslrType::Memory => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
            )?;
            let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                format!(
                    "AS{}.{}",
                    direction.get_format(),
                    operation_size.get_format()
                ),
                format!("{}", ea_format),
            ))
        }
    }
}
