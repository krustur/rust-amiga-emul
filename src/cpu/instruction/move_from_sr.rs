use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::{
    cpu::Cpu,
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
        true => crate::cpu::match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word),
        false => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    match reg.reg_sr.is_sr_supervisor_set() {
        true => {
            let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
                instr_word,
                reg,
                mem,
                |instr_word| Ok(OperationSize::Word),
            )?;

            let sr = reg.reg_sr.get_value();
            let data = ea_data.set_value_word(pc, reg, mem, sr, true);

            Ok(())
        }
        false => Err(StepError::PriviliegeViolation),
    }
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(OperationSize::Word),
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("MOVE.W"),
        format!("SR,{}", ea_format.format),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn move_from_sr_to_data_register_direct_ff00() {
        // arrange
        let code = [0x40, 0xc0].to_vec(); // MOVE.W SR,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0xffffffff);
        cpu.register.reg_sr.set_value(0xff00);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("SR,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff_ff00, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_from_sr_to_data_register_direct_020f() {
        // arrange
        let code = [0x40, 0xc7].to_vec(); // MOVE.W SR,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0xffffffff);
        cpu.register.reg_sr.set_value(0x200F);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("SR,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffff_200f, cpu.register.get_d_reg_long(7));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn move_from_sr_privilege_violation() {
        // arrange
        let code = [0x40, 0xc7].to_vec(); // MOVE.W SR,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_value(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        cpu.memory.set_long(0x00000020, 0x11223344);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("SR,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x11223344, cpu.register.reg_pc.get_address());
        assert_eq!(0x11223344, cpu.register.reg_pc.get_address_next());
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_supervisor_set());
    }
}
