use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::{instruction::OperationSize, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

// Instruction State
// =================
// step: DONE
// step cc: DONE
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
            Cpu::extract_size000110_from_bit_pos_6(instr_word)
        })?;
    let status_register_result = match ea_data.operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let data = pc.fetch_next_byte(mem);
            let ea_value = ea_data.get_value_byte(pc, reg, mem, false);
            let add_result = Cpu::sub_bytes(data, ea_value);
            ea_data.set_value_byte(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        OperationSize::Word => {
            let data = pc.fetch_next_word(mem);
            let ea_value = ea_data.get_value_word(pc, reg, mem, false);
            let add_result = Cpu::sub_words(data, ea_value);
            ea_data.set_value_word(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        OperationSize::Long => {
            let data = pc.fetch_next_long(mem);
            let ea_value = ea_data.get_value_long(pc, reg, mem, false);
            let add_result = Cpu::sub_longs(data, ea_value);
            ea_data.set_value_long(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
    };

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())
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
    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(ea_data.operation_size), mem);
    match ea_data.operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let data = pc.fetch_next_byte(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("SUBI.B"),
                format!("#${:02X},{}", data, ea_format),
            ))
        }
        OperationSize::Word => {
            let data = pc.fetch_next_word(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("SUBI.W"),
                format!("#${:04X},{}", data, ea_format),
            ))
        }
        OperationSize::Long => {
            let data = pc.fetch_next_long(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("SUBI.L"),
                format!("#${:04X},{}", data, ea_format),
            ))
        }
    }
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
    fn subi_immediate_data_to_data_register_direct_byte() {
        // arrange
        let code = [0x04, 0x07, 0x00, 0x23].to_vec(); // SUBI.B #$23,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0x00004325);
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
                String::from("SUBI.B"),
                String::from("#$23,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x4302, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subi_immediate_data_to_data_register_direct_word() {
        // arrange
        let code = [0x04, 0x47, 0x12, 0x34].to_vec(); // SUBI.W #$1234,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(7, 0x00004321);
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
                String::from("SUBI.W"),
                String::from("#$1234,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x30ED, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn subi_immediate_data_to_data_register_direct_long() {
        // arrange
        let code = [0x04, 0x80, 0x10, 0x10, 0x10, 0x10].to_vec(); // SUBI.L #$10101010,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x76857685);
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
                0xC00006,
                String::from("SUBI.L"),
                String::from("#$10101010,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x66756675, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
