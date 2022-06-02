use crate::{
    cpu::Cpu,
    mem::Mem,
    register::Register,
};

use super::{
    DisassemblyResult, EffectiveAddressingMode, InstructionExecutionResult, OperationSize, PcResult,
};

// Instruction State
// =================
// step-logic: TODO
// step cc: TODO (none)
// step tests: TODO
// get_disassembly: DONE
// get_disassembly tests: TODO

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    let src_ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let src_ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos_3(instr_word);

    let dst_ea_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let dst_ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos(instr_word, 6);

    let size = Cpu::extract_size011110_from_bit_pos(instr_word, 12);
    
    
    
    match size {
        OperationSize::Byte => {
            let ea_value = Cpu::get_ea_value_unsigned_byte(src_ea_mode, src_ea_register, instr_address + 2, reg, mem);
            let set_result = Cpu::set_ea_value_unsigned_byte(dst_ea_mode, dst_ea_register, instr_address + 2 + (ea_value.num_extension_words << 1), ea_value.value, reg, mem);
            reg.reg_sr = set_result.status_register_result.merge_status_register(reg.reg_sr);
            InstructionExecutionResult::Done {
                pc_result: PcResult::Increment(2 + (ea_value.num_extension_words << 1) + (set_result.num_extension_words << 1)),
            }
        }
        OperationSize::Word => {
            let ea_value = Cpu::get_ea_value_unsigned_word(src_ea_mode, src_ea_register, instr_address + 2, reg, mem);
            let set_result = Cpu::set_ea_value_unsigned_word(dst_ea_mode, dst_ea_register, instr_address + 2 + (ea_value.num_extension_words << 1), ea_value.value, reg, mem);
            reg.reg_sr = set_result.status_register_result.merge_status_register(reg.reg_sr);
            InstructionExecutionResult::Done {
                pc_result: PcResult::Increment(2 + (ea_value.num_extension_words << 1) + (set_result.num_extension_words << 1)),
            }
        }
        OperationSize::Long => {
            let ea_value = Cpu::get_ea_value_unsigned_long(src_ea_mode, src_ea_register, instr_address + 2, reg, mem);
            let set_result = Cpu::set_ea_value_unsigned_long(dst_ea_mode, dst_ea_register, instr_address + 2 + (ea_value.num_extension_words << 1), ea_value.value, reg, mem);
            reg.reg_sr = set_result.status_register_result.merge_status_register(reg.reg_sr);
            InstructionExecutionResult::Done {
                pc_result: PcResult::Increment(2 + (ea_value.num_extension_words << 1) + (set_result.num_extension_words << 1)),
            }
        }
    }


    
    // InstructionExecutionResult::Done {
    //     pc_result: PcResult::Increment(2 + (ea_value.num_extension_words << 1)),
    // }
}

pub fn get_disassembly<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let src_ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let src_ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos_3(instr_word);

    let dst_ea_register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    let dst_ea_mode = Cpu::extract_effective_addressing_mode_from_bit_pos(instr_word, 6);

    let size = Cpu::extract_size011110_from_bit_pos(instr_word, 12);

    let src_ea_debug = Cpu::get_ea_format(
        src_ea_mode,
        src_ea_register,
        instr_address + 2,
        Some(size),
        reg,
        mem,
    );
    let dst_ea_debug = Cpu::get_ea_format(
        dst_ea_mode,
        dst_ea_register,
        instr_address + 2 + (src_ea_debug.num_extension_words << 1),
        Some(size),
        reg,
        mem,
    );

    let name = match dst_ea_mode {
        EffectiveAddressingMode::ARegDirect => match size {            
            OperationSize::Byte => panic!("AddressRegisterDirect as destination only available for Word and Long"),
            OperationSize::Word => String::from("MOVEA.W"),
            OperationSize::Long => String::from("MOVEA.L"),
        },
        _ => match size {
            OperationSize::Byte => String::from("MOVE.B"),
            OperationSize::Word => String::from("MOVE.W"),
            OperationSize::Long => String::from("MOVE.L"),
        },
    };

    DisassemblyResult::Done {
        name,
        operands_format: format!("{},{}", src_ea_debug.format, dst_ea_debug.format),
        instr_address,
        next_instr_address: instr_address
            + 2
            + (src_ea_debug.num_extension_words << 1)
            + (dst_ea_debug.num_extension_words << 1),
    }
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
    fn data_reg_direct_to_absolute_long_addressing_mode() {
        // arrange
        let code = [0x13, 0xc0, 0x00, 0x09, 0x00, 0x00].to_vec(); // MOVE.B D0,($00090000).L
        let mem_range = MemRange::from_bytes(0x00090000, [0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_d[0] = 0x34;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.B"),
                operands_format: String::from("D0,($00090000).L"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00006
            },
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x34, cpu.memory.get_unsigned_byte(0x00090000));
        assert_eq!(0x00C00006, cpu.register.reg_pc);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_direct_to_data_reg_direct() {
        // arrange
        let code = [0x30, 0x08].to_vec(); // MOVE.W A0,D0
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00000000;
        cpu.register.reg_d[0] = 0xffffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            // | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            // | STATUS_REGISTER_MASK_EXTEND
            ;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.W"),
                operands_format: String::from("A0,D0"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0xffff0000, cpu.register.reg_d[0]);
        assert_eq!(0x00C00002, cpu.register.reg_pc);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_to_address_reg_direct() {
        // arrange
        let code = [0x32, 0x50].to_vec(); // MOVE.W (A0),A1
        let mem_range = MemRange::from_bytes(0x00090000, [0x12, 0x34].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_a[0] = 0x00090000;
        cpu.register.reg_a[1] = 0xffffffff;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_NEGATIVE;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVEA.W"),
                operands_format: String::from("(A0),A1"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0xffff1234, cpu.register.reg_a[1]);
        assert_eq!(0x00C00002, cpu.register.reg_pc);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_post_increment_to_address_reg_indirect() {
        // arrange
        let code = [0x14, 0x99].to_vec(); // MOVE.B (A1)+,(A2)
        let mem_range = MemRange::from_bytes(0x00090000, [0xf0, 0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_a[1] = 0x00090000;
        cpu.register.reg_a[2] = 0x00090001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.B"),
                operands_format: String::from("(A1)+,(A2)"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00090001, cpu.register.reg_a[1]);
        assert_eq!(0x00C00002, cpu.register.reg_pc);
    
        assert_eq!(0xf0, cpu.memory.get_unsigned_byte(0x00090001));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_pre_decrement_to_address_reg_indirect_with_post_increment() {
        // arrange
        let code = [0x26, 0xe2].to_vec(); // MOVE.B -(A2),(A3)+
        let mem_range = MemRange::from_bytes(0x00090000, [0xff, 0xff, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_a[2] = 0x00090004;
        cpu.register.reg_a[3] = 0x00090004;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.L"),
                operands_format: String::from("-(A2),(A3)+"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00090000, cpu.register.reg_a[2]);
        assert_eq!(0x00090008, cpu.register.reg_a[3]);
        assert_eq!(0x00C00002, cpu.register.reg_pc);
        assert_eq!(0xfffffff0, cpu.memory.get_unsigned_long(0x00090004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_displacement_to_address_reg_indirect_with_pre_decrement() {
        // arrange
        let code = [0x29, 0x2b, 0x7f, 0xf0].to_vec(); // MOVE.L ($7FF0,A3),-(A4)
        let mem_range = MemRange::from_bytes(0x00090000, [0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_a[3] = 0x00090000 - 0x7ff0;
        cpu.register.reg_a[4] = 0x00090008;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.L"),
                operands_format: String::from("($7FF0,A3) [32752],-(A4)"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00004
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00088010, cpu.register.reg_a[3]);
        assert_eq!(0x00090004, cpu.register.reg_a[4]);
        assert_eq!(0x00C00004, cpu.register.reg_pc);
        assert_eq!(0x12345678, cpu.memory.get_unsigned_long(0x00090004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn address_reg_indirect_with_index_to_address_reg_indirect_with_displacement() {
        // arrange
        let code = [0x2b, 0x74, 0x0e, 0x80, 0x80, 0x10].to_vec(); // MOVE.L ($80,A4,D0.L*8),$8010(A5)
        let mem_range = MemRange::from_bytes(0x00090000, [0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_d[0] = 0x00000100;
        cpu.register.reg_a[4] = 0x0008f800 + 0x80; // $80 displacement is -128 => +128 => 0x80
        cpu.register.reg_a[5] = 0x00090004 + 0x7ff0; // $8010 displacement is -32752 => +32752 => 0x7ff0
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.L"),
                operands_format: String::from("($80,A4,D0.L*8) [-128],($8010,A5) [-32752]"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00006
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00006, cpu.register.reg_pc);
        assert_eq!(0x12345678, cpu.memory.get_unsigned_long(0x00090004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn absolute_short_addressing_mode_to_address_reg_indirect_with_index() {
        // arrange
        let code = [0x1d, 0xb8, 0x90, 0x00, 0x70, 0x7c].to_vec(); // MOVE.B ($9000).W,($7C,A6,D7.W)
        let mem_range = MemRange::from_bytes(0xffff9000, [0x00, 0xff].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_d[7] = 0xffffff00;
        cpu.register.reg_a[6] = 0xffff9001 - 0x7c + 0x100; 
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.B"),
                operands_format: String::from("($9000).W [$FFFF9000],($7C,A6,D7.W) [124]"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00006
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00006, cpu.register.reg_pc);
        assert_eq!(0x00, cpu.memory.get_unsigned_byte(0xffff9001));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn absolute_long_addressing_mode_to_address_short_addressing_mode() {
        // arrange
        let code = [0x11, 0xf9, 0x00, 0xC0, 0x00, 0x08, 0x90, 0x00, 0xff].to_vec(); // MOVE.B ($C00008).L,(9000).W
                                                                                             // DC.B $FF
        let mem_range = MemRange::from_bytes(0xffff9000, [0x88].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));
        cpu.register.reg_d[7] = 0xffffff00;
        cpu.register.reg_a[6] = 0xffff9001 - 0x7c + 0x100; 
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.B"),
                operands_format: String::from("($00C00008).L,($9000).W [$FFFF9000]"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00008
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00008, cpu.register.reg_pc);
        assert_eq!(0xff, cpu.memory.get_unsigned_byte(0xffff9000));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn program_counter_inderect_with_displacement_mode_to_address_long_addressing_modex() {
        // arrange
        let code = [0x33, 0xfa, 0x80, 0x00, 0x00, 0xC0, 0x00, 0x08, 0x00, 0x00].to_vec(); // MOVE.W ($8000,PC),($C00008).L
                                                                                                   // DC.B $00,$00
        let mem_range = MemRange::from_bytes(0x00BF8002, [0xab, 0xba].to_vec());
        let mut cpu = crate::instr_test_setup(code, Some(mem_range));        
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.W"),
                operands_format: String::from("($8000,PC) [$00BF8002],($00C00008).L"),
                instr_address: 0xC00000,
                next_instr_address: 0xC00008
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00C00008, cpu.register.reg_pc);
        assert_eq!(0xabba, cpu.memory.get_unsigned_word(0x00BF8002));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
