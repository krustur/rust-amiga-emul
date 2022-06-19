use crate::{
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};

const CMP_BYTE: usize = 0b000;
const CMP_WORD: usize = 0b001;
const CMP_LONG: usize = 0b010;
const CMPA_WORD: usize = 0b011;
const CMPA_LONG: usize = 0b111;

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
    todo!();
    // let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    // let condition = Cpu::evaluate_condition(reg, &conditional_test);

    // let displacement_8bit = (instr_word & 0x00ff) as i8;

    // let result = match displacement_8bit {
    //     0x00 => todo!("16 bit displacement"),
    //     -1 => todo!("32 bit displacement"), // 0xff
    //     _ => branch_8bit(reg, conditional_test, condition, displacement_8bit), //,
    // };

    // result
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    todo!();
    // let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem);
    // let ea_mode = ea_data.ea_mode;
    // let opmode = Cpu::extract_op_mode_from_bit_pos_6(ea_data.instr_word);
    // let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9);

    // let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);

    // let (name, register_type) = match opmode {
    //     CMP_BYTE => (String::from("CMP.B"), 'D'),
    //     CMP_WORD => (String::from("CMP.W"), 'D'),
    //     CMP_LONG => (String::from("CMP.L"), 'D'),
    //     CMPA_WORD => (String::from("CMPA.W"), 'A'),
    //     CMPA_LONG => (String::from("CMPA.L"), 'A'),
    //     _ => (String::from("unknown CMP"), 'X'),
    // };

    // DisassemblyResult::from_pc(
    //     pc,
    //     name,
    //     format!("{},{}{}", ea_format, register_type, register),
    // )
}

// #[cfg(test)]
// mod tests {
//     use crate::{register::{
//         STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
//         STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
//     }, cpu::instruction::DisassemblyResult};

//     #[test]
//     fn step_bcc_b_when_carry_clear() {
//         // arrange
//         let code = [0x64, 0x02].to_vec(); // BCC.B 2
//         let mut cpu = crate::instr_test_setup(code, None);
//         cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;
//         // act assert - debug
//         let debug_result = cpu.get_next_disassembly();
//         assert_eq!(
//             DisassemblyResult::Done {
//                 name: String::from("BCC.B"),
//                 operands_format: String::from("$02 [$00080004]"),
//                 instr_address: 0x080000,
//                 next_instr_address: 0x080002
//             },
//             debug_result
//         );
//         // act
//         cpu.execute_next_instruction();
//         // assert
//         assert_eq!(0x080004, cpu.register.reg_pc);
//     }

//     #[test]
//     fn step_bcc_b_when_carry_set() {
//         // arrange
//         let code = [0x64, 0x02].to_vec(); // BCC.B 2
//         let mut cpu = crate::instr_test_setup(code, None);
//         cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
//         // act assert - debug
//         let debug_result = cpu.get_next_disassembly();
//         assert_eq!(
//             DisassemblyResult::Done {
//                 name: String::from("BCC.B"),
//                 operands_format: String::from("$02 [$00080004]"),
//                 instr_address: 0x080000,
//                 next_instr_address: 0x080002
//             },
//             debug_result
//         );
//         // act
//         cpu.execute_next_instruction();
//         // assert
//         assert_eq!(0x080002, cpu.register.reg_pc);
//     }
// }
