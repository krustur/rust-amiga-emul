use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::Register,
};

use super::{ConditionalTest, DisassemblyResult, InstructionExecutionResult, OperationSize};

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: TODO
// get_disassembly tests: TODO

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

pub fn get_disassembly<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos_3(instr_word);
    let size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    // let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    // let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);

    let ea_format = Cpu::get_ea_format(ea_mode, ea_register, instr_address + 2, None, reg, mem);

    let (name, num_immediate_words, immediate_data) = match size {
        OperationSize::Byte => (
            String::from("CMPI.B"),
            1,
            format!("#${:02X}", mem.get_unsigned_byte(instr_address + 3)),
        ),
        OperationSize::Word => (
            String::from("CMPI.W"),
            1,
            format!("#${:04X}", mem.get_unsigned_word(instr_address + 2)),
        ),
        OperationSize::Long => (
            String::from("CMPI.L"),
            2,
            format!("#${:08X}", mem.get_unsigned_longword(instr_address + 2)),
        ),
    };

    DisassemblyResult::Done {
        name,
        operands_format: format!("{},{}", immediate_data, ea_format),
        instr_address,
        next_instr_address: instr_address + 2 + (num_immediate_words << 1),
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
