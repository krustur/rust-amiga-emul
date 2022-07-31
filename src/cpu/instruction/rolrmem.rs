use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
use crate::cpu::step_log::StepLog;
use crate::cpu::{Cpu, StatusRegisterResult};
use crate::mem::Mem;
use crate::register::{
    ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
    STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: Tests!

enum RolrDirection {
    Right,
    Left,
}

impl RolrDirection {
    pub fn get_format(&self) -> char {
        match self {
            RolrDirection::Right => 'R',
            RolrDirection::Left => 'L',
        }
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
        |instr_word| Ok(OperationSize::Word),
    )?;
    let rolr_direction = match ea_data.instr_word & 0x0100 {
        0x0100 => RolrDirection::Left,
        _ => RolrDirection::Right,
    };

    let value = ea_data.get_value_word(pc, reg, mem, step_log, false);
    println!("value: ${:08X}", value);
    let (result, overflow) = match rolr_direction {
        RolrDirection::Left => {
            // println!("rolr_direction: left");
            // let result = value.checked_shl(1).unwrap_or(0);
            // let overflow = result & 0x10000 != 0;
            // let result = (result & 0xffff) as u16;
            let result = value.rotate_left(1);
            (result, (result & 0x00000001) != 0)
        }
        RolrDirection::Right => {
            // println!("rolr_direction: right");
            // let result = (value << 1).checked_shr(1).unwrap_or(0);
            // let overflow = result & 0x0001 != 0;
            // let result = ((result >> 1) & 0xffff) as u16;
            let result = value.rotate_right(1);

            (result, (result & 0x00000001) != 0)
        }
    };

    ea_data.set_value_word(pc, reg, mem, step_log, result, true);
    let mut status_register = 0x0000;
    match result {
        0 => status_register |= STATUS_REGISTER_MASK_ZERO,
        0x8000..=0xffff => status_register |= STATUS_REGISTER_MASK_NEGATIVE,
        _ => (),
    }
    // println!("shift_count: {}", shift_count);
    // println!("result: {}", result);
    // println!("overflow: {}", overflow);
    match overflow {
        true => status_register |= STATUS_REGISTER_MASK_EXTEND | STATUS_REGISTER_MASK_CARRY,
        false => (),
    }
    let status_register_result = StatusRegisterResult {
        status_register,
        status_register_mask: get_status_register_mask(1),
    };

    // println!(
    //     "status_register_result: ${:04X}-${:04X}",
    //     status_register_result.status_register, status_register_result.status_register_mask
    // );
    reg.reg_sr
        .merge_status_register(step_log, status_register_result);

    Ok(())
}

fn get_status_register_mask(shift_count: u32) -> u16 {
    match shift_count == 0 {
        true => {
            STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
        }
        false => {
            STATUS_REGISTER_MASK_EXTEND
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_CARRY
        }
    }
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
        |instr_word| Ok(OperationSize::Word),
    )?;
    let rolr_direction = match ea_data.instr_word & 0x0100 {
        0x0100 => RolrDirection::Left,
        _ => RolrDirection::Right,
    };
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);
    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("LS{}.W", rolr_direction.get_format(),),
        format!("#$01,{}", ea_format),
    ))
}

#[cfg(test)]
mod tests {

    // rol/ror memory(ea) by 1 / XNZC / word

    // #[test]
    // fn lsl_memory_word() {
    //     // arrange
    //     let code = [0xe3, 0xe0, /* DC */ 0x11, 0x11].to_vec(); // LSL.W #1,-(A0)
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(0, 0x00C00004);
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
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,-(A0)")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x2222, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00002, cpu.register.get_a_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsl_memory_word_negative() {
    //     // arrange
    //     let code = [0xe3, 0xe0, /* DC */ 0x41, 0x41].to_vec(); // LSL.W #1,-(A0)
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(0, 0x00C00004);
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
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,-(A0)")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x8282, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00002, cpu.register.get_a_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsl_memory_word_zero() {
    //     // arrange
    //     let code = [0xe3, 0xe0, /* DC */ 0x00, 0x00].to_vec(); // LSL.W #1,-(A0)
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(0, 0x00C00004);
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
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,-(A0)")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x0000, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00002, cpu.register.get_a_reg_long(0));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsl_memory_word_extend_carry() {
    //     // arrange
    //     let code = [0xe3, 0xe0, /* DC */ 0x81, 0x00].to_vec(); // LSL.W #1,-(A0)
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(0, 0x00C00004);
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
    //             0xC00002,
    //             String::from("LSL.W"),
    //             String::from("#$01,-(A0)")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x0200, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00002, cpu.register.get_a_reg_long(0));
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsr_memory_word() {
    //     // arrange
    //     let code = [0xe2, 0xde, /* DC */ 0x11, 0x10].to_vec(); // LSR.W #1,(A6)+
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(6, 0x00C00002);
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
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$01,(A6)+")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x0888, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00004, cpu.register.get_a_reg_long(6));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsr_memory_word_zero() {
    //     // arrange
    //     let code = [0xe2, 0xde, /* DC */ 0x00, 0x00].to_vec(); // LSR.W #1,(A6)+
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(6, 0x00C00002);
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
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$01,(A6)+")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x0000, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00004, cpu.register.get_a_reg_long(6));
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    // }

    // #[test]
    // fn lsr_memory_word_extend_carry() {
    //     // arrange
    //     let code = [0xe2, 0xde, /* DC */ 0x81, 0x01].to_vec(); // LSR.W #1,(A6)+
    //     let mut cpu = crate::instr_test_setup(code, None);
    //     cpu.register.set_a_reg_long(6, 0x00C00002);
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
    //             0xC00002,
    //             String::from("LSR.W"),
    //             String::from("#$01,(A6)+")
    //         ),
    //         debug_result
    //     );
    //     // act
    //     cpu.execute_next_instruction();
    //     // assert
    //     assert_eq!(0x4080, cpu.memory.get_word(0x00C00002));
    //     assert_eq!(0x00C00004, cpu.register.get_a_reg_long(6));
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    //     assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    //     assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    // }
}
