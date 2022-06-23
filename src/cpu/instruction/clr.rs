use super::{
    GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError, StepResult,
};
use crate::{
    cpu::Cpu,
    memhandler::MemHandler,
    register::{ProgramCounter, Register},
};

// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO
pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut MemHandler,
) -> Result<StepResult, StepError> {
    let instr_word = pc.peek_next_word(mem);
    let size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, Some(size))?;

    match size {
        OperationSize::Byte => ea_data.set_value_byte(pc, reg, mem, 0x00, true),
        OperationSize::Word => ea_data.set_value_word(pc, reg, mem, 0x0000, true),
        OperationSize::Long => ea_data.set_value_long(pc, reg, mem, 0x00000000, true),
    };
    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &MemHandler,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
    let size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, Some(size))?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(size), reg, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from(format!("CLR.{}", size.get_format())),
        ea_format.format,
    ))
}
