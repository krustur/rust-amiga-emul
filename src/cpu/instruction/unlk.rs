use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
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

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    let restored_sp = reg.get_a_reg_long(register);

    reg.set_a_reg_long(7, restored_sp);

    let restored_a_reg = reg.stack_pop_long(mem);
    reg.set_a_reg_long(register, restored_a_reg);

    // println!(
    //     "unlk - A{}=${:08X} - SP=${:08x}",
    //     register,
    //     restored_a_reg,
    //     reg.get_a_reg_long(7)
    // );

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let register = Cpu::extract_register_index_from_bit_pos_0(instr_word)?;

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("UNLK"),
        format!("A{}", register,),
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
    fn unlk_a5_no_sr_set() {
        // arrange
        let code = [0x4e, 0x5d].to_vec(); // UNLK A5
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        cpu.register.set_a_reg_long(5, 0x10002fc);
        cpu.memory.set_long(0x10002fc, 0xa756789a);
        cpu.register.set_a_reg_long(7, 0x12345678);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("UNLK"),
                String::from("A5")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x1000300, cpu.register.get_a_reg_long(7));
        assert_eq!(0xa756789a, cpu.register.get_a_reg_long(5));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn unlk_a0_no_sr_cleared() {
        // arrange
        let code = [0x4e, 0x58].to_vec(); // UNLK A0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO,
        );
        cpu.register.set_a_reg_long(0, 0x1000330);
        cpu.memory.set_long(0x1000330, 0xfedcba98);
        cpu.register.set_a_reg_long(7, 0x00000000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("UNLK"),
                String::from("A0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x1000334, cpu.register.get_a_reg_long(7));
        assert_eq!(0xfedcba98, cpu.register.get_a_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }
}
