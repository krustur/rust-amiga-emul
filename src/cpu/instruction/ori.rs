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
        true => match crate::cpu::match_check_size000110_from_bit_pos_6(instr_word) {
            true => {
                crate::cpu::match_check_ea_only_data_alterable_addressing_modes_pos_0(instr_word)
            }
            false => false,
        },
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
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();
    let status_register_result = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let source = pc.fetch_next_byte(mem);

            let ea_data = pc.get_effective_addressing_data_from_bit_pos(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_byte(pc, reg, mem, step_log, true);

            let result = Cpu::or_bytes(source, dest);
            ea_data.set_value_byte(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Word => {
            let source = pc.fetch_next_word(mem);

            let ea_data = pc.get_effective_addressing_data_from_bit_pos(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_word(pc, reg, mem, step_log, true);

            let result = Cpu::or_words(source, dest);
            ea_data.set_value_word(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
        OperationSize::Long => {
            let source = pc.fetch_next_long(mem);

            let ea_data = pc.get_effective_addressing_data_from_bit_pos(
                instr_word,
                reg,
                mem,
                step_log,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_long(pc, reg, mem, step_log, true);

            let result = Cpu::or_longs(source, dest);
            ea_data.set_value_long(pc, reg, mem, step_log, result.result, true);
            result.status_register_result
        }
    };

    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word).unwrap();
    let immediate_data = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            format!("#${:02X}", pc.fetch_next_byte(mem))
        }
        OperationSize::Word => format!("#${:04X}", pc.fetch_next_word(mem)),
        OperationSize::Long => format!("#${:08X}", pc.fetch_next_long(mem)),
    };

    let ea_data = pc.get_effective_addressing_data_from_bit_pos(
        instr_word,
        reg,
        mem,
        step_log,
        |instr_word| Ok(operation_size),
        3,
        0,
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("ORI.{}", ea_data.operation_size.get_format()),
        format!("{},{}", immediate_data, ea_format),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        mem::rammemory::RamMemory,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn ori_byte_to_data_register_direct() {
        // arrange
        let code = [0x00, 0x00, 0x00, 0x55].to_vec(); // ORI.B #$55,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x12345661);
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
                String::from("ORI.B"),
                String::from("#$55,D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x12345675, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_byte_to_absolute_short() {
        // arrange
        let code = [0x00, 0x38, 0x00, 0x80, 0x40, 0x00].to_vec(); // ORI.B #$80,($4000).W
        let mem_range = RamMemory::from_bytes(0x00004000, [0xf0].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut mm = crate::tests::instr_test_setup(code, Some(mem_ranges));
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
                0xC00006,
                String::from("ORI.B"),
                String::from("#$80,($4000).W")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xf0, mm.mem.get_byte_no_log(0x00004000));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_word_to_data_register_direct() {
        // arrange
        let code = [0x00, 0x40, 0x66, 0x55].to_vec(); // ORI.W #$6655,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0x1234dff1);
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
                String::from("ORI.W"),
                String::from("#$6655,D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0x1234fff5, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_word_to_absolute_short() {
        // arrange
        let code = [0x00, 0x78, 0x80, 0x80, 0x70, 0x00].to_vec(); // ORI.W #$8080,($7000).W
        let mem_range = RamMemory::from_bytes(0x00007000, [0xf0, 0x7f].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut mm = crate::tests::instr_test_setup(code, Some(mem_ranges));
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
                0xC00006,
                String::from("ORI.W"),
                String::from("#$8080,($7000).W")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xf0ff, mm.mem.get_word_no_log(0x00007000));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_long_to_data_register_direct() {
        // arrange
        let code = [0x00, 0x80, 0x11, 0x33, 0x66, 0x55].to_vec(); // ORI.L #$11336655,D0
        let mut mm = crate::tests::instr_test_setup(code, None);
        mm.cpu.register.set_d_reg_long_no_log(0, 0xccccdff1);
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
                0xC00006,
                String::from("ORI.L"),
                String::from("#$11336655,D0")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xDDFFFFF5, mm.cpu.register.get_d_reg_long_no_log(0));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn ori_long_to_absolute_short() {
        // arrange
        let code = [0x00, 0xb8, 0xf0, 0x20, 0x80, 0x80, 0x70, 0x00].to_vec(); // ORI.L #$F0208080,($7000).W
        let mem_range = RamMemory::from_bytes(0x00007000, [0x88, 0x88, 0x56, 0xf1].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut mm = crate::tests::instr_test_setup(code, Some(mem_ranges));
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
                0xC00008,
                String::from("ORI.L"),
                String::from("#$F0208080,($7000).W")
            ),
            debug_result
        );
        // act
        mm.step();
        // assert
        assert_eq!(0xF8A8_D6F1, mm.mem.get_long_no_log(0x00007000));
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, mm.cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, mm.cpu.register.reg_sr.is_sr_extend_set());
    }
}
