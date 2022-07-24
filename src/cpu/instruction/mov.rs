use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, Instruction,
    OperationSize, StepError,
};
use crate::{
    cpu::Cpu,
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

pub fn match_check(instruction: &Instruction, instr_word: u16) -> bool {
    match crate::cpu::match_check(instruction, instr_word) {
        true => match crate::cpu::match_check_size011110_from_bit_pos(instr_word, 12) {
            true => match crate::cpu::match_check_ea_all_addressing_modes_pos_0(instr_word) {
                true => {
                    crate::cpu::match_check_ea_only_data_alterable_addressing_modes_and_areg_direct_pos(
                                    instr_word, 6, 9,
                                )
                }
                false => false,
            },
            false => false,
        },
        false => false,
    }
}

pub fn step<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> Result<(), StepError> {
    let src_ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(Cpu::extract_size011110_from_bit_pos(instr_word, 12).unwrap()),
    )?;

    let dst_ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
        src_ea_data.instr_word,
        reg,
        mem,
        |_| Ok(src_ea_data.operation_size),
        6,
        9,
    )?;
    let dst_ea_mode = dst_ea_data.ea_mode;

    let set_result = match src_ea_data.operation_size {
        OperationSize::Byte => {
            let ea_value = src_ea_data.get_value_byte(pc, reg, mem, true);
            dst_ea_data.set_value_byte(pc, reg, mem, ea_value, true)
        }
        OperationSize::Word => {
            let ea_value = src_ea_data.get_value_word(pc, reg, mem, true);
            dst_ea_data.set_value_word(pc, reg, mem, ea_value, true)
        }
        OperationSize::Long => {
            let ea_value = src_ea_data.get_value_long(pc, reg, mem, true);
            dst_ea_data.set_value_long(pc, reg, mem, ea_value, true)
        }
    };

    reg.reg_sr
        .merge_status_register(set_result.status_register_result);
    Ok(())
}

pub fn get_disassembly<'a>(
    instr_word: u16,
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let src_ea_data = pc.get_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(
        instr_word,
        reg,
        mem,
        |instr_word| Ok(Cpu::extract_size011110_from_bit_pos(instr_word, 12).unwrap()),
    )?;
    let src_ea_mode = src_ea_data.ea_mode;

    let dst_ea_data = pc.get_effective_addressing_data_from_instr_word_bit_pos(
        src_ea_data.instr_word,
        reg,
        mem,
        |_| Ok(src_ea_data.operation_size),
        6,
        9,
    )?;
    let dst_ea_mode = dst_ea_data.ea_mode;

    // let size = Cpu::extract_size011110_from_bit_pos(src_ea_data.instr_word, 12)?;

    let src_ea_debug = Cpu::get_ea_format(src_ea_mode, pc, Some(src_ea_data.operation_size), mem);
    let dst_ea_debug = Cpu::get_ea_format(dst_ea_mode, pc, Some(src_ea_data.operation_size), mem);

    let name = match dst_ea_mode {
        EffectiveAddressingMode::ARegDirect {
            ea_register: register,
        } => match src_ea_data.operation_size {
            OperationSize::Byte => {
                panic!("AddressRegisterDirect as destination only available for Word and Long")
            }
            OperationSize::Word => String::from("MOVEA.W"),
            OperationSize::Long => String::from("MOVEA.L"),
        },
        _ => match src_ea_data.operation_size {
            OperationSize::Byte => String::from("MOVE.B"),
            OperationSize::Word => String::from("MOVE.W"),
            OperationSize::Long => String::from("MOVE.L"),
        },
    };

    Ok(GetDisassemblyResult::from_pc(
        pc,
        name,
        format!("{},{}", src_ea_debug.format, dst_ea_debug.format),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::GetDisassemblyResult,
        mem::rammemory::RamMemory,
        register::{
            ProgramCounter, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
            STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW,
            STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn data_reg_direct_to_absolute_long_addressing_mode() {
        // arrange
        let code = [0x13, 0xc0, 0x00, 0x09, 0x00, 0x00].to_vec(); // MOVE.B D0,($00090000).L
        let mem_range = RamMemory::from_bytes(0x00090000, [0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0x34);
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
                String::from("MOVE.B"),
                String::from("D0,($00090000).L")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x34, cpu.memory.get_byte(0x00090000));
        assert_eq!(0x00C00006, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_reg_direct_to_data_reg_direct() {
        // arrange
        let code = [0x30, 0x08].to_vec(); // MOVE.W A0,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x00000000);
        cpu.register.set_d_reg_long(0, 0xffffffff);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            // | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE, // | STATUS_REGISTER_MASK_EXTEND
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.W"),
                String::from("A0,D0")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0xffff0000, cpu.register.get_d_reg_long(0));
        assert_eq!(0x00C00002, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_to_address_reg_direct() {
        // arrange
        let code = [0x32, 0x50].to_vec(); // MOVEA.W (A0),A1
        let mem_range = RamMemory::from_bytes(0x00090000, [0x12, 0x34].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(0, 0x00090000);
        cpu.register.set_a_reg_long(1, 0xffffffff);
        cpu.register.reg_sr.set_sr_reg_flags_abcde(
            STATUS_REGISTER_MASK_CARRY
                | STATUS_REGISTER_MASK_OVERFLOW
                | STATUS_REGISTER_MASK_NEGATIVE,
        );
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVEA.W"),
                String::from("(A0),A1")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00001234, cpu.register.get_a_reg_long(1));
        assert_eq!(0x00C00002, cpu.register.reg_pc.get_address());
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_post_increment_to_address_reg_indirect() {
        // arrange
        let code = [0x14, 0x99].to_vec(); // MOVE.B (A1)+,(A2)
        let mem_range = RamMemory::from_bytes(0x00090000, [0xf0, 0x00].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(1, 0x00090000);
        cpu.register.set_a_reg_long(2, 0x00090001);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.B"),
                String::from("(A1)+,(A2)")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(
            0x00090001,
            cpu.register.get_a_reg_long(1),
            "AReg post increment"
        );
        assert_eq!(
            0x00C00002,
            cpu.register.reg_pc.get_address(),
            "PC increment"
        );

        assert_eq!(0xf0, cpu.memory.get_byte(0x00090001), "Result");
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_pre_decrement_to_address_reg_indirect_with_post_increment() {
        // arrange
        let code = [0x26, 0xe2].to_vec(); // MOVE.L -(A2),(A3)+
        let mem_range = RamMemory::from_bytes(
            0x00090000,
            [0xff, 0xff, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00].to_vec(),
        );
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(2, 0x00090004);
        cpu.register.set_a_reg_long(3, 0x00090004);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("MOVE.L"),
                String::from("-(A2),(A3)+")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(
            0x00090000,
            cpu.register.get_a_reg_long(2),
            "AReg pre decrement"
        );
        assert_eq!(
            0x00090008,
            cpu.register.get_a_reg_long(3),
            "AReg post increment"
        );
        assert_eq!(
            0x00C00002,
            cpu.register.reg_pc.get_address(),
            "PC increment"
        );
        assert_eq!(0xfffffff0, cpu.memory.get_long(0x00090004), "Result");
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_displacement_to_address_reg_indirect_with_pre_decrement() {
        // arrange
        let code = [0x29, 0x2b, 0x7f, 0xf0].to_vec(); // MOVE.L ($7FF0,A3),-(A4)
        let mem_range = RamMemory::from_bytes(
            0x00090000,
            [0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00].to_vec(),
        );
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_a_reg_long(3, 0x00090000 - 0x7ff0);
        cpu.register.set_a_reg_long(4, 0x00090008);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVE.L"),
                String::from("($7FF0,A3) [32752],-(A4)")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00088010, cpu.register.get_a_reg_long(3));
        assert_eq!(
            0x00090004,
            cpu.register.get_a_reg_long(4),
            "AReg pre decrement"
        );
        assert_eq!(
            0x00C00004,
            cpu.register.reg_pc.get_address(),
            "PC increment"
        );
        assert_eq!(0x12345678, cpu.memory.get_long(0x00090004), "Result");
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set(), "Carry");
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set(), "Overflow");
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set(), "Zero");
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set(), "Negative");
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set(), "Extend");
    }

    #[test]
    fn address_reg_indirect_with_index_to_address_reg_indirect_with_displacement() {
        // arrange
        let code = [0x2b, 0x74, 0x0e, 0x80, 0x80, 0x10].to_vec(); // MOVE.L ($80,A4,D0.L*8),$8010(A5)
        let mem_range = RamMemory::from_bytes(
            0x00090000,
            [0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00].to_vec(),
        );
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(0, 0x00000100);
        cpu.register.set_a_reg_long(4, 0x0008f800 + 0x80); // $80 displacement is -128 => +128 => 0x80
        cpu.register.set_a_reg_long(5, 0x00090004 + 0x7ff0); // $8010 displacement is -32752 => +32752 => 0x7ff0
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("MOVE.L"),
                String::from("($80,A4,D0.L*8) [-128],($8010,A5) [-32752]")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00006, cpu.register.reg_pc.get_address());
        assert_eq!(0x12345678, cpu.memory.get_long(0x00090004));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn absolute_short_addressing_mode_to_address_reg_indirect_with_index() {
        // arrange
        let code = [0x1d, 0xb8, 0x90, 0x00, 0x70, 0x7c].to_vec(); // MOVE.B ($9000).W,($7C,A6,D7.W)
        let mem_range = RamMemory::from_bytes(0xffff9000, [0x00, 0xff].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(7, 0xffffff00);
        cpu.register.set_a_reg_long(6, 0xffff9001 - 0x7c + 0x100);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00006,
                String::from("MOVE.B"),
                String::from("($9000).W [$FFFF9000],($7C,A6,D7.W) [124]")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00006, cpu.register.reg_pc.get_address());
        assert_eq!(0x00, cpu.memory.get_byte(0xffff9001));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn absolute_long_addressing_mode_to_address_short_addressing_mode() {
        // arrange
        let code = [0x11, 0xf9, 0x00, 0xC0, 0x00, 0x08, 0x90, 0x00, 0xff].to_vec(); // MOVE.B ($C00008).L,(9000).W
                                                                                    // DC.B $FF
        let mem_range = RamMemory::from_bytes(0xffff9000, [0x88].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(7, 0xffffff00);
        cpu.register.set_a_reg_long(6, 0xffff9001 - 0x7c + 0x100);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00008,
                String::from("MOVE.B"),
                String::from("($00C00008).L,($9000).W [$FFFF9000]")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00008, cpu.register.reg_pc.get_address());
        assert_eq!(0xff, cpu.memory.get_byte(0xffff9000));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn program_counter_inderect_with_displacement_mode_to_address_long_addressing_mode() {
        // arrange
        let code = [0x66].to_vec();
        let code_mem_range = RamMemory::from_bytes(
            0x00D00000,
            [0x33, 0xfa, 0x80, 0x00, 0x00, 0xD0, 0x00, 0x08, 0x00, 0x00].to_vec(),
        ); // MOVE.W ($8000,PC),($C00008).L
           // DC.B $00,$00
        let data_mem_range = RamMemory::from_bytes(0x00CF8002, [0xab, 0xba].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(code_mem_range);
        mem_ranges.push(data_mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.reg_pc = ProgramCounter::from_address(0x00D00000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xD00000,
                0xD00008,
                String::from("MOVE.W"),
                String::from("($8000,PC) [$00CF8002],($00D00008).L")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00D00008, cpu.register.reg_pc.get_address());
        assert_eq!(0xabba, cpu.memory.get_word(0x00CF8002));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn program_counter_inderect_with_index_8bit_displacement_mode_to_data_register_direct() {
        // arrange
        let code = [0x2E, 0x3B, 0x5C, 0x80, 0x00, 0x00].to_vec(); // MOVE.L ($80,PC,D5.L*4),D7
                                                                  // DC.B $00,$00
        let mem_range = RamMemory::from_bytes(0x50BFFF82, [0xab, 0xba, 0xba, 0xab].to_vec());
        let mut mem_ranges = Vec::new();
        mem_ranges.push(mem_range);
        let mut cpu = crate::instr_test_setup(code, Some(mem_ranges));
        cpu.register.set_d_reg_long(5, 0x14000000);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVE.L"),
                String::from("($80,PC,D5.L*4) [$50BFFF82],D7")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00004, cpu.register.reg_pc.get_address());
        assert_eq!(0xabbabaab, cpu.memory.get_long(0x50BFFF82));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(true, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }

    #[test]
    fn absolute_short_mode_to_address_register_word_direct() {
        // arrange
        let code = [0x30, 0x7C, 0x00, 0x08].to_vec(); // MOVEA.W #$0008,A0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.set_a_reg_long(0, 0x12345678);
        cpu.register
            .reg_sr
            .set_sr_reg_flags_abcde(STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_OVERFLOW);
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("MOVEA.W"),
                String::from("#$0008,A0")
            ),
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00000008, cpu.register.get_a_reg_long(0));
        assert_eq!(false, cpu.register.reg_sr.is_sr_carry_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_overflow_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_zero_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_negative_set());
        assert_eq!(false, cpu.register.reg_sr.is_sr_extend_set());
    }
}
