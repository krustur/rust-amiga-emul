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
                instr_address: 0x080000,
                next_instr_address: 0x080006
            },
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x34, cpu.memory.get_unsigned_byte(0x00090000));
        assert_eq!(0x00080006, cpu.register.reg_pc);
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
                instr_address: 0x080000,
                next_instr_address: 0x080002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0xffff0000, cpu.register.reg_d[0]);
        assert_eq!(0x00080002, cpu.register.reg_pc);
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
            // | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            // | STATUS_REGISTER_MASK_EXTEND
            ;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVEA.W"),
                operands_format: String::from("(A0),A1"),
                instr_address: 0x080000,
                next_instr_address: 0x080002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0xffff1234, cpu.register.reg_a[1]);
        assert_eq!(0x00080002, cpu.register.reg_pc);
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
            | STATUS_REGISTER_MASK_OVERFLOW
            // | STATUS_REGISTER_MASK_ZERO
            // | STATUS_REGISTER_MASK_NEGATIVE
            // | STATUS_REGISTER_MASK_EXTEND
            ;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.B"),
                operands_format: String::from("(A1)+,(A2)"),
                instr_address: 0x080000,
                next_instr_address: 0x080002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00090001, cpu.register.reg_a[1]);
        assert_eq!(0x00080002, cpu.register.reg_pc);
    
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
            | STATUS_REGISTER_MASK_OVERFLOW
            // | STATUS_REGISTER_MASK_ZERO
            // | STATUS_REGISTER_MASK_NEGATIVE
            // | STATUS_REGISTER_MASK_EXTEND
            ;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            DisassemblyResult::Done {
                name: String::from("MOVE.L"),
                operands_format: String::from("-(A2),(A3)+"),
                instr_address: 0x080000,
                next_instr_address: 0x080002
            },
            debug_result
        );
        // // act
        cpu.execute_next_instruction();
        // // assert
        assert_eq!(0x00090000, cpu.register.reg_a[2]);
        assert_eq!(0x00090008, cpu.register.reg_a[3]);
        assert_eq!(0x00080002, cpu.register.reg_pc);
    
        assert_eq!(0xfffffff0, cpu.memory.get_unsigned_long(0x00090004));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
