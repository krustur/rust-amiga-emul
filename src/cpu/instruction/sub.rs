use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};
use crate::{
    cpu::{instruction::OperationSize, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

const BYTE_WITH_DN_AS_DEST: usize = 0b000;
const WORD_WITH_DN_AS_DEST: usize = 0b001;
const LONG_WITH_DN_AS_DEST: usize = 0b010;
const BYTE_WITH_EA_AS_DEST: usize = 0b100;
const WORD_WITH_EA_AS_DEST: usize = 0b101;
const LONG_WITH_EA_AS_DEST: usize = 0b110;
const WORD_WITH_AN_AS_DEST: usize = 0b011;
const LONG_WITH_AN_AS_DEST: usize = 0b111;

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    let instr_word = pc.peek_next_word(mem);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let operation_size = match opmode {
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        WORD_WITH_AN_AS_DEST => OperationSize::Word,
        LONG_WITH_AN_AS_DEST => OperationSize::Long,
        _ => panic!("Unrecognized opmode"),
    };

    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(operation_size)
        })?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let status_register_result = match opmode {
        BYTE_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, true);
            let reg_value = Cpu::get_byte_from_long(reg.reg_d[register]);
            let result = Cpu::sub_bytes(ea_value, reg_value);

            reg.reg_d[register] = Cpu::set_byte_in_long(result.result, reg.reg_d[register]);
            result.status_register_result
        }
        WORD_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, true);
            let reg_value = Cpu::get_word_from_long(reg.reg_d[register]);
            let result = Cpu::sub_words(ea_value, reg_value);

            reg.reg_d[register] = Cpu::set_word_in_long(result.result, reg.reg_d[register]);
            result.status_register_result
        }
        LONG_WITH_DN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, true);
            let reg_value = reg.reg_d[register];
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.reg_d[register] = result.result;
            result.status_register_result
        }
        BYTE_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_byte(pc, reg, mem, false);
            let reg_value = Cpu::get_byte_from_long(reg.reg_d[register]);
            let result = Cpu::sub_bytes(ea_value, reg_value);
            ea_data.set_value_byte(pc, reg, mem, result.result, true);
            result.status_register_result
        }
        WORD_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, false);
            let reg_value = Cpu::get_word_from_long(reg.reg_d[register]);
            let result = Cpu::sub_words(ea_value, reg_value);
            ea_data.set_value_word(pc, reg, mem, result.result, true);
            result.status_register_result
        }
        LONG_WITH_EA_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, false);
            let reg_value = reg.reg_d[register];
            let result = Cpu::sub_longs(ea_value, reg_value);
            ea_data.set_value_long(pc, reg, mem, result.result, true);
            result.status_register_result
        }
        WORD_WITH_AN_AS_DEST => {
            let ea_value = ea_data.get_value_word(pc, reg, mem, true);
            let ea_value = Cpu::sign_extend_word(ea_value);
            let reg_value = reg.reg_a[register];
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.reg_a[register] = result.result;
            StatusRegisterResult::cleared()
        }
        LONG_WITH_AN_AS_DEST => {
            let ea_value = ea_data.get_value_long(pc, reg, mem, true);
            let reg_value = reg.reg_a[register];
            let result = Cpu::sub_longs(ea_value, reg_value);

            reg.reg_a[register] = result.result;
            StatusRegisterResult::cleared()
        }
        _ => panic!("Unhandled ea_opmode"),
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);

    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let operation_size = match opmode {
        BYTE_WITH_DN_AS_DEST => OperationSize::Byte,
        WORD_WITH_DN_AS_DEST => OperationSize::Word,
        LONG_WITH_DN_AS_DEST => OperationSize::Long,
        BYTE_WITH_EA_AS_DEST => OperationSize::Byte,
        WORD_WITH_EA_AS_DEST => OperationSize::Word,
        LONG_WITH_EA_AS_DEST => OperationSize::Long,
        WORD_WITH_AN_AS_DEST => OperationSize::Word,
        LONG_WITH_AN_AS_DEST => OperationSize::Long,
        _ => panic!("Unrecognized opmode"),
    };

    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(operation_size)
        })?;
    let ea_mode = ea_data.ea_mode;
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    match opmode {
        BYTE_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.B"),
            format!("{},D{}", ea_format, register),
        )),
        WORD_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.W"),
            format!("{},D{}", ea_format, register),
        )),
        LONG_WITH_DN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.L"),
            format!("{},D{}", ea_format, register),
        )),
        BYTE_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.B"),
            format!("D{},{}", register, ea_format),
        )),
        WORD_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.W"),
            format!("D{},{}", register, ea_format),
        )),
        LONG_WITH_EA_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUB.L"),
            format!("D{},{}", register, ea_format),
        )),
        WORD_WITH_AN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUBA.W"),
            format!("{},A{}", ea_format, register),
        )),
        LONG_WITH_AN_AS_DEST => Ok(GetDisassemblyResult::from_pc(
            pc,
            String::from("SUBA.L"),
            format!("{},A{}", ea_format, register),
        )),
        _ => panic!("Unhandled ea_opmode: {}", opmode),
    }
}
