use crate::cpu::instruction::*;
use crate::mem::Mem;
use crate::register::*;
use byteorder::{BigEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::convert::TryInto;

mod instruction;

pub struct Cpu {
    pub register: Register,
    pub memory: Mem,
    instructions: Vec<Instruction>,
}

impl Cpu {
    pub fn new(mem: Mem) -> Cpu {
        let reg_ssp = mem.get_unsigned_longword(0x0);
        let reg_pc = mem.get_unsigned_longword(0x4);
        let instructions = vec![
            Instruction::new(
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                InstructionFormat::EffectiveAddress {
                    common_step: instruction::lea::common_step,
                    common_get_debug: instruction::lea::common_get_debug,
                    areg_direct_step: instruction::lea::areg_direct_step,
                    areg_direct_get_debug: instruction::lea::areg_direct_get_debug,
                },
            ),
            // 0x5---
            Instruction::new(
                String::from("ADDQ"),
                0xf100,
                0x5000,
                InstructionFormat::Uncommon {
                    step: instruction::todo::step,
                    get_debug: instruction::todo::get_debug,
                },
            ),
            Instruction::new(
                String::from("SUBQ"),
                0xf100,
                0x5100,
                InstructionFormat::EffectiveAddress {
                    common_step: instruction::subq::common_step,
                    common_get_debug: instruction::subq::common_get_debug,
                    areg_direct_step: instruction::subq::areg_direct_step,
                    areg_direct_get_debug: instruction::subq::areg_direct_get_debug,
                },
            ),
            Instruction::new(
                String::from("DBcc"),
                0xf0f8,
                0x50c8,
                InstructionFormat::Uncommon {
                    step: instruction::dbcc::step,
                    get_debug: instruction::dbcc::get_debug,
                },
            ),
            Instruction::new(
                String::from("TRAPcc"),
                0xf0f8,
                0x50f8,
                InstructionFormat::Uncommon {
                    step: instruction::todo::step,
                    get_debug: instruction::todo::get_debug,
                },
            ),
            Instruction::new(
                String::from("Scc"),
                0xf0c0,
                0x50c0,
                InstructionFormat::Uncommon {
                    step: instruction::todo::step,
                    get_debug: instruction::todo::get_debug,
                },
            ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                InstructionFormat::Uncommon {
                    step: instruction::bcc::step,
                    get_debug: instruction::todo::get_debug,
                },
            ),
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                InstructionFormat::Uncommon {
                    step: instruction::moveq::step,
                    get_debug: instruction::moveq::get_debug,
                },
            ),
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                InstructionFormat::EffectiveAddress {
                    common_step: instruction::add::common_step,
                    common_get_debug: instruction::add::common_get_debug,
                    areg_direct_step: instruction::add::areg_direct_step_func,
                    areg_direct_get_debug: instruction::add::areg_direct_get_debug,
                },
            ),
            Instruction::new(
                String::from("ADDX"),
                0xf130,
                0xd100,
                InstructionFormat::Uncommon {
                    step: instruction::addx::step,
                    get_debug: instruction::addx::get_debug,
                },
            ),
        ];
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

    fn sign_extend_i16(address: i16) -> u32 {
        // // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes: [u8; 4] = if address < 0 {
            [0xff, 0xff, address_bytes[0], address_bytes[1]]
        } else {
            [0x00, 0x00, address_bytes[0], address_bytes[1]]
        };
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    fn get_address_with_i8_displacement(address: u32, displacement: i8) -> u32 {
        let displacement = Cpu::sign_extend_i8(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    fn get_address_with_i16_displacement(address: u32, displacement: i16) -> u32 {
        let displacement = Cpu::sign_extend_i16(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    fn extract_effective_addressing_mode(word: u16) -> EffectiveAddressingMode {
        let ea_mode = (word >> 3) & 0x0007;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract EffectiveAddressingMode"),
        };
        ea_mode
    }

    fn extract_conditional_test_pos_8(word: u16) -> ConditionalTest {
        let ea_mode = (word >> 8) & 0x000f;
        let ea_mode = match FromPrimitive::from_u16(ea_mode) {
            Some(r) => r,
            None => panic!("Unable to extract ConditionalTest"),
        };
        ea_mode
    }

    fn extract_op_mode_from_bit_pos_6(word: u16) -> usize {
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

    pub fn extract_size_from_bit_pos_6(word: u16) -> Option<OperationSize> {
        let size = word & 0x0007;
        let size = (word >> 6) & 0x0003;
        match size {
            0b00 => Some(OperationSize::Byte),
            0b01 => Some(OperationSize::Word),
            0b10 => Some(OperationSize::Long),
            _ => None,
        }
    }

    pub fn print_registers(self: &mut Cpu) {
        for n in 0..8 {
            print!(" D{} {:#010X}", n, self.register.reg_d[n]);
        }
        println!();
        for n in 0..8 {
            print!(" A{} {:#010X}", n, self.register.reg_a[n]);
        }
        println!();
        print!(" SR {:#06X} ", self.register.reg_sr);
        if (self.register.reg_sr & STATUS_REGISTER_MASK_EXTEND) == STATUS_REGISTER_MASK_EXTEND {
            print!("X");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_NEGATIVE) == STATUS_REGISTER_MASK_NEGATIVE {
            print!("N");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_ZERO) == STATUS_REGISTER_MASK_ZERO {
            print!("Z");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_OVERFLOW) == STATUS_REGISTER_MASK_OVERFLOW {
            print!("V");
        } else {
            print!("-");
        }
        if (self.register.reg_sr & STATUS_REGISTER_MASK_CARRY) == STATUS_REGISTER_MASK_CARRY {
            print!("C");
        } else {
            print!("-");
        }
        println!();
        print!(" PC {:#010X}", self.register.reg_pc);
        println!();
    }

    fn evaluate_condition(reg: &mut Register, conditional_test: &ConditionalTest) -> bool {
        match conditional_test {
            ConditionalTest::T => true,
            ConditionalTest::F => false,
            ConditionalTest::CC => reg.reg_sr & STATUS_REGISTER_MASK_CARRY == 0x0000,
            _ => panic!("ConditionalTest not implemented"),
        }
    }

    pub fn execute_next_instruction(self: &mut Cpu) -> InstructionDebugResult {
        let instr_addr = self.register.reg_pc;
        let instr_word = self.memory.get_unsigned_word(instr_addr);

        let instruction = self.get_instruction(instr_addr, instr_word);

        // // TODO: Expose the debugger somewhere else, not when executing instructions
        // let debug_string = match instruction.instruction_format {
        //     InstructionFormat::Uncommon{step, get_debug} => {
        //         println!("{}", instruction.name);
        //         get_debug(instr_addr, instr_word, &mut self.register, &mut self.memory)
        //     }
        // };

        let (exec_result, debug_result) = match instruction.instruction_format {
            InstructionFormat::Uncommon { step, get_debug } => {
                // println!("{}", instruction.name);
                let debug_result =
                    get_debug(instr_addr, instr_word, &mut self.register, &mut self.memory);
                // println!("{}", debug_result);
                let exec_result =
                    step(instr_addr, instr_word, &mut self.register, &mut self.memory);
                (exec_result, debug_result)
            }
            InstructionFormat::EffectiveAddress {
                common_step,
                common_get_debug,
                areg_direct_step,
                areg_direct_get_debug,
            } => {
                let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
                let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);

                match ea_mode {
                    EffectiveAddressingMode::DRegDirect => {
                        panic!(
                            "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                            instr_addr, instr_word, ea_mode, ea_register
                        );
                    }
                    EffectiveAddressingMode::ARegDirect => {
                        let debug_result = areg_direct_get_debug(
                            instr_addr,
                            instr_word,
                            &mut self.register,
                            &mut self.memory,
                            ea_register,
                        );
                        let exec_result = areg_direct_step(
                            instr_addr,
                            instr_word,
                            &mut self.register,
                            &mut self.memory,
                            ea_register,
                        );
                        (exec_result, debug_result)
                    }
                    EffectiveAddressingMode::ARegIndirect
                    | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                        let ea = self.register.reg_a[ea_register];
                        let ea_format = format!("(A{})+", ea_register);

                        let debug_result = common_get_debug(
                            instr_addr,
                            instr_word,
                            &mut self.register,
                            &mut self.memory,
                            ea_format,
                            ea,
                        );
                        let exec_result = common_step(
                            instr_addr,
                            instr_word,
                            &mut self.register,
                            &mut self.memory,
                            ea,
                        );

                        if ea_mode == EffectiveAddressingMode::ARegIndirectWithPostIncrement {
                            if let InstructionExecutionResult::Done { pc_result } = exec_result {
                                todo!("post incrementen");
                                // self.register.reg_a[ea_register] += op_size.size_in_bytes();
                            }
                        }
                        (exec_result, debug_result)
                    }
                    EffectiveAddressingMode::ARegIndirectWithPreDecrement
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
                                let ea = Cpu::sign_extend_i16(extension_word);
                                let ea_format = format!("({:#06x}).W", extension_word);
                                let debug_result = common_get_debug(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea_format,
                                    ea,
                                );
                                let exec_result = common_step(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea,
                                );
                                (exec_result, debug_result)
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
                                let ea = Cpu::get_address_with_i16_displacement(
                                    self.register.reg_pc + 2,
                                    extension_word,
                                );
                                //  let operand =
                                //  self.memory.get_unsigned_longword_with_i16_displacement(
                                //         instr_addr + 2,
                                //         extension_word,
                                //     );
                                let ea_format = format!("({:#06x},PC)", extension_word);
                                let debug_result = common_get_debug(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea_format,
                                    ea,
                                );
                                let exec_result = common_step(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea,
                                );
                                (exec_result, debug_result)
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
            } // InstructionFormat::EffectiveAddressWithOpmodeAndRegister(exec_func) => {
              //     let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
              //     let ea_opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
              //     let ea_mode = Cpu::extract_effective_addressing_mode(instr_word);
              //     let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
              //     println!(
              //         "register {} ea_mode {:?} ea_register {} ea_opmode {:?} ",
              //         register, ea_mode, ea_register, ea_opmode
              //     );

              //     match ea_mode {
              //         EffectiveAddressingMode::DRegDirect | EffectiveAddressingMode::ARegDirect => {
              //             panic!(
              //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
              //                 instr_addr, instr_word, ea_mode, ea_register
              //             );
              //         }
              //         EffectiveAddressingMode::ARegIndirect
              //         | EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
              //             let ea = self.register.reg_a[ea_register];
              //             let ea_format = format!("(A{})+", ea_register);

              //             // let operand = self.memory.get_unsigned_longword(ea);
              //             let exec_result = exec_func(
              //                 instr_addr,
              //                 instr_word,
              //                 &mut self.register,
              //                 &mut self.memory,
              //                 ea_format,
              //                 ea_opmode,
              //                 register,
              //                 ea,
              //             );
              //             if ea_mode == EffectiveAddressingMode::ARegIndirectWithPostIncrement {
              //                 self.register.reg_a[ea_register] += exec_result.op_size.size_in_bytes();
              //             }
              //             exec_result
              //         }
              //         EffectiveAddressingMode::ARegIndirectWithPreDecrement
              //         | EffectiveAddressingMode::ARegIndirectWithDisplacement
              //         | EffectiveAddressingMode::ARegIndirectWithIndex
              //         | EffectiveAddressingMode::PcIndirectAndLotsMore => {
              //             panic!(
              //                 "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
              //                 instr_addr, instr_word, ea_mode, ea_register
              //             );
              //         }
              //     }
              //     // panic!("EffectiveAddressWithOpmodeAndRegister not quite done");
              // }
        };
        if let InstructionDebugResult::Done {
            name,
            operands_format,
            next_instr_address,
        } = &debug_result
        {
            let instr_format = format!("{} {}", name, operands_format);
            print!("{:#010X} ", instr_addr);
            for i in (instr_addr .. instr_addr + 8).step_by(2) {
                if i < *next_instr_address {
                    let op_mem = self.memory.get_unsigned_word(i);
                    print!("{:04X} ", op_mem);
                } else {
                    print!("     ");
                }
            }
            println!("{: <30}", instr_format);
        }
        if let InstructionExecutionResult::Done {
            // comment,
            // op_size,
            pc_result,
        } = &exec_result
        {
            self.register.reg_pc = match pc_result {
                PcResult::Set(lepc) => *lepc,
                PcResult::Increment(pc_increment) => self.register.reg_pc + pc_increment,
            }
        }

        debug_result
    }

    fn get_instruction(self: &mut Cpu, instr_addr: u32, instr_word: u16) -> &Instruction {
        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode);
        let instruction = match instruction_pos {
            None => panic!(
                "{:#010x} Unidentified instruction {:#06x}",
                instr_addr, instr_word
            ),
            Some(instruction_pos) => &self.instructions[instruction_pos],
        };
        instruction
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

    #[test]
    fn sign_extend_i16_positive() {
        let res = Cpu::sign_extend_i16(345);
        assert_eq!(345, res);
    }

    #[test]
    fn sign_extend_i16_negative() {
        let res = Cpu::sign_extend_i16(-345);
        assert_eq!(0xFFFFFEA7, res);
    }

    #[test]
    fn sign_extend_i16_negative2() {
        let res = Cpu::sign_extend_i16(-1);
        assert_eq!(0xFFFFFFFF, res);
    }

    #[test]
    fn get_address_with_i8_displacement() {
        let res = Cpu::get_address_with_i8_displacement(0x00100000, i8::MAX);
        assert_eq!(0x0010007f, res);
    }

    #[test]
    fn get_address_with_i8_displacement_negative() {
        let res = Cpu::get_address_with_i8_displacement(0x00100000, i8::MIN);
        assert_eq!(0x000fff80, res);
    }

    #[test]
    fn get_address_with_i8_displacement_overflow() {
        let res = Cpu::get_address_with_i8_displacement(0xffffffff, i8::MAX);
        assert_eq!(0x0000007e, res);
    }

    #[test]
    fn get_address_with_i8_displacement_overflow_negative() {
        let res = Cpu::get_address_with_i8_displacement(0x00000000, i8::MIN);
        assert_eq!(0xffffff80, res);
    }

    #[test]
    fn get_address_with_i16_displacement() {
        let res = Cpu::get_address_with_i16_displacement(0x00100000, i16::MAX);
        assert_eq!(0x00107fff, res);
    }

    #[test]
    fn get_address_with_i16_displacement_negative() {
        let res = Cpu::get_address_with_i16_displacement(0x00100000, i16::MIN);
        assert_eq!(0x000f8000, res);
    }

    #[test]
    fn get_address_with_i16_displacement_overflow() {
        let res = Cpu::get_address_with_i16_displacement(0xffffffff, i16::MAX);
        assert_eq!(0x00007ffe, res);
    }

    #[test]
    fn get_address_with_i16_displacement_overflow_neg() {
        let res = Cpu::get_address_with_i16_displacement(0x00000000, i16::MIN);
        assert_eq!(0xffff8000, res);
    }

    #[test]
    fn evaluate_condition_cc_cleared() {
        let mut register = Register::new();
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(true, res);
    }

    #[test]
    fn evaluate_condition_cc_set() {
        let mut register = Register::new();
        register.reg_sr = 0x0001;
        let res = Cpu::evaluate_condition(&mut register, &ConditionalTest::CC);
        assert_eq!(false, res);
    }
}
