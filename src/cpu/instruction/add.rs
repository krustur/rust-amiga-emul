use std::panic;

use crate::cpu::instruction::{EffectiveAddressingMode, PcResult};
use crate::cpu::Cpu;
use crate::mem::Mem;
use crate::register::{
    Register, STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND,
    STATUS_REGISTER_MASK_NEGATIVE, STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
};

use super::{DisassemblyResult, InstructionExecutionResult};

pub fn step<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &mut Register,
    mem: &mut Mem,
    // ea: u32,
) -> InstructionExecutionResult {
    const BYTE_WITH_DN_AS_DEST: usize = 0b000;
    const WORD_WITH_DN_AS_DEST: usize = 0b001;
    const LONG_WITH_DN_AS_DEST: usize = 0b010;
    const BYTE_WITH_EA_AS_DEST: usize = 0b100;
    const WORD_WITH_EA_AS_DEST: usize = 0b101;
    const LONG_WITH_EA_AS_DEST: usize = 0b110;
    let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);

    let opsize = match opmode {
        BYTE_WITH_DN_AS_DEST => 1,
        WORD_WITH_DN_AS_DEST => 2,
        LONG_WITH_DN_AS_DEST => 4,
        BYTE_WITH_EA_AS_DEST => 1,
        WORD_WITH_EA_AS_DEST => 2,
        LONG_WITH_EA_AS_DEST => 4,
        _ => panic!("What")
    };

    // let ea = match ea_mode {
    //     EffectiveAddressingMode::DRegDirect => {
    //         panic!(
    //             "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
    //             instr_address, instr_word, ea_mode, ea_register
    //         );
    //     }
    //     EffectiveAddressingMode::ARegDirect => {
    //         panic!(
    //             "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
    //             instr_address, instr_word, ea_mode, ea_register
    //         );
    //     }
    //     EffectiveAddressingMode::ARegIndirect
    //     | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
    //         reg.reg_a[ea_register]
    //     }
    //     EffectiveAddressingMode::ARegIndirectWithPreDecrement => {
    //         reg.reg_a[ea_register] += opsize;
    //         reg.reg_a[ea_register]
    //     }
    //     EffectiveAddressingMode::ARegIndirectWithDisplacement
    //     | EffectiveAddressingMode::ARegIndirectWithIndex
    //     | EffectiveAddressingMode::PcIndirectAndLotsMore => {
    //         panic!(
    //             "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
    //             instr_address, instr_word, ea_mode, ea_register
    //         );
    //     }
    // };
    //  = reg.reg_a[ea_register];

    let status_register_mask = 0xffe0;
    // TODO: Condition codes
    match opmode {
        BYTE_WITH_DN_AS_DEST => {
            let ea_value = Cpu::get_ea_value_unsigned_byte(ea_mode, ea_register, instr_address, reg, mem);
            // let in_mem = mem.get_unsigned_byte(ea);
            let in_reg = (reg.reg_d[register] & 0x000000ff) as u8;
            let (in_reg, carry) = in_reg.overflowing_add(ea_value);
            let in_mem_signed = Cpu::get_ea_value_signed_byte(ea_mode, ea_register, instr_address, reg, mem);
            let in_reg_signed = (reg.reg_d[register] & 0x000000ff) as i8;
            let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            reg.reg_d[register] = (reg.reg_d[register] & 0xffffff00) | (in_reg as u32);
            // let instr_comment = format!("adding {:#04x} to D{}", in_mem, register);

            let mut status_register_flags = 0x0000;
            match carry {
                true => {
                    status_register_flags |=
                        STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND
                }
                false => (),
            }
            match overflow {
                true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                false => (),
            }
            match in_mem_signed {
                0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return InstructionExecutionResult::Done {
                // name: "ADD.B",
                // operands_format: &format!("{},D{}", ea_format, register),
                // comment: &instr_comment,
                // op_size: OperationSize::Byte,
                pc_result: PcResult::Increment(2),
            };
        }
        WORD_WITH_DN_AS_DEST => {
            let in_mem = Cpu::get_ea_value_unsigned_word(ea_mode, ea_register, instr_address, reg, mem);
            let in_reg = (reg.reg_d[register] & 0x0000ffff) as u16;
            let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            let in_mem_signed = Cpu::get_ea_value_signed_word(ea_mode, ea_register, instr_address, reg, mem);
            let in_reg_signed = (reg.reg_d[register] & 0x0000ffff) as i16;
            let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            reg.reg_d[register] = (reg.reg_d[register] & 0xffff0000) | (in_reg as u32);
            let instr_comment = format!("adding {:#06x} to D{}", in_mem, register);

            let mut status_register_flags = 0x0000;
            match carry {
                true => {
                    status_register_flags |=
                        STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND
                }
                false => (),
            }
            match overflow {
                true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                false => (),
            }
            match in_mem_signed {
                0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                i16::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return InstructionExecutionResult::Done {
                // name: "ADD.W",
                // operands_format: &format!("{},D{}", ea_format, register),
                // comment: &instr_comment,
                // op_size: OperationSize::Word,
                pc_result: PcResult::Increment(2),
            };
        }
        LONG_WITH_DN_AS_DEST => {
            let ea_value = Cpu::get_ea_value_unsigned_long(ea_mode, ea_register, instr_address, reg, mem);
            let in_reg = reg.reg_d[register];
            let (in_reg, carry) = in_reg.overflowing_add(ea_value.value);
            let ea_value_signed = Cpu::get_ea_value_signed_long(ea_mode, ea_register, instr_address, reg, mem);
            let in_reg_signed = reg.reg_d[register] as i32;
            let (in_reg_signed, overflow) = in_reg_signed.overflowing_add(ea_value_signed);
            reg.reg_d[register] = in_reg;
            // let instr_comment = format!("adding {:#010x} to D{}", in_mem, register);

            let mut status_register_flags = 0x0000;
            match carry {
                true => {
                    status_register_flags |=
                        STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND
                }
                false => (),
            }
            match overflow {
                true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                false => (),
            }
            match ea_value_signed {
                0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                i32::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                _ => (),
            }
            reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return InstructionExecutionResult::Done {
                // name: "ADD.L",
                // operands_format: &format!("{},D{}", ea_format, register),
                // comment: &instr_comment,
                // op_size: OperationSize::Long,
                pc_result: PcResult::Increment(2),
            };
        }
        _ => panic!("Unhandled ea_opmode"),
    }
}

pub fn get_debug<'a>(
    instr_address: u32,
    instr_word: u16,
    reg: &Register,
    mem: &Mem,
    // ea_format: String,
    // ea: u32,
) -> DisassemblyResult {
    const BYTE_WITH_DN_AS_DEST: usize = 0b000;
    const WORD_WITH_DN_AS_DEST: usize = 0b001;
    const LONG_WITH_DN_AS_DEST: usize = 0b010;
    const BYTE_WITH_EA_AS_DEST: usize = 0b100;
    const WORD_WITH_EA_AS_DEST: usize = 0b101;
    const LONG_WITH_EA_AS_DEST: usize = 0b110;
    // let status_register_mask = 0xffe0;
    let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
    let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
    let opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
    // let ea_opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
    let ea_format = Cpu::get_ea_format(ea_mode, ea_register, instr_address, reg, mem);
    // TODO: Condition codes
    match opmode {
        BYTE_WITH_DN_AS_DEST => {
            // let in_mem = mem.get_unsigned_byte(ea);
            // let in_reg = (reg.reg_d[register] & 0x000000ff) as u8;
            // let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            // let in_mem_signed = mem.get_signed_byte(ea);
            // let in_reg_signed = (reg.reg_d[register] & 0x000000ff) as i8;
            // let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            // reg.reg_d[register] = (reg.reg_d[register] & 0xffffff00) | (in_reg as u32);
            // let instr_comment = format!("adding {:#04x} to D{}", in_mem, register);

            // let mut status_register_flags = 0x0000;
            // match carry {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            //     false => (),
            // }
            // match overflow {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
            //     false => (),
            // }
            // match in_mem_signed {
            //     0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
            //     i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
            //     _ => (),
            // }
            // reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return DisassemblyResult::Done {
                name: String::from("ADD.B"),
                operands_format: format!("{},D{}", ea_format, register),
                instr_address,
                next_instr_address: instr_address + 2,
            };
        }
        WORD_WITH_DN_AS_DEST => {
            // let in_mem = mem.get_unsigned_word(ea);
            // let in_reg = (reg.reg_d[register] & 0x0000ffff) as u16;
            // let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            // let in_mem_signed = mem.get_signed_word(ea);
            // let in_reg_signed = (reg.reg_d[register] & 0x0000ffff) as i16;
            // let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            // reg.reg_d[register] = (reg.reg_d[register] & 0xffff0000) | (in_reg as u32);
            // let instr_comment = format!("adding {:#06x} to D{}", in_mem, register);

            // let mut status_register_flags = 0x0000;
            // match carry {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            //     false => (),
            // }
            // match overflow {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
            //     false => (),
            // }
            // match in_mem_signed {
            //     0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
            //     i16::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
            //     _ => (),
            // }
            // reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return DisassemblyResult::Done {
                name: String::from("ADD.W"),
                operands_format: format!("{},D{}", ea_format, register),
                instr_address,
                next_instr_address: instr_address + 2,
            };
        }
        LONG_WITH_DN_AS_DEST => {
            // let in_mem = mem.get_unsigned_longword(ea);
            // let in_reg = reg.reg_d[register];
            // let (in_reg, carry) = in_reg.overflowing_add(in_mem);
            // let in_mem_signed = mem.get_signed_longword(ea);
            // let in_reg_signed = reg.reg_d[register] as i32;
            // let (in_reg_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
            // reg.reg_d[register] = in_reg;
            // let instr_comment = format!("adding {:#010x} to D{}", in_mem, register);

            // let mut status_register_flags = 0x0000;
            // match carry {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_CARRY | STATUS_REGISTER_MASK_EXTEND,
            //     false => (),
            // }
            // match overflow {
            //     true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
            //     false => (),
            // }
            // match in_mem_signed {
            //     0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
            //     i32::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
            //     _ => (),
            // }
            // reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

            return DisassemblyResult::Done {
                name: String::from("ADD.L"),
                operands_format: format!("{},D{}", ea_format, register),
                instr_address,
                next_instr_address: instr_address + 2,
            };
        }
        _ => panic!("Unhandled ea_opmode"),
    }
}

#[cfg(test)]
mod tests {
    use crate::register::{
        STATUS_REGISTER_MASK_CARRY, STATUS_REGISTER_MASK_EXTEND, STATUS_REGISTER_MASK_NEGATIVE,
        STATUS_REGISTER_MASK_OVERFLOW, STATUS_REGISTER_MASK_ZERO,
    };

    #[test]
    fn step_byte_to_d0() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B d1,d0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x01, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_byte_to_d0_overflow() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B d1,d0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[0] = 0x0000007f;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x80, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_byte_to_d0_carry() {
        // arrange
        let code = [0xd0, 0x10, 0x01].to_vec(); // ADD.B d1,d0
                                                // DC.B 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[0] = 0x000000ff;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x00, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W d1,d0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0001, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0_overflow() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W d1,d0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[0] = 0x00007fff;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x8000, cpu.register.reg_d[0]);
        assert_eq!(false, cpu.register.is_sr_carry_set());
        assert_eq!(true, cpu.register.is_sr_coverflow_set());
        assert_eq!(false, cpu.register.is_sr_zero_set());
        assert_eq!(true, cpu.register.is_sr_negative_set());
        assert_eq!(false, cpu.register.is_sr_extend_set());
    }

    #[test]
    fn step_word_to_d0_carry() {
        // arrange
        let code = [0xd0, 0x50, 0x00, 0x01].to_vec(); // ADD.W d1,d0
                                                      // DC.W 0x01
        let mut cpu = crate::instr_test_setup(code, None);
        cpu.register.reg_a[0] = 0x00080002;
        cpu.register.reg_d[0] = 0x0000ffff;
        cpu.register.reg_d[1] = 0x00000001;
        cpu.register.reg_sr = STATUS_REGISTER_MASK_CARRY
            | STATUS_REGISTER_MASK_OVERFLOW
            | STATUS_REGISTER_MASK_ZERO
            | STATUS_REGISTER_MASK_NEGATIVE
            | STATUS_REGISTER_MASK_EXTEND;
        // act
        cpu.execute_next_instruction();
        // assert
        assert_eq!(0x0000, cpu.register.reg_d[0]);
        assert_eq!(true, cpu.register.is_sr_carry_set());
        assert_eq!(false, cpu.register.is_sr_coverflow_set());
        assert_eq!(true, cpu.register.is_sr_zero_set());
        assert_eq!(false, cpu.register.is_sr_negative_set());
        assert_eq!(true, cpu.register.is_sr_extend_set());
    }
}
