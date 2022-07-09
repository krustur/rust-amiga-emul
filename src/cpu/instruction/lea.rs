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
    // TODO: Tests
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(OperationSize::Long)
        })?;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
    let ea_address = ea_data.get_address(pc, reg, mem);

    reg.reg_a[register] = ea_address;
    Ok(())
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    // TODO: Tests
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            Ok(OperationSize::Long)
        })?;
    let ea_mode = ea_data.ea_mode;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;

    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    Ok(GetDisassemblyResult::from_pc(
        pc,
        String::from("LEA"),
        format!("{},A{}", ea_format, register),
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
    fn lea_absolute_short_addressing_mode_to_a0() {
        // arrange
        let code = [0x41, 0xf8, 0x05, 0x00].to_vec(); // LEA ($0500).W,A0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00000000;
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
                String::from("LEA"),
                String::from("($0500).W,A0")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x500, cpu.register.reg_a[0]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn lea_absolute_long_addressing_mode_to_a0() {
        // arrange
        let code = [0x43, 0xf9, 0x00, 0xf8, 0x00, 0x00].to_vec(); // LEA ($00F80000).L,A1
        let mem_range = RamMemory::from_bytes(0xf80000, [0x00, 0x00, 0x00, 0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.reg_a[0] = 0x00000000;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("LEA"),
                String::from("($00F80000).L,A1")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00F80000, cpu.register.reg_a[1]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
