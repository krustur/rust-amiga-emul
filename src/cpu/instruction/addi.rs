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
    let instr_word = pc.fetch_next_word(mem);
    let operation_size = Cpu::extract_size000110_from_bit_pos_6(instr_word)?;
    let status_register_result = match operation_size {
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

            let add_result = Cpu::add_bytes(source, dest);
            ea_data.set_value_byte(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
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

            let add_result = Cpu::add_words(source, dest);
            ea_data.set_value_word(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
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

            let add_result = Cpu::add_longs(source, dest);
            ea_data.set_value_long(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
    };

    reg.reg_sr.merge_status_register(status_register_result);

    Ok(())
    // let ea_data =
    //     pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
    //         Cpu::extract_size000110_from_bit_pos_6(instr_word)
    //     })?;
    // let status_register_result = match ea_data.operation_size {
    //     OperationSize::Byte => {
    //         pc.skip_byte();
    //         let data = pc.fetch_next_byte(mem);
    //         let ea_value = ea_data.get_value_byte(pc, reg, mem, false);
    //         let add_result = Cpu::add_bytes(data, ea_value);
    //         ea_data.set_value_byte(pc, reg, mem, add_result.result, true);
    //         add_result.status_register_result
    //     }
    //     OperationSize::Word => {
    //         let data = pc.fetch_next_word(mem);
    //         let ea_value = ea_data.get_value_word(pc, reg, mem, false);
    //         let add_result = Cpu::add_words(data, ea_value);
    //         ea_data.set_value_word(pc, reg, mem, add_result.result, true);
    //         add_result.status_register_result
    //     }
    //     OperationSize::Long => {
    //         let data = pc.fetch_next_long(mem);
    //         let ea_value = ea_data.get_value_long(pc, reg, mem, false);
    //         let add_result = Cpu::add_longs(data, ea_value);
    //         ea_data.set_value_long(pc, reg, mem, add_result.result, true);
    //         add_result.status_register_result
    //     }
    // };

    // reg.reg_sr.merge_status_register(status_register_result);

    // Ok(())
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
        format!("ADDI.{}", ea_data.operation_size.get_format()),
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
    fn addi_byte_immediate_data_to_data_register_direct() {
        // arrange
        let code = [0x06, 0x07, 0x00, 0x23].to_vec(); // ADDI.B #$23,D7
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
                String::from("ADDI.B"),
                String::from("#$23,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x4344, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn addi_byte_immediate_data_to_absolute_short() {
        // arrange
        let code = [0x06, 0x38, 0x00, 0x38, 0x40, 0x00].to_vec(); // ADDI.B #$38,($4000).W
        let mem_range = RamMemory::from_bytes(0x00004000, [0x4C].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
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
                0xC00006,
                String::from("ADDI.B"),
                String::from("#$38,($4000).W")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x84, cpu.memory.get_byte(0x00004000));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn addi_word_immediate_data_to_data_register_direct() {
        // arrange
        let code = [0x06, 0x47, 0x12, 0x34].to_vec(); // ADDI.W #$1234,D7
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
                String::from("ADDI.W"),
                String::from("#$1234,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5555, cpu.register.get_d_reg_long(7));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn addi_word_immediate_data_to_absolute_long() {
        // arrange
        let code = [0x06, 0x79, 0x38, 0x78, 0x00, 0x04, 0x00, 0x00].to_vec(); // ADDI.W #$3878,($40000).L
        let mem_range = RamMemory::from_bytes(0x00040000, [0x3C, 0x09].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
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
                0xC00008,
                String::from("ADDI.W"),
                String::from("#$3878,($00040000).L")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x7481, cpu.memory.get_word(0x00040000));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn addi_long_immediate_data_to_data_register_direct() {
        // arrange
        let code = [0x06, 0x80, 0x76, 0x85, 0x76, 0x85].to_vec(); // ADDI.L #$76857685,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_d_reg_long(0, 0x10101010);
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
                String::from("ADDI.L"),
                String::from("#$76857685,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x86958695, cpu.register.get_d_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn addi_long_immediate_data_to_absolute_long() {
        // arrange
        let code = [0x06, 0xb9, 0x38, 0x78, 0x45, 0x45, 0x00, 0x04, 0x00, 0x00].to_vec(); // ADDI.L #$38784545,($00040000).L
        let mem_range = RamMemory::from_bytes(0x00040000, [0xeC, 0x09, 0x00, 0x01].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
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
                0xC0000a,
                String::from("ADDI.L"),
                String::from("#$38784545,($00040000).L")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x24814546, cpu.memory.get_long(0x00040000));
        assert_eq!(true, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }
}
