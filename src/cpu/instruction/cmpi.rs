use crate::{
    cpu::{instruction::OperationSize, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};

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
) -> Result<StepResult, StepError> {
    let instr_word = pc.peek_next_word(mem);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        reg,
        mem,
        Some(operation_size),
    )?;

    let status_register_result = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let source = pc.fetch_next_byte(mem);
            let dest = ea_data.get_value_byte(pc, reg, mem, true);

            let add_result = Cpu::sub_bytes(source, dest);

            add_result.status_register_result
        }
        OperationSize::Word => {
            let source = pc.fetch_next_word(mem);
            let dest = ea_data.get_value_word(pc, reg, mem, true);

            let add_result = Cpu::sub_words(source, dest);

            add_result.status_register_result
        }
        OperationSize::Long => {
            let source = pc.fetch_next_long(mem);
            let dest = ea_data.get_value_long(pc, reg, mem, true);

            let add_result = Cpu::sub_longs(source, dest);

            add_result.status_register_result
        }
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);

    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let instr_word = pc.peek_next_word(mem);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word);
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        reg,
        mem,
        Some(operation_size),
    )?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, None, reg, mem);

    let immediate_data = match operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            format!("#${:02X}", pc.fetch_next_byte(mem))
        }
        OperationSize::Word => format!("#${:04X}", pc.fetch_next_word(mem)),
        OperationSize::Long => format!("#${:08X}", pc.fetch_next_long(mem)),
    };

    Ok(GetDisassemblyResult::from_pc(
        pc,
        format!("CMPI.{}", operation_size.get_format()),
        format!("{},{}", immediate_data, ea_format),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        register::{STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_ZERO},
    };

    // cmpi byte

    #[test]
    fn cmpi_byte_set_negative() {
        // arrange
        let code = [0x0c, 0x00, 0x00, 0xff].to_vec(); // CMPI.B #$FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0xF0;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0xF0, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_negative_set());
    }

    #[test]
    fn cmpi_byte_clear_negative() {
        // arrange
        let code = [0x0c, 0x00, 0x00, 0x50].to_vec(); // CMPI.B #$50,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x7f;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_NEGATIVE;

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
        assert_eq!(0x7f, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_negative_set());
    }

    // cmpi word

    #[test]
    fn cmpi_word_set_zero() {
        // arrange
        let code = [0x0c, 0x40, 0x50, 0xff].to_vec(); // CMPI.B #$50FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x50FF;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x50FF, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmpi_word_clear_zero() {
        // arrange
        let code = [0x0c, 0x40, 0x50, 0x50].to_vec(); // CMPI.W #$5050,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x5040;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_ZERO;

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
        assert_eq!(0x5040, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_zero_set());
    }

    // cmpi long

    #[test]
    fn cmpi_long_set_zero() {
        // arrange
        let code = [0x0c, 0x80, 0x55, 0x55, 0x50, 0xff].to_vec(); // CMPI.B #$555550FF,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x555550FF;
        cpu.register.reg_sr = 0x0000;

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
        assert_eq!(0x555550FF, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_zero_set());
    }

    #[test]
    fn cmpi_long_clear_zero() {
        // arrange
        let code = [0x0c, 0x80, 0x55, 0x55, 0x50, 0x50].to_vec(); // CMPI.L #$55555050,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x55555040;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_ZERO;

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
        assert_eq!(0x55555040, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_zero_set());
    }
}
