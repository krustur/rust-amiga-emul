use super::{
    GetDisassemblyResult, GetDisassemblyResultError, Instruction, OperationSize, StepError,
};
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};

// Instruction State
// =================
// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => crate::cpu::match_check_ea_only_data_addressing_modes_pos_0(instr_word),
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
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            // Mulu for 68000 is always word*word => long. MULU.L for 020+ in own get_disassembly_long function.
            Ok(OperationSize::Word)
        },
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let source = ea_data.get_value_word(pc, reg, mem, step_log, true);

    let dest = reg.get_d_reg_word(register, step_log);
    let result = Cpu::mulu_words(source, dest);

    reg.set_d_reg_long(step_log, register, result.result);

    reg.reg_sr
        .merge_status_register(step_log, result.status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| {
            // Mulu for 68000 is always word*word => long. MULU.L for 020+ in own get_disassembly_long function.
            Ok(OperationSize::Word)
        },
    )?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        mem,
        String::from("MULU.W"),
        format!("{},D{}", ea_format, register),
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
    fn mulu_word_immediate_data_to_data_register() {
        // arrange
        let code = [0xc0, 0xfc, 0x12, 0x34].to_vec(); // MULU.W #$1234,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x11110018);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MULU.W"),
                String::from("#$1234,D0"),
                vec![0xc0fc, 0x1234]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x0001b4e0, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn mulu_word_immediate_data_to_data_register_zero() {
        // arrange
        let code = [0xc0, 0xfc, 0x12, 0x34].to_vec(); // MULU.W #$1234,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xffff_0000);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MULU.W"),
                String::from("#$1234,D0"),
                vec![0xc0fc, 0x1234]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x00000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn mulu_word_immediate_data_to_data_register_negative() {
        // arrange
        let code = [0xc0, 0xfc, 0xf0, 0x00].to_vec(); // MULU.W #$F000,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xffff_c000);
        mm.cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act assert - debug
        let debug_result = mm.get_next_disassembly_no_log();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MULU.W"),
                String::from("#$F000,D0"),
                vec![0xc0fc, 0xf000]
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xb4000000, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
