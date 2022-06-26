use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};
use crate::{
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

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    todo!("SUBQ common_exec_func");
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
    todo!("SUBQ common_exec_func");
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
