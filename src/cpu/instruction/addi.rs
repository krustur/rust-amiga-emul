use super::{GetDisassemblyResult, GetDisassemblyResultError, StepError, StepResult};
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
) -> Result<StepResult, StepError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Cpu::extract_size000110_from_bit_pos_6(instr_word)
        })?;
    let ea_mode = ea_data.ea_mode;
    let status_register_result = match ea_data.operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let data = pc.fetch_next_byte(mem);
            let ea_value = ea_data.get_value_byte(pc, reg, mem, false);
            let add_result = Cpu::add_bytes(data, ea_value);
            ea_data.set_value_byte(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        OperationSize::Word => {
            let data = pc.fetch_next_word(mem);
            let ea_value = ea_data.get_value_word(pc, reg, mem, false);
            let add_result = Cpu::add_words(data, ea_value);
            ea_data.set_value_word(pc, reg, mem, add_result.result, true);
            add_result.status_register_result
        }
        OperationSize::Long => {
            let data = pc.fetch_next_long(mem);
            let ea_value = ea_data.get_value_long(pc, reg, mem, false);
            let add_result = Cpu::add_longs(data, ea_value);
            ea_data.set_value_long(pc, reg, mem, add_result.result, true);
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
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Cpu::extract_size000110_from_bit_pos_6(instr_word)
        })?;
    let ea_mode = ea_data.ea_mode;
    let ea_format = Cpu::get_ea_format(ea_mode, pc, Some(ea_data.operation_size), reg, mem);
    match ea_data.operation_size {
        OperationSize::Byte => {
            pc.skip_byte();
            let data = pc.fetch_next_byte(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("ADDI.B"),
                format!("#${:02X},{}", data, ea_format),
            ))
        }
        OperationSize::Word => {
            let data = pc.fetch_next_word(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("ADDI.W"),
                format!("#${:04X},{}", data, ea_format),
            ))
        }
        OperationSize::Long => {
            let data = pc.fetch_next_long(mem);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from("ADDI.L"),
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
    fn immediate_data_to_data_register_direct_byte() {
        // arrange
        let code = [0x06, 0x07, 0x00, 0x23].to_vec(); // ADDI.B #$23,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[7] = 0x00004321;
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
                String::from("ADDI.B"),
                String::from("#$23,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x4344, cpu.register.reg_d[7]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_data_register_direct_word() {
        // arrange
        let code = [0x06, 0x47, 0x12, 0x34].to_vec(); // ADDI.W #$1234,D7
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[7] = 0x00004321;
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
                String::from("ADDI.W"),
                String::from("#$1234,D7")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x5555, cpu.register.reg_d[7]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn immediate_data_to_data_register_direct_long() {
        // arrange
        let code = [0x06, 0x80, 0x76, 0x85, 0x76, 0x85].to_vec(); // ADDI.L #$76857685,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_d[0] = 0x10101010;
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
                0xC00006,
                String::from("ADDI.L"),
                String::from("#$76857685,D0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x86958695, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
