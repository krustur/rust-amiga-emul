use crate::{
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};

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
    pc.skip_word();
    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    pc.skip_word();

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("NOP"),
        String::from(""),
    ))
}

// #[cfg(test)]
// mod tests {
//     use crate::{register::{
//         STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
//         STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
//     }, cpu::instruction::DisassemblyResult, memrange::MemRange};

//     #[test]
//     fn absolute_short_addressing_mode_to_a0() {
//         // arrange
//         let code = [0x41, 0xf8, 0x05, 0x00].to_vec(); // LEA ($0500).W,A0
//         let mut cpu = crate::instr_test_setup(code, None);
//         cpu.register.reg_a[0] = 0x00000000;
//         cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
//             | STATUS_REGISTER_MASK_OVERFLOW
//             | STATUS_REGISTER_MASK_ZERO
//             | STATUS_REGISTER_MASK_NEGATIVE
//             | STATUS_REGISTER_MASK_EXTEND;
//         // act assert - debug
//         let debug_result = cpu.get_next_disassembly();
//         assert_eq!(
//             DisassemblyResult::Done {
//                 name: String::from("LEA"),
//                 operands_format: String::from("($0500).W,A0"),
//                 instr_address: 0x080000,
//                 next_instr_address: 0x080004
//             },
//             debug_result
//         );
//         // act
//         cpu.execute_next_instruction();
//         // assert
//         assert_eq!(0x500, cpu.register.reg_a[0]);
//         assert_eq!(true, cpu.register.is_sr_carry_set());
//         assert_eq!(true, cpu.register.is_sr_coverflow_set());
//         assert_eq!(true, cpu.register.is_sr_zero_set());
//         assert_eq!(true, cpu.register.is_sr_negative_set());
//         assert_eq!(true, cpu.register.is_sr_extend_set());
//     }

//     #[test]
//     fn absolute_long_addressing_mode_to_a0() {
//         // arrange
//         let code = [0x43, 0xf9, 0x00, 0xf8, 0x00, 0x00].to_vec(); // LEA ($00F80000).L,A1
//         let mem_range = MemRange::from_bytes(0xf80000, [0x00, 0x00, 0x00, 0x00].to_vec());
//         let mut cpu = crate::instr_test_setup(code, Some(mem_range));
//         cpu.register.reg_a[0] = 0x00000000;
//         cpu.register.reg_sr = 0x0000;
//          // act assert - debug
//          let debug_result = cpu.get_next_disassembly();
//          assert_eq!(
//              DisassemblyResult::Done {
//                  name: String::from("LEA"),
//                  operands_format: String::from("($00F80000).L,A1"),
//                  instr_address: 0x080000,
//                  next_instr_address: 0x080006
//              },
//              debug_result
//          );
//         // // act
//         cpu.execute_next_instruction();
//         // // assert
//         assert_eq!(0x00F80000, cpu.register.reg_a[1]);
//         assert_eq!(false, cpu.register.is_sr_carry_set());
//         assert_eq!(false, cpu.register.is_sr_coverflow_set());
//         assert_eq!(false, cpu.register.is_sr_zero_set());
//         assert_eq!(false, cpu.register.is_sr_negative_set());
//         assert_eq!(false, cpu.register.is_sr_extend_set());
//     }
// }
