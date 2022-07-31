use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::cpu::step_log::StepLog;
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{ProgramCounter, Register};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Tests!
// TODO: Check Supervisor

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
    step_log: &mut StepLog,
) -> Result<(), StepError> {
    match reg.reg_sr.is_sr_supervisor_set(step_log) {
        true => {
            let immediate_data = pc.fetch_next_word(mem);

            let dest = reg.reg_sr.get_value();

            let result = Cpu::or_words(immediate_data, dest);

            reg.reg_sr.set_value(result.result);

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
    step_log: &mut StepLog,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let immediate_data = format!("#${:04X}", pc.fetch_next_word(mem));

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("ORI.W"),
        format!("{},SR", immediate_data),
    ))
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn andi_byte_to_data_register_direct() {
    //     // arrange
    //     let code = [0x02, 0x00, 0x00, 0x55].to_vec(); // ANDI.B #$55,D0
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_d_reg_long(0, 0x123456f1);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00004,
    //             String::from("ANDI.B"),
    //             String::from("#$55,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x12345651, cpu.register.get_d_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn andi_byte_to_absolute_short() {
    //     // arrange
    //     let code = [0x02, 0x38, 0x00, 0x80, 0x40, 0x00].to_vec(); // ANDI.B #$80,($4000).W
    //     let mem_range = RamMemory::from_bytes(0x00004000, [0xf0].to_vec());
    //     let mut mem_ranges = Vec::new();
    //     mem_ranges.push(mem_range);
    //     let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00006,
    //             String::from("ANDI.B"),
    //             String::from("#$80,($4000).W")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x80, cpu.memory.get_byte(0x00004000));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn andi_word_to_data_register_direct() {
    //     // arrange
    //     let code = [0x02, 0x40, 0x66, 0x55].to_vec(); // ANDI.W #$6655,D0
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_d_reg_long(0, 0x1234dff1);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00004,
    //             String::from("ANDI.W"),
    //             String::from("#$6655,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x12344651, cpu.register.get_d_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn andi_word_to_absolute_short() {
    //     // arrange
    //     let code = [0x02, 0x78, 0x80, 0x80, 0x70, 0x00].to_vec(); // ANDI.W #$8080,($7000).W
    //     let mem_range = RamMemory::from_bytes(0x00007000, [0xf0, 0x7f].to_vec());
    //     let mut mem_ranges = Vec::new();
    //     mem_ranges.push(mem_range);
    //     let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00006,
    //             String::from("ANDI.W"),
    //             String::from("#$8080,($7000).W")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x8000, cpu.memory.get_word(0x00007000));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn andi_long_to_data_register_direct() {
    //     // arrange
    //     let code = [0x02, 0x80, 0x11, 0x33, 0x66, 0x55].to_vec(); // ANDI.L #$11336655,D0
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_d_reg_long(0, 0xccccdff1);
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE
    //             | STATUS_REGISTER_MASK_EXTEND,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00006,
    //             String::from("ANDI.L"),
    //             String::from("#$11336655,D0")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x00004651, cpu.register.get_d_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn andi_long_to_absolute_short() {
    //     // arrange
    //     let code = [0x02, 0xb8, 0xf0, 0x20, 0x80, 0x80, 0x70, 0x00].to_vec(); // ANDI.L #$F0208080,($7000).W
    //     let mem_range = RamMemory::from_bytes(0x00007000, [0x88, 0x88, 0x56, 0xf1].to_vec());
    //     let mut mem_ranges = Vec::new();
    //     mem_ranges.push(mem_range);
    //     let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
    //     cpu.register.reg_sr.set_sr_reg_flags_abcde(
    //         STATUS_REGISTER_MASK_CARRY
    //             | STATUS_REGISTER_MASK_OVERFLOW
    //             | STATUS_REGISTER_MASK_ZERO
    //             | STATUS_REGISTER_MASK_NEGATIVE,
    //     );
    //     // act assert - debug
    //     let debug_result = cpu.get_next_disassembly_no_log();
    //     assert_eq!(
    //         GetDisassemblyResult::from_address_and_address_next(
    //             0xC00000,
    //             0xC00008,
    //             String::from("ANDI.L"),
    //             String::from("#$F0208080,($7000).W")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x8000_0080, cpu.memory.get_long(0x00007000));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }
}
