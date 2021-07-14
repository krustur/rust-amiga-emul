use crate::instruction::{EffectiveAddressingMode, Instruction, InstructionFormat, OpMode};
use crate::mem::Mem;
use crate::register::Register;
use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::convert::TryInto;

pub struct Cpu<'a> {
    register: Register,
    pub memory: Mem<'a>,
    instructions: Vec<Instruction<'a>>,
}

impl<'a> Cpu<'a> {
    pub fn new(mem: Mem<'a>) -> Cpu {
        let reg_ssp = mem.get_unsigned_longword(0x0);
        let reg_pc = mem.get_unsigned_longword(0x4);
        let instructions = vec![
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                InstructionFormat::Uncommon(Cpu::execute_moveq),
            ),
            Instruction::new(
                String::from("ADDX"),
                0xf130,
                0xd100,
                InstructionFormat::Uncommon(Cpu::execute_addx),
            ),
            Instruction::new(
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                InstructionFormat::EffectiveAddressWithRegister {
                    exec_func_absolute_short: Some(Cpu::execute_lea_absolute_short),
                    exec_func_pc_indirect_with_displacement_mode: Some(
                        Cpu::execute_lea_program_counter_indirect_with_displacement_mode_func,
                    ),
                },
            ),
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                InstructionFormat::EffectiveAddressWithOpmodeAndRegister {
                    exec_func_areg_indirect_with_post_inc: Some(Cpu::execute_add_ea),
                },
            ),
        ];
        // let ea_instructions = vec![];
        let mut register = Register::new();
        register.reg_a[7] = reg_ssp;
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            register: register,
            memory: mem,
            instructions: instructions,
        };
        cpu
    }

    fn sign_extend_i8(address: i8) -> u32 {
        // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes: [u8; 4] = if address < 0 {
            [0xff, 0xff, 0xff, address_bytes[0]]
        } else {
            [0x00, 0x00, 0x00, address_bytes[0]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    // fn sign_extend_i16(address: i16) -> u32 {
    //     // TODO: Any better way to do this?
    //     let address_bytes = address.to_be_bytes();
    //     let fixed_bytes: [u8; 4] = if address < 0 {
    //         [0xff, 0xff, address_bytes[0], address_bytes[1]]
    //     } else {
    //         [0x00, 0x00, address_bytes[0], address_bytes[1]]
    //     };
    //     let mut fixed_bytes_slice = &fixed_bytes[0..4];
    //     let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
    //     res
    // }

    fn extract_effective_addressing_mode(word: u16) -> EffectiveAddressingMode {
        let ea_mode = (word >> 3) & 0x0007;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract EffectiveAddressingMode"),
        };
        ea_mode
    }

    fn extract_op_mode(word: u16) -> OpMode {
        let op_mode = (word >> 6) & 0x0007;
        let op_mode = match FromPrimitive::from_u16(op_mode) {
            Some(r) => r,
            None => panic!("Unable to extract OpMode"),
        };
        op_mode
    }

    fn extract_register_index_from_bit_pos(word: u16, bit_pos: u8) -> usize {
        let register = (word >> bit_pos) & 0x0007;
        let register = register.try_into().unwrap();
        register
    }

    fn extract_register_index_from_bit_pos_0(word: u16) -> usize {
        let register = word & 0x0007;
        let register = register.try_into().unwrap();
        register
    }

    pub fn print_registers(self: &mut Cpu<'a>) {
        for n in 0..8 {
            print!(" D{} {:#010X}", n, self.register.reg_d[n]);
        }
        println!();
        for n in 0..8 {
            print!(" A{} {:#010X}", n, self.register.reg_a[n]);
        }
        println!();
        print!(" PC {:#010X}", self.register.reg_pc);
        println!();
    }

    fn execute_lea_absolute_short(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
        register: usize,
        operand: u32,
    ) -> String {
        reg.reg_a[register] = operand;
        let instr_comment = format!("moving {:#010x} into A{}", operand, register);
        return instr_comment;
    }

    fn execute_lea_program_counter_indirect_with_displacement_mode_func(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
        register: usize,
        operand: u32,
    ) -> String {
        reg.reg_a[register] = operand;
        let instr_comment = format!("moving {:#010x} into A{}", operand, register);
        instr_comment
    }

    fn execute_moveq(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> (String, String, u32) {
        // TODO: Condition codes
        let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
        let operand = instr_bytes.read_i8().unwrap();
        let operand_ptr = Cpu::sign_extend_i8(operand);
        let operands_format = format!("#{},D{}", operand, register);
        let instr_comment = format!("moving {:#010x} into D{}", operand_ptr, register);
        reg.reg_d[register] = operand_ptr;
        (operands_format, instr_comment, 2)
    }

    fn execute_add_ea(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
        register: usize,
        operand: u32,
    ) -> String {
        // TODO: Condition codes
        println!("will be: ADD.L (A0)+,D5");
        // SWITCH ON EA_MODE FIRST - WILL BE HANDLED OUTSIDE LATER
        String::from("comment")
    }

    fn execute_addx(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> (String, String, u32) {
        println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
        (String::from("ops"), String::from("comment"), 2)
    }

    pub fn execute_next_instruction(self: &mut Cpu<'a>) {
        let instr_addr = self.register.reg_pc;
        let instr_word = self.memory.get_unsigned_word(instr_addr);

        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode);

        let instruction = match instruction_pos {
            None => panic!(
                "{:#010x} Unknown instruction {:#06x}",
                instr_addr, instr_word
            ),
            Some(instruction_pos) => &self.instructions[instruction_pos],
        };

        let pc_increment = match instruction.instruction_format {
            InstructionFormat::Uncommon(exec_func) => {
                let (operands_format, instr_comment, pc_increment) =
                    exec_func(instr_addr, instr_word, &mut self.register, &mut self.memory);
                let instr_format = format!("{} {}", instruction.name, operands_format);
                // let instr_comment = format!("moving {:#010x} into D{}", operand_ptr, register);
                println!(
                    "{:#010x} {: <30} ; {}",
                    instr_addr, instr_format, instr_comment
                );
                pc_increment
            }
            InstructionFormat::EffectiveAddressWithRegister {
                exec_func_absolute_short,
                exec_func_pc_indirect_with_displacement_mode,
            } => {
                let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
                let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
                let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

                match ea_mode {
                    EffectiveAddressingMode::DRegDirect
                    | EffectiveAddressingMode::ARegDirect
                    | EffectiveAddressingMode::ARegIndirect
                    | EffectiveAddressingMode::ARegIndirectWithPostIncrement
                    | EffectiveAddressingMode::ARegIndirectWithPreDecrement
                    | EffectiveAddressingMode::ARegIndirectWithDisplacement
                    | EffectiveAddressingMode::ARegIndirectWithIndex => {
                        panic!(
                            "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                            instr_addr, instr_word, ea_mode, ea_register
                        );
                        // pc_increment = Some(2);
                    }
                    EffectiveAddressingMode::PcIndirectAndLotsMore => {
                        match ea_register {
                            0b000 => {
                                // absolute short addressing mode
                                // (xxx).W
                                let extension_word = self.memory.get_signed_word(instr_addr + 2);
                                let operand =
                                    self.memory.get_unsigned_longword_from_i16(extension_word);
                                let exec_func_absolute_short = exec_func_absolute_short.unwrap_or_else(|| panic!("Effective Addressing 'Absolute short addressing mode' not implemented for {}", instruction.name));
                                let instr_comment = exec_func_absolute_short(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    register,
                                    operand,
                                );
                                let instr_format = format!(
                                    "{} ({:#06x}).W,A{}",
                                    instruction.name, extension_word, register
                                );
                                println!(
                                    "{:#010x} {: <30} ; {}",
                                    instr_addr, instr_format, instr_comment
                                );
                                4
                            }
                            0b001 => {
                                // (xxx).L
                                panic!(
                                    "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                            0b010 => {
                                // PC indirect with displacement mode
                                // (d16,PC)
                                let extension_word = self.memory.get_signed_word(instr_addr + 2);
                                let operand =
                                    self.memory.get_unsigned_longword_with_i16_displacement(
                                        instr_addr + 2,
                                        extension_word,
                                    );
                                let exec_func_pc_indirect_with_displacement_mode = exec_func_pc_indirect_with_displacement_mode.unwrap_or_else(|| panic!("Effective Addressing 'PC indirect with displacement mode' not implemented for {}", instruction.name));
                                let instr_format = format!(
                                    "{} ({:#06x},PC),A{}",
                                    instruction.name, extension_word, register
                                );
                                let instr_comment = exec_func_pc_indirect_with_displacement_mode(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    register,
                                    operand,
                                );
                                println!(
                                    "{:#010x} {: <30} ; {}",
                                    instr_addr, instr_format, instr_comment
                                );
                                4
                            }
                            0b011 => {
                                // (d8,PC,Xn)
                                panic!(
                                    "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                            _ => {
                                panic!(
                                    "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                        }
                    }
                }
            }
            InstructionFormat::EffectiveAddressWithOpmodeAndRegister {
                exec_func_areg_indirect_with_post_inc,
            } => {
                let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
                let ea_opmode = Cpu::extract_op_mode(instr_word);
                let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
                let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
                println!(
                    "register {} ea_mode {:?} ea_register {} ea_opmode {:?} ",
                    register, ea_mode, ea_register, ea_opmode
                );

                match ea_mode {
                    EffectiveAddressingMode::DRegDirect
                    | EffectiveAddressingMode::ARegDirect
                    | EffectiveAddressingMode::ARegIndirect => {
                        panic!(
                            "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                            instr_addr, instr_word, ea_mode, ea_register
                        );
                    },
                    EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                        match ea_opmode {
                            OpMode::ByteWithDnAsDest
                            | OpMode::WordWithDnAsDest => {
                                panic!(
                                    "{:#010x} {:#06x} UNKNOWN_EA_OPMODE {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                            OpMode::LongWithDnAsDest => {
                                let ea_address = self.register.reg_a[ea_register];
                                self.register.reg_a[ea_register] += 4;
                                println!("ea_address: {:#010x}", ea_address);
                                panic!(
                                    "{:#010x} {:#06x} THIS IS IT {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                            OpMode::ByteWithEaAsDest
                            | OpMode::WordWithEaAsDest
                            | OpMode::LongWithEaAsDest => {
                                panic!(
                                    "{:#010x} {:#06x} UNKNOWN_EA_OPMODE {:?} {}",
                                    instr_addr, instr_word, ea_mode, ea_register
                                );
                            }
                        }
                        
                    },
                    EffectiveAddressingMode::ARegIndirectWithPreDecrement
                    | EffectiveAddressingMode::ARegIndirectWithDisplacement
                    | EffectiveAddressingMode::ARegIndirectWithIndex
                    | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                        panic!(
                            "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                            instr_addr, instr_word, ea_mode, ea_register
                        );
                        // pc_increment = Some(2);
                    }
                }
                // panic!("EffectiveAddressWithOpmodeAndRegister not quite done");
            }
        };
        self.register.reg_pc = self.register.reg_pc + pc_increment;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_i8_positive() {
        let res = Cpu::sign_extend_i8(45);
        assert_eq!(45, res);
    }

    #[test]
    fn sign_extend_i8_negative() {
        let res = Cpu::sign_extend_i8(-45);
        assert_eq!(0xFFFFFFD3, res);
    }

    #[test]
    fn sign_extend_i8_negative2() {
        let res = Cpu::sign_extend_i8(-1);
        assert_eq!(0xFFFFFFFF, res);
    }

    // #[test]
    // fn sign_extend_i16_positive() {
    //     let res = Cpu::sign_extend_i16(345);
    //     assert_eq!(345, res);
    // }

    // #[test]
    // fn sign_extend_i16_negative() {
    //     let res = Cpu::sign_extend_i16(-345);
    //     assert_eq!(0xFFFFFEA7, res);
    // }

    // #[test]
    // fn sign_extend_i16_negative2() {
    //     let res = Cpu::sign_extend_i16(-1);
    //     assert_eq!(0xFFFFFFFF, res);
    // }
}
