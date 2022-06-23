use super::{GetDisassemblyResultError, StepError, StepResult};
use crate::{
    cpu::{instruction::GetDisassemblyResult, Cpu},
    memhandler::MemHandler,
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
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut MemHandler,
) -> Result<StepResult, StepError> {
    let instr_word = pc.fetch_next_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let condition_result = Cpu::evaluate_condition(reg, &conditional_test);
    let displacement_16bit = pc.fetch_next_word(mem);

    let result = match condition_result {
        false => {
            let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;
            let reg_word = Cpu::get_word_from_long(reg.reg_d[register]);
            let reg_word = reg_word.wrapping_sub(1);
            reg.reg_d[register] = Cpu::set_word_in_long(reg_word, reg.reg_d[register]);

            match reg_word {
                0xffff => {
                    // == -1 => loop done, next instruction
                    StepResult::Done {}
                }
                _ => {
                    // != -1 => loop not done, branch
                    pc.branch_word(displacement_16bit);
                    StepResult::Done {}
                }
            }
        }
        true => StepResult::Done {},
    };

    Ok(result)
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &MemHandler,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.fetch_next_word(mem);
    let conditional_test = Cpu::extract_conditional_test_pos_8(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    let displacement_16bit = pc.fetch_next_word(mem);

    let branch_to = Cpu::get_address_with_word_displacement_sign_extended(
        pc.get_address() + 2,
        displacement_16bit,
    );

    let result = Ok(GetDisassemblyResult::from_pc(
        pc,
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
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0xffff0001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D0,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0000, cpu.register.reg_d[0]);
        assert_eq!(0xC00006, cpu.register.reg_pc.get_address());
    }

    #[test]
    fn dbcc_cc_when_carry_set_and_reg_equal_to_zero_decrease_reg_and_no_branch() {
        // arrange
        let code = [0x54, 0xc9, 0x00, 0x04].to_vec(); // DBCC D1,$0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[1] = 0x11110000;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D1,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x1111ffff, cpu.register.reg_d[1]);
        assert_eq!(0xc00004, cpu.register.reg_pc.get_address());
    }

    #[test]
    fn dbcc_cc_when_carry_clear_do_nothing() {
        // arrange
        let code = [0x54, 0xca, 0x00, 0x04].to_vec(); // DBCC D2,$0004
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[2] = 0xffff0001;
        cpu.register.reg_sr = 0x0000; //STATUS_REGISTER_MASK_CARRY;

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("DBCC"),
                String::from("D2,$0004 [$00C00006]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff0001, cpu.register.reg_d[2]);
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
    }
}
