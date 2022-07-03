use num_traits::zero;

use super::{
    EffectiveAddressingMode, GetDisassemblyResult, GetDisassemblyResultError, OperationSize,
    StepError, StepResult,
};
use crate::{
    cpu::Cpu,
    mem::Mem,
    register::{ProgramCounter, Register, STATUS_REGISTER_MASK_ZERO},
};

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
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        })?;

    let bit_number = match ea_data.instr_word & 0x0100 {
        0x0100 => {
            // Bit Number Dynamic, Specified in a Register
            println!("Bit Number Dynamic, Specified in a Register");
            let dreg = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
            match ea_data.operation_size {
                OperationSize::Long => Cpu::get_byte_from_long(reg.reg_d[dreg]) % 32,
                _ => Cpu::get_byte_from_long(reg.reg_d[dreg]) % 8,
            }
        }
        _ => {
            // Bit Number Static, Specified as Immediate Data
            match ea_data.operation_size {
                OperationSize::Long => Cpu::get_byte_from_word(pc.fetch_next_word(mem)) % 32,
                _ => Cpu::get_byte_from_word(pc.fetch_next_word(mem)) % 8,
            }
        }
    };

    let bit_set = match ea_data.operation_size {
        OperationSize::Long => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_long(pc, reg, mem, true);
            println!(
                "testing bit {} [{}] in value ${:08X}",
                bit_number, bit_number_mask, value
            );
            (value & bit_number_mask) != 0
        }
        _ => {
            let bit_number_mask = 1 << bit_number;
            let value = ea_data.get_value_byte(pc, reg, mem, true);
            println!(
                "testing bit {} [{}] in value ${:08X}",
                bit_number, bit_number_mask, value
            );
            (value & bit_number_mask) != 0
        }
    };

    let zero_flag = !bit_set;
    println!("bit_set: {}", bit_set);
    println!("zero_flag: {}", zero_flag);
    println!("sr before: ${:04X}", reg.reg_sr);
    reg.reg_sr = match zero_flag {
        false => reg.reg_sr & 0b0000000000011011,
        true => reg.reg_sr | STATUS_REGISTER_MASK_ZERO,
    };
    println!("sr after: ${:04X}", reg.reg_sr);

    Ok(StepResult::Done {})
}

pub fn get_disassembly<'a>(
    pc: &mut ProgramCounter,
    reg: &Register,
    mem: &Mem,
) -> Result<GetDisassemblyResult, GetDisassemblyResultError> {
    let ea_data =
        pc.fetch_effective_addressing_data_from_bit_pos_3_and_reg_pos_0(reg, mem, |instr_word| {
            match instr_word & 0x0038 {
                0x0000 => {
                    // DRegDirect
                    Ok(OperationSize::Long)
                }
                _ => {
                    // other
                    Ok(OperationSize::Byte)
                }
            }
        })?;

    let ea_format = Cpu::get_ea_format(ea_data.ea_mode, pc, Some(OperationSize::Byte), reg, mem);

    match ea_data.instr_word & 0x0100 {
        0x0100 => {
            // Bit Number Dynamic, Specified in a Register
            // println!("Bit Number Dynamic, Specified in a Register");
            let dreg = Cpu::extract_register_index_from_bit_pos(ea_data.instr_word, 9)?;
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from(format!("BTST.{}", ea_data.operation_size.get_format())),
                format!("D{},{}", dreg, ea_format.format),
            ))
        }
        _ => {
            // Bit Number Static, Specified as Immediate Data
            // println!("Bit Number Static, Specified as Immediate Data");
            let bit_number = match ea_data.operation_size {
                OperationSize::Long => pc.fetch_next_word(mem),
                _ => pc.fetch_next_word(mem),
            };
            // println!("bit number: {}", bit_number);
            Ok(GetDisassemblyResult::from_pc(
                pc,
                String::from(format!("BTST.{}", ea_data.operation_size.get_format())),
                format!("#${:02X},{}", bit_number, ea_format.format),
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

    // long (data register direct)

    #[test]
    fn btst_long_bit_number_static_data_register_direct_bit_set() {
        // arrange
        let code = [0x08, 0x01, 0x00, 0x00].to_vec(); // BTST.L #$00,D1
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BTST.L"),
                String::from("#$00,D1")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_long_bit_number_static_data_register_direct_bit_clear() {
        // arrange
        let code = [0x08, 0x02, 0x00, 0x21].to_vec(); // BTST.L #$21,D2
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[2] = 0x0000000d;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BTST.L"),
                String::from("#$21,D2")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_long_bit_number_dynamic_data_register_direct_bit_set() {
        // arrange
        let code = [0x01, 0x03].to_vec(); // BTST.L D0,D3
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[0] = 0x00000002;
        cpu.register.reg_d[3] = 0x0000fff7;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BTST.L"),
                String::from("D0,D3")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_long_bit_number_dynamic_data_data_register_direct_bit_clear() {
        // arrange
        let code = [0x0f, 0x06].to_vec(); // BTST.L D7,D6
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[7] = 0x00000003;
        cpu.register.reg_d[6] = 0x0000fff7;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BTST.L"),
                String::from("D7,D6")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    // byte

    #[test]
    fn btst_byte_bit_number_static_address_register_indirect_bit_set() {
        // arrange
        let code = [0x08, 0x10, 0x00, 0x08, /* DC  */ 0x01].to_vec(); // BTST.B #$08,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_a[0] = 0x00c00004;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BTST.B"),
                String::from("#$08,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_byte_bit_number_static_address_register_indirect_bit_clear() {
        // arrange
        let code = [0x08, 0x10, 0x00, 0x09, /* DC  */ 0x01].to_vec(); // BTST.B #$09,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[2] = 0x0000fffd;
        cpu.register.reg_a[0] = 0x00c00004;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00004,
                String::from("BTST.B"),
                String::from("#$09,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_byte_bit_number_dynamic_address_register_indirect_bit_clear() {
        // arrange
        let code = [0x0b, 0x10, /* DC  */ 0x01].to_vec(); // BTST.B D5,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[5] = 0x00000001;
        cpu.register.reg_a[0] = 0x00c00002;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_EXTEND
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_OVERFLOW;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BTST.B"),
                String::from("D5,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_overflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn btst_byte_bit_number_dynamic_address_register_indirect_bit_set() {
        // arrange
        let code = [0x0b, 0x10, /* DC  */ 0x01].to_vec(); // BTST.B D5,(A0)
        let mut cpu = crate::instr_test_setup(code, None);

        cpu.register.reg_d[7] = 0x00000000;
        cpu.register.reg_a[0] = 0x00c00002;
        cpu.register.reg_sr = 0x0000;
        // act assert - debug
        let debug_result = cpu.get_next_disassembly();
        assert_eq!(
            GetDisassemblyResult::from_address_and_address_next(
                0xC00000,
                0xC00002,
                String::from("BTST.B"),
                String::from("D5,(A0)")
            ),
            debug_result
        );
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_overflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }
}
