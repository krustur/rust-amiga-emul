use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: TODO
// step cc: DONE (not affected)
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    pc.skip_word();
    reg.stack_pop_pc(mem, pc);
    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    pc.skip_word();
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("RTS"),
        String::from(""),
    ))
}

#[cfg(test)]
mod tests {
    use crate::cpu::instruction::GetDisassemblyResult;

    // // byte

    // #[test]
    // fn step_bsr_byte() {
    //     // arrange
    //     let code = [0x61, 0x02].to_vec(); // BSR.B $02
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("BSR.B"),
    //             String::from("$02 [$00C00004]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00002, cpu.memory.get_long(0x10003fc));
    // }

    // #[test]
    // fn step_bsr_byte_negative() {
    //     // arrange
    //     let code = [0x61, 0xfc].to_vec(); // BSR.B $FC
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00002,
    //             String::from("BSR.B"),
    //             String::from("$FC [$00BFFFFE]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xBFFFFE, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00002, cpu.memory.get_long(0x10003fc));
    // }

    // // word

    // #[test]
    // fn step_bsr_word() {
    //     // arrange
    //     let code = [0x61, 0x00, 0x00, 0x04].to_vec(); // BSR.W $0004
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00004,
    //             String::from("BSR.W"),
    //             String::from("$0004 [$00C00006]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xC00006, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00004, cpu.memory.get_long(0x10003fc));
    // }

    // #[test]
    // fn step_bsr_word_negative() {
    //     // arrange
    //     let code = [0x61, 0x00, 0xff, 0xfc].to_vec(); // BSR.W $FFFC
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00004,
    //             String::from("BSR.W"),
    //             String::from("$FFFC [$00BFFFFE]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xBFFFFE, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00004, cpu.memory.get_long(0x10003fc));
    // }

    // // long

    // #[test]
    // fn step_bsr_long() {
    //     // arrange
    //     let code = [0x61, 0xff, 0x00, 0x00, 0x00, 0x06].to_vec(); // BSR.L $00000006
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00006,
    //             String::from("BSR.L"),
    //             String::from("$00000006 [$00C00008]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xC00008, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00006, cpu.memory.get_long(0x10003fc));
    // }

    // #[test]
    // fn step_bsr_long_negative() {
    //     // arrange
    //     let code = [0x61, 0xff, 0xff, 0xff, 0xff, 0xfc].to_vec(); // BSR.L $FFFFFFFC
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

    //     println!("sp: ${:08X}", cpu.register.get_a_reg_long(7));

    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00006,
    //             String::from("BSR.L"),
    //             String::from("$FFFFFFFC [$00BFFFFE]")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0xBFFFFE, cpu.register.reg_pc.get_address());
    //     assert_eq!(0x10003fc, cpu.register.get_a_reg_long(7));
    //     assert_eq!(0xC00006, cpu.memory.get_long(0x10003fc));
    // }
}
