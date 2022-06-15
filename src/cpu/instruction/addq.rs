use crate::{
    cpu::{instruction::PcResult, Cpu},
    mem::Mem,
    register::{ProgramCounter, Register},
};

use super::{DisassemblyResult, InstructionExecutionResult, OperationSize};

// Instruction State
// =================
// step: TODO
// step cc: TODO
// get_disassembly: TODO

// 020+ step: TODO
// 020+ get_disassembly: TODO

// TODO: test Areg writes doesn't alter CCs
// TODO: test Areg writes alters entire Long

pub fn step<'a>(
    pc: &mut ProgramCounter,
    reg: &mut Register,
    mem: &mut Mem,
) -> InstructionExecutionResult {
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;

    let size = Cpu::extract_size000110_from_bit_pos_6(ea_data.instr_word);
    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    let data = Cpu::extract_3_bit_data_1_to_8_from_word_at_pos(ea_data.instr_word, 9);
    let status_register_result = match size {
        OperationSize::Byte => {
            let ea_value = Cpu::get_ea_value_byte_with_address(ea_mode, pc, reg, mem);
            let add_result = Cpu::add_bytes(data, ea_value.value);
            mem.set_byte(ea_value.address, add_result.result);

            add_result.status_register_result
        }
        OperationSize::Word => {
            let ea_value = Cpu::get_ea_value_word_with_address(ea_mode, pc, reg, mem);
            let add_result = Cpu::add_words(data as u16, ea_value.value);
            mem.set_word(ea_value.address, add_result.result);

            add_result.status_register_result
        }
        OperationSize::Long => {
            let ea_value = Cpu::get_ea_value_long_with_address(ea_mode, pc, reg, mem);
            let add_result = Cpu::add_longs(data as u32, ea_value.value);
            mem.set_long(ea_value.address, add_result.result);

            add_result.status_register_result
        }
    };

    reg.reg_sr = status_register_result.merge_status_register(reg.reg_sr);

    InstructionExecutionResult::Done {
        pc_result: PcResult::Increment,
    }
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> DisassemblyResult {
    let ea_data = pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(mem);
    let ea_mode = ea_data.ea_mode;
    let size = Cpu::extract_size000110_from_bit_pos_6(ea_data.instr_word);
    let ea_format = Cpu::get_ea_format(ea_mode, pc, None, reg, mem);
    let data = Cpu::extract_3_bit_data_1_to_8_from_word_at_pos(ea_data.instr_word, 9);
    match size {
        OperationSize::Byte => DisassemblyResult::from_pc(
            pc,
            String::from("ADDQ.B"),
            format!("#${:X},{}", data, ea_format),
        ),
        OperationSize::Word => DisassemblyResult::from_pc(
            pc,
            String::from("ADDQ.W"),
            format!("#${:X},{}", data, ea_format),
        ),
        OperationSize::Long => DisassemblyResult::from_pc(
            pc,
            String::from("ADDQ.L"),
            format!("#${:X},{}", data, ea_format),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::instruction::DisassemblyResult,
        register::{
            STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
            STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
        },
    };

    #[test]
    fn addq_data_to_data_register_direct_byte() {
        // arrange
        let code = [0x5a, 0x18, /* DC */ 0x10].to_vec(); // ADDQ.B #$5,(A0)+
                                                         // DC.B $10
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0xC00002;
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
                0xC00002,
                String::from("ADDQ.B"),
                String::from("#$5,(A0)+")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x15, cpu.memory.get_byte(0xC00002));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn addq_data_to_data_register_direct_word() {
        // arrange
        let code = [0x50, 0x5b, /* DC */ 0x60, 0x20].to_vec(); // ADDQ.W #$8,(A3)+
                                                               // DC.W $6020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[3] = 0xC00002;
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
                0xC00002,
                String::from("ADDQ.W"),
                String::from("#$8,(A3)+")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x6028, cpu.memory.get_word(0xC00002));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn addq_data_to_data_register_direct_long() {
        // arrange
        let code = [0x52, 0x9d, /* DC */ 0x60, 0x70, 0x80, 0x20].to_vec(); // ADDQ.L #$1,(A5)+
                                                                           // DC.W $60708020
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[5] = 0xC00002;
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
                0xC00002,
                String::from("ADDQ.L"),
                String::from("#$1,(A5)+")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x60708021, cpu.memory.get_long(0xC00002));
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
