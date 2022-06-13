use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{DisassemblyResult, InstructionExecutionResult};

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
    mem: &mut Mem,
) -> InstructionExecutionResult {
    // TODO: Tests
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9);
    let ea_value = Cpu::get_ea(ea_mode, pc, None, reg, mem);

    reg.reg_a[register] = ea_value.address;
    InstructionExecutionResult::Done {
        pc_result: PcResult::Increment,
    }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    // TODO: Tests
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;
    let register = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9);

    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    DisassemblyResult::from_pc(
        pc,
        String::from("LEA"),
        format!("{},A{}", ea_format, register),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::DisassemblyResult,
        memrange::MemRange,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn absolute_short_addressing_mode_to_a0() {
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
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(true, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn absolute_long_addressing_mode_to_a0() {
        // arrange
        let code = [0x43, 0xf9, 0x00, 0xf8, 0x00, 0x00].to_vec(); // LEA ($00F80000).L,A1
        let mem_range = MemRange::from_bytes(0xf80000, [0x00, 0x00, 0x00, 0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_a[0] = 0x00000000;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::from_address_and_address_next(
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
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
