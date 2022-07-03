use super::{
    GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError, StepResult,
};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_ZERO},
};

// step: DONE
// step cc: DONE
// get_disassembly: DONE

// 020+ step: TODO
// 020+ get_disassembly: TODO

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<StepResult, StepError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Cpu::extract_size000110_from_bit_pos_6(instr_word)
        })?;

    match ea_data.operation_size {
        OperationSize::Byte => ea_data.set_value_byte(pc, reg, mem, 0x00, true),
        OperationSize::Word => ea_data.set_value_word(pc, reg, mem, 0x0000, true),
        OperationSize::Long => ea_data.set_value_long(pc, reg, mem, 0x00000000, true),
    };

    reg.reg_sr = (reg.reg_sr & STATUS_REGISTER_MASK_EXTEND) | STATUS_REGISTER_MASK_ZERO;

    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Cpu::extract_size000110_from_bit_pos_6(instr_word)
        })?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), reg, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from(format!("CLR.{}", ea_data.operation_size.get_format())),
        ea_format.format,
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
    fn clr_byte_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0x2b, 0x00, 0x0a, /* DC */ 0x88].to_vec(); // CLR.B ($000A,A3)
                                                                     // DC.B $88
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0xBFFFFA;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CLR.B"),
                String::from("($000A,A3) [10]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.memory.get_byte(0xC00004));
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn clr_word_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0x6b, 0x00, 0x0a, /* DC */ 0x88, 0x77].to_vec(); // CLR.W ($000A,A3)
                                                                           // DC.W $8877
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0xBFFFFA;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CLR.W"),
                String::from("($000A,A3) [10]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // asser00t
        assert_eq!(0x00, cpu.memory.get_word(0xC00004));
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn clr_long_address_register_indirect_with_displacement() {
        // arrange
        let code = [0x42, 0xab, 0x00, 0x0a, /* DC */ 0x88, 0x77, 0x99, 0x66].to_vec(); // CLR.L ($000A,A3)
                                                                                       // DC.W $88779966
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0xBFFFFA;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CLR.L"),
                String::from("($000A,A3) [10]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // asser00t
        assert_eq!(0x00000000, cpu.memory.get_long(0xC00004));
        assert_eq!(0xC00004, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
