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
                InstructionFormat::EffectiveAddressWithRegister(instruction::lea::step),
            ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                InstructionFormat::Uncommon(instruction::bcc::step),
            ),
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                InstructionFormat::Uncommon(instruction::moveq::step),
            ),
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                InstructionFormat::EffectiveAddressWithOpmodeAndRegister(instruction::add::step),
            ),
            Instruction::new(
                String::from("ADDX"),
                0xf130,
                0xd100,
                InstructionFormat::Uncommon(instruction::addx::step),
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

    fn extract_conditional_test(word: u16) -> ConditionalTest {
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

    pub fn execute_next_instruction(self: &mut Cpu) {
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

        let exec_result: InstructionExecutionResult = match instruction.instruction_format {
            InstructionFormat::Uncommon(exec_func) => {
                let exec_result =
                    exec_func(instr_addr, instr_word, &mut self.register, &mut self.memory);
                let instr_format = format!("{} {}", exec_result.name, exec_result.operands_format);
                exec_result
            }
            InstructionFormat::EffectiveAddressWithRegister(exec_func) => {
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
                                let ea = Cpu::sign_extend_i16(extension_word);
                                let ea_format = format!("({:#06x}).W", extension_word);
                                let exec_result = exec_func(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea_format,
                                    register,
                                    ea,
                                );
                                exec_result
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
                                let exec_result = exec_func(
                                    instr_addr,
                                    instr_word,
                                    &mut self.register,
                                    &mut self.memory,
                                    ea_format,
                                    register,
                                    ea,
                                );
                                exec_result
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
            InstructionFormat::EffectiveAddressWithOpmodeAndRegister(exec_func) => {
                let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
                let ea_opmode = Cpu::extract_op_mode_from_bit_pos_6(instr_word);
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
                    }
                    EffectiveAddressingMode::ARegIndirectWithPostIncrement => {
                        let ea = self.register.reg_a[ea_register];
                        let ea_format = format!("(A{})+", ea_register);
                        println!("ea_address: {:#010x}", ea);

                        let operand = self.memory.get_unsigned_longword(ea);
                        let exec_result = exec_func(
                            instr_addr,
                            instr_word,
                            &mut self.register,
                            &mut self.memory,
                            ea_format,
                            ea_opmode,
                            register,
                            ea,
                        );
                        self.register.reg_a[ea_register] += exec_result.op_size.size_in_bytes();
                        exec_result
                    }
                    EffectiveAddressingMode::ARegIndirectWithPreDecrement
                    | EffectiveAddressingMode::ARegIndirectWithDisplacement
                    | EffectiveAddressingMode::ARegIndirectWithIndex
                    | EffectiveAddressingMode::PcIndirectAndLotsMore => {
                        panic!(
                            "{:#010x} {:#06x} UNKNOWN_EA {:?} {}",
                            instr_addr, instr_word, ea_mode, ea_register
                        );
                    }
                }
                // panic!("EffectiveAddressWithOpmodeAndRegister not quite done");
            }
        };
        let instr_format = format!("{} {}", exec_result.name, exec_result.operands_format);
        println!(
            "{:#010x} {: <30} ; {}",
            instr_addr, instr_format, exec_result.comment
        );

        self.register.reg_pc = match exec_result.pc_result {
            PcResult::Set(lepc) => lepc,
            PcResult::Increment(pc_increment) => self.register.reg_pc + pc_increment,
        }
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
