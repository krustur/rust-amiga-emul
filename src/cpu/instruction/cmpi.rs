use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError};
use crate::{
    cpu::{instruction::OperationSize, Cpu, StatusRegisterResult},
    mem::Mem,
    register::{
        ProgramCounter, Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    },
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
    let instr_word = pc.fetch_next_word(mem);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    let status_register = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let source = pc.fetch_next_byte(mem);

            let ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
                instr_word,
                reg,
                mem,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_byte(pc, reg, mem, true);

            let add_result = Cpu::sub_bytes(source, dest);

            add_result.status_register_result.status_register
        }
        OperationSize::Word => {
            let source = pc.fetch_next_word(mem);

            let ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
                instr_word,
                reg,
                mem,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_word(pc, reg, mem, true);

            let add_result = Cpu::sub_words(source, dest);

            add_result.status_register_result.status_register
        }
        OperationSize::Long => {
            let source = pc.fetch_next_long(mem);

            let ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
                instr_word,
                reg,
                mem,
                |instr_word| Ok(operation_size),
                3,
                0,
            )?;
            let dest = ea_data.get_value_long(pc, reg, mem, true);

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result.status_register
        }
    };

    let status_register_result = StatusRegisterResult {
        status_register,
        status_register_mask: STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE,
    };

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.fetch_next_word(mem);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    let immediate_data = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            format!("#${:02X}", pc.fetch_next_byte(mem))
        }
        OperationSize::Word => format!("#${:04X}", pc.fetch_next_word(mem)),
        OperationSize::Long => format!("#${:08X}", pc.fetch_next_long(mem)),
    };

    let ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(operation_size),
        3,
        0,
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, mem);

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("CMPI.{}", ea_data.operation_size.get_format()),
        format!("{},{}", immediate_data, ea_format),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        mem::rammemory::RamMemory,
        register::{STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_ZERO},
    };

    // cmpi byte

    #[test]
    fn cmpi_byte_set_negative() {
        // arrange
        let code = [0x0c, 0x00, 0x00, 0xff].to_vec(); // CMPI.B #$FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0xF0);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CMPI.B"),
                String::from("#$FF,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0xF0, cpu.register.get_d_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
    }

    #[test]
    fn cmpi_byte_clear_negative() {
        // arrange
        let code = [0x0c, 0x00, 0x00, 0x50].to_vec(); // CMPI.B #$50,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x7f);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_NEGATIVE);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CMPI.B"),
                String::from("#$50,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x7f, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
    }

    // cmpi word

    #[test]
    fn cmpi_word_set_zero() {
        // arrange
        let code = [0x0c, 0x40, 0x50, 0xff].to_vec(); // CMPI.B #$50FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x50FF);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CMPI.W"),
                String::from("#$50FF,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x50FF, cpu.register.get_d_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn cmpi_word_clear_zero() {
        // arrange
        let code = [0x0c, 0x40, 0x50, 0x50].to_vec(); // CMPI.W #$5050,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x5040);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("CMPI.W"),
                String::from("#$5050,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5040, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    }

    // cmpi long

    #[test]
    fn cmpi_long_set_zero() {
        // arrange
        let code = [0x0c, 0x80, 0x55, 0x55, 0x50, 0xff].to_vec(); // CMPI.B #$555550FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x555550FF);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(0x0000);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("CMPI.L"),
                String::from("#$555550FF,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x555550FF, cpu.register.get_d_reg_long(0));
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn cmpi_long_clear_zero() {
        // arrange
        let code = [0x0c, 0x80, 0x55, 0x55, 0x50, 0x50].to_vec(); // CMPI.L #$55555050,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x55555040);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("CMPI.L"),
                String::from("#$55555050,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x55555040, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
    }

    #[test]
    fn cmpi_long_immediate_data_to_absolute_short() {
        // arrange
        let code = [0x0C, 0xB8, 0x4C, 0x4F, 0x57, 0x4D, 0x00, 0x00].to_vec(); // CMPI.L #$4C4F574D,($0000).W
        let mem_range = RamMemory::from_bytes(0x00000000, [0x4C, 0x4F, 0x57, 0x4D].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_ZERO);

        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("CMPI.L"),
                String::from("#$4C4F574D,($0000).W")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x4C4F574D, cpu.memory.get_long(0x0000));
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
    }
}
