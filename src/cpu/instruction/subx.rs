use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register, RegisterType},
};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    todo!("SUBX step");
    // TODO: Tests
    // let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    // reg.reg_a[register] = ea;
    // let instr_comment = format!("subtracting {:#010x} into A{}", ea, register);
    // InstructionExecutionResult {
    //     name: String::from("LEA"),
    //     operands_format: format!("{},A{}", ea_format, register),
    //     comment: instr_comment,
    //     op_size: OperationSize::Long,
    //     pc_result: PcResult::Increment,
    // }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.fetch_next_word(mem);
    let register_type = match instr_word & 0x0008 {
        0x0008 => RegisterType::Address,
        _ => RegisterType::Data,
    };

    let source_register_index = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
    let destination_register_index = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    match register_type {
        RegisterType::Data => Ok(GetDisassemblyResult::from_pc(
            pc,
            format!("SUBX.{}", operation_size.get_format(),),
            format!(
                "{}{},{}{}",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        )),
        RegisterType::Address => Ok(GetDisassemblyResult::from_pc(
            pc,
            format!("SUBX.{}", operation_size.get_format(),),
            format!(
                "-({}{}),-({}{})",
                register_type.get_format(),
                source_register_index,
                register_type.get_format(),
                destination_register_index,
            ),
        )),
    }
}
