use super::{GetDisassemblyResultError, StepError};
use crate::{
    cpu::{instruction::GetDisassemblyResult, step_log::StepLog, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition_result = reg.reg_sr.evaluate_condition(&conditional_test);
    let displacement_16bit = pc.fetch_next_word(mem);

    match condition_result {
        false => {
            let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            let reg_word = reg.get_d_reg_word(register, step_log);
            let reg_word = reg_word.wrapping_sub(1);
            reg.set_d_reg_word(step_log, register, reg_word);

            match reg_word {
                0xffff => {
                    // == -1 => loop done, next instruction
                }
                _ => {
                    // != -1 => loop not done, branch
                    pc.branch_word(displacement_16bit);
                }
            }
        }
        true => (),
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
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    let displacement_16bit = pc.fetch_next_word(mem);

    let branch_to = Cpu::get_address_with_word_displacement_sign_extended(
        pc.get_address() + 2,
        displacement_16bit,
    );

    let result = Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        format!("DB{:?}", conditional_test),
        format!(
            "D{},${:04X} [${:08X}]",
            register, displacement_16bit, branch_to
        ),
    ));

    result
}

#[cfg(test)]
mod tests {
    use crate::{cpu::instruction::GetDisassemblyResult, register::STATUS_REGISTER_MASK_CARRY};

    // DBCC C set/reg gt 0 => decrease reg and branch (not -1)
    // DBCC C set/reg eq 0 => decrease reg and no branch (-1)
    // DBCC C clear => do nothing

    #[test]
    fn dbcc_cc_when_carry_set_and_reg_greater_than_zero_decrease_reg_and_branch() {
        // arrange
        let code = [0x54, 0xc8, 0x00, 0x04].to_vec(); // DBCC D0,$0004
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xffff0001);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D0,$0004 [$00C00006]"),
                vec![0x54c8, 0x0004]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffff0000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(0xC00006, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn dbcc_cc_when_carry_set_and_reg_equal_to_zero_decrease_reg_and_no_branch() {
        // arrange
        let code = [0x54, 0xc9, 0x00, 0x04].to_vec(); // DBCC D1,$0004
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(1, 0x11110000);
        mm.cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY);
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D1,$0004 [$00C00006]"),
                vec![0x54c9, 0x0004]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x1111ffff, mm.cpu.register.get_d_reg_long_no_log(1));
        assert_eq!(0xc00004, mm.cpu.register.reg_pc.get_address());
    }

    #[test]
    fn dbcc_cc_when_carry_clear_do_nothing() {
        // arrange
        let code = [0x54, 0xca, 0x00, 0x04].to_vec(); // DBCC D2,$0004
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(2, 0xffff0001);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000); //STATUS_REGISTER_MASK_CARRY;

        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D2,$0004 [$00C00006]"),
                vec![0x54ca, 0x0004]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xffff0001, mm.cpu.register.get_d_reg_long_no_log(2));
        assert_eq!(0xC00004, mm.cpu.register.reg_pc.get_address());
    }
}
