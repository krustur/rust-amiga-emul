use super::{GetDisassemblyResult, GetDisassemblyResultError, OperationSize, StepError};
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
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(OperationSize::Long)
        })?;
    let ea_address = ea_data.get_address(pc, reg, mem);

    reg.stack_push_long(mem, ea_address);
    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(OperationSize::Long)
        })?;
    let ea_mode = ea_data.ea_mode;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, mem);
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("PEA"),
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
    fn pea_pc_indirect_with_displacement() {
        // arrange
        let code = [0x48, 0x7a, 0x00, 0x18].to_vec(); // PEA ($0018,PC) [$00C0001a]
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_ZERO
                | STATUS_REGISTER_MASK_NEGATIVE
                | STATUS_REGISTER_MASK_EXTEND,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("PEA"),
                String::from("($0018,PC) [$00C0001A]")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x010003fc, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00C0001a, cpu.memory.get_long(0x010003fc));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn pea_absolute_short() {
        // arrange
        let code = [0x48, 0x78, 0x40, 0x00].to_vec(); // PEA ($4000).W
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("PEA"),
                String::from("($4000).W")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x010003fc, cpu.register.get_a_reg_long(7));
        assert_eq!(0x00004000, cpu.memory.get_long(0x010003fc));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
