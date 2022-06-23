use crate::cpu::instruction::GetDisassemblyResult;
use crate::memhandler::MemHandler;
use crate::register::{ProgramCounter, Register, STATUS_REGISTER_MASK_ZERO};
use crate::{cpu::Cpu, register::STATUS_REGISTER_MASK_NEGATIVE};

use super::{GetDisassemblyResultError, StepError, StepResult};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut MemHandler,
) -> Result<StepResult, StepError> {
    // TODO: Condition codes
    let instr_word = pc.fetch_next_word(mem);
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    // let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    // let operand = instr_bytes.read_i8().unwrap();
    let data = Cpu::get_byte_from_word(instr_word);
    // println!("data: {}", data);
    let mut status_register_flags = 0x0000;
    match data {
        0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
        0x80..=0xff => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
        _ => (),
    }
    let data = Cpu::sign_extend_byte(data);
    // let operands_format = format!("#{},D{}", data, register);
    // let instr_comment = format!("moving {:#010x} into D{}", operand, register);
    let status_register_mask = 0xfff0;

    reg.reg_d[register] = data;
    reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;
    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &MemHandler,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    // TODO: Condition codes
    let instr_word = pc.fetch_next_word(mem);
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9)?;
    // let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    let data = Cpu::get_byte_from_word(instr_word);
    // let operand = instr_bytes.read_i8().unwrap();
    // let mut status_register_flags = 0x0000;
    // match operand {
    //     0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
    //     i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
    //     _ => (),
    // }
    // let operand = Cpu::sign_extend_i8(operand);
    let data = Cpu::sign_extend_byte(data);
    let data_signed = Cpu::get_signed_long_from_long(data);
    let operands_format = format!("#{},D{}", data_signed, register);
    // let instr_comment = format!("moving {:#010x} into D{}", data, register);
    let status_register_mask = 0xfff0;

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("MOVEQ"),
        operands_format,
    ))
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

    #[test]
    fn step_positive_d0() {
        // arrange
        let code = [0x70, 0x1d].to_vec(); // MOVEQ #$1d,d0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        // act
        let debug_result = cpu.get_next_disassembly();
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x1d, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#29,D0")
            ),
            debug_result
        );
    }

    #[test]
    fn step_negative_d0() {
        // arrange
        let code = [0x72, 0xff].to_vec(); // MOVEQ #-1,d1
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO;
        // act
        let debug_result = cpu.get_next_disassembly();
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffffff, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#-1,D1")
            ),
            debug_result
        );
    }

    #[test]
    fn step_zero_d0() {
        // arrange
        let code = [0x74, 0x00].to_vec(); // MOVEQ #0,d0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;
        // act
        let debug_result = cpu.get_next_disassembly();
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0, cpu.register.reg_d[2]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEQ"),
                String::from("#0,D2")
            ),
            debug_result
        );
    }
}
