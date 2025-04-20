use super::{
    ConditionalTest, GetDisassemblyResult, GetDisassemblyResultError, Instruction, StepError,
};
use crate::{
    cpu::{step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE (not affected)
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => {
            let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
            match conditional_test {
                ConditionalTest::F | ConditionalTest::T => false,
                _ => true,
            }
        }
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
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition = reg.reg_sr.evaluate_condition(&conditional_test);

    let displacement = Cpu::get_byte_from_word(instr_word);

    let result = match displacement {
        0x00 => {
            let displacement = pc.fetch_next_word(mem);
            if condition == true {
                pc.branch_word(displacement);
            }
        }
        0xff => {
            let displacement = pc.fetch_next_long(mem);
            if condition == true {
                pc.branch_long(displacement);
            }
        }
        _ => {
            if condition == true {
                pc.branch_byte(displacement);
            }
        }
    };
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);

    let displacement = Cpu::get_byte_from_word(instr_word);

    match displacement {
        0x00 => {
            let displacement = pc.fetch_next_word(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                format!("B{}.W", conditional_test),
                format!(
                    "${:04X} [${:08X}]",
                    displacement,
                    pc.get_branch_word_address(displacement)
                ),
            ))
        }
        0xff => {
            let displacement = pc.fetch_next_long(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                mem,
                format!("B{}.L", conditional_test),
                format!(
                    "${:08X} [${:08X}]",
                    displacement,
                    pc.get_branch_long_address(displacement)
                ),
            ))
        }
        _ => Ok(GetDisassemblyResult::from_pc(
            pc,
            mem,
            format!("B{}.B", conditional_test),
            format!(
                "${:02X} [${:08X}]",
                displacement,
                pc.get_branch_byte_address(displacement)
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    // byte

    #[test]
    fn step_bcc_byte_when_carry_clear() {
        // arrange
        let code = [0x64, 0x06].to_vec(); // BCC.B $06
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCC.B"),
                String::from("$06 [$00C00008]"),
                vec![0x6406]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00008, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_bcc_byte_when_carry_set() {
        // arrange
        let code = [0x64, 0x06].to_vec(); // BCC.B $06
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BCC.B"),
                String::from("$06 [$00C00008]"),
                vec![0x6406]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00002, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_beq_byte_when_zero_set_negative() {
        // arrange
        let code = [0x67, 0xfa].to_vec(); // BEQ.B $FA
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BEQ.B"),
                String::from("$FA [$00BFFFFC]"),
                vec![0x67fa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00BFFFFC, mm.cpu.register.reg_pc.get_address());
    }

    // word

    #[test]
    fn step_beq_word_when_zero_set_negative() {
        // arrange
        let code = [0x67, 0x00, 0xff, 0xfa].to_vec(); // BEQ.W $FFFA
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BEQ.W"),
                String::from("$FFFA [$00BFFFFC]"),
                vec![0x6700, 0xfffa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00BFFFFC, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_beq_word_when_zero_set() {
        // arrange
        let code = [0x67, 0x00, 0x00, 0x60].to_vec(); // BEQ.W $0060
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BEQ.W"),
                String::from("$0060 [$00C00062]"),
                vec![0x6700, 0x0060]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00062, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_beq_word_when_zero_clear_negative() {
        // arrange
        let code = [0x67, 0x00, 0xff, 0xfa].to_vec(); // BEQ.W $FFFA
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BEQ.W"),
                String::from("$FFFA [$00BFFFFC]"),
                vec![0x6700, 0xfffa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00C00004, mm.cpu.register.reg_pc.get_address());
    }

    // long

    #[test]
    fn step_bgt_long_when_true_negative() {
        // arrange
        let code = [0x6e, 0xff, 0xff, 0xff, 0xff, 0xfa].to_vec(); // BGT.L $FFFFFFFA
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_NEGATIVE | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BGT.L"),
                String::from("$FFFFFFFA [$00BFFFFC]"),
                vec![0x6eff, 0xffff, 0xfffa]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00BFFFFC, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_bgt_long_when_true() {
        // arrange
        let code = [0x6e, 0xff, 0x00, 0x00, 0x80, 0x00].to_vec(); // BGT.L $00008000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BGT.L"),
                String::from("$00008000 [$00C08002]"),
                vec![0x6eff, 0x0000, 0x8000]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC08002, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn step_bgt_long_when_false() {
        // arrange
        let code = [0x6e, 0xff, 0x00, 0x00, 0x80, 0x00].to_vec(); // BGT.L $00008000
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("BGT.L"),
                String::from("$00008000 [$00C08002]"),
                vec![0x6eff, 0x0000, 0x8000]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xC00006, mm.cpu.register.reg_pc.get_address());
    }
}
