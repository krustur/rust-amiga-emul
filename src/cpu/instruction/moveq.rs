use crate::cpu::instruction::InstructionDebugResult;
use crate::mem::Mem;
use crate::register::{Register, STATUS_REGISTER_MASK_ZERO};
use crate::{cpu::instruction::PcResult, cpu::Cpu, register::STATUS_REGISTER_MASK_NEGATIVE};
use byteorder::ReadBytesExt;

use super::InstructionExecutionResult;

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // TODO: Condition codes
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    let operand = instr_bytes.read_i8().unwrap();
    let mut status_register_flags = 0x0000;
    match operand {
        0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
        i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
        _ => (),
    }
    let operand = Cpu::sign_extend_i8(operand);
    let operands_format = format!("#{},D{}", operand, register);
    let instr_comment = format!("moving {:#010x} into D{}", operand, register);
    let status_register_mask = 0xfff0;

    reg.reg_d[register] = operand;
    reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;
    InstructionExecutionResult::Done {
        // name: "MOVEQ",
        // operands_format: &operands_format,
        // comment: &instr_comment,
        // op_size: OperationSize::Long,
        pc_result: PcResult::Increment(2),
    }
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> InstructionDebugResult {
    // TODO: Condition codes
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
    let operand = instr_bytes.read_i8().unwrap();
    // let mut status_register_flags = 0x0000;
    // match operand {
    //     0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
    //     i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
    //     _ => (),
    // }
    // let operand = Cpu::sign_extend_i8(operand);
    let operands_format = format!("#{},D{}", operand, register);
    let instr_comment = format!("moving {:#010x} into D{}", operand, register);
    let status_register_mask = 0xfff0;

    InstructionDebugResult::Done {
        name: String::from("MOVEQ"),
        operands_format: operands_format,
        // comment: &instr_comment,
        // op_size: OperationSize::Long,
        // pc_result: PcResult::Increment(2),
        next_instr_address: instr_address + 2,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::InstructionDebugResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn step_positive_d0() {
        // arrange
        let code = [0x70, 0x1d].to_vec(); // MOVEQ #$1d,d0
        let mut cpu = crate::instr_test_setup(code);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE;
        // act
        let debug_result = cpu.execute_next_instruction();
        // assert
        assert_eq!(0x1d, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("MOVEQ"),
                operands_format: String::from("#29,D0"),
                next_instr_address: 0x080002
            },
            debug_result
        );
    }

    #[test]
    fn step_negative_d0() {
        // arrange
        let code = [0x72, 0xff].to_vec(); // MOVEQ #-1,d1
        let mut cpu = crate::instr_test_setup(code);
        cpu.register.reg_sr =
            STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW | STATUS_REGISTER_MASK_ZERO;
        // act
        let debug_result = cpu.execute_next_instruction();
        // assert
        assert_eq!(0xffffffff, cpu.register.reg_d[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("MOVEQ"),
                operands_format: String::from("#-1,D1"),
                next_instr_address: 0x080002
            },
            debug_result
        );
    }

    #[test]
    fn step_zero_d0() {
        // arrange
        let code = [0x74, 0x00].to_vec(); // MOVEQ #0,d0
        let mut cpu = crate::instr_test_setup(code);
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;
        // act
        let debug_result = cpu.execute_next_instruction();
        // assert
        assert_eq!(0, cpu.register.reg_d[2]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(
            InstructionDebugResult::Done {
                name: String::from("MOVEQ"),
                operands_format: String::from("#0,D2"),
                next_instr_address: 0x080002
            },
            debug_result
        );
    }
}
