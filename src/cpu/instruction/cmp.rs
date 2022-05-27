use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::Register,
};

use super::{ConditionalTest, DisassemblyResult, InstructionExecutionResult};

const CMP_BYTE: usize = 0b000;
const CMP_WORD: usize = 0b001;
const CMP_LONG: usize = 0b010;
const CMPA_WORD: usize = 0b011;
const CMPA_LONG: usize = 0b111;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
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

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);

    let ea_format = Cpu::get_ea_format(ea_mode, ea_register, instr_address, reg, mem);

    let (name, register_type) = match opmode {
        CMP_BYTE => (String::from("CMP.B"), 'D'),
        CMP_WORD => (String::from("CMP.W"), 'D'),
        CMP_LONG => (String::from("CMP.L"), 'D'),
        CMPA_WORD => (String::from("CMPA.W"), 'A'),
        CMPA_LONG => (String::from("CMPA.L"), 'A'),
        _ => (String::from("unknown CMP"), 'X'),
    };

    DisassemblyResult::Done {
        name,
        operands_format: format!("{},{}{}", ea_format, register_type, register),
        instr_address,
        next_instr_address: instr_address + 2,
    }
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
