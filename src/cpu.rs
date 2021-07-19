use crate::instruction::*;
use crate::mem::Mem;
use crate::register::*;
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
                String::from("LEA"),
                0xf1c0,
                0x41c0,
                InstructionFormat::EffectiveAddressWithRegister(Cpu::instruction_lea),
            ),
            Instruction::new(
                String::from("Bcc"),
                0xf000,
                0x6000,
                InstructionFormat::Uncommon(Cpu::instruction_bcc),
            ),
            Instruction::new(
                String::from("MOVEQ"),
                0xf100,
                0x7000,
                InstructionFormat::Uncommon(Cpu::instruction_moveq),
            ),
            Instruction::new(
                String::from("ADD"),
                0xf000,
                0xd000,
                InstructionFormat::EffectiveAddressWithOpmodeAndRegister(Cpu::instruction_add),
            ),
            Instruction::new(
                String::from("ADDX"),
                0xf130,
                0xd100,
                InstructionFormat::Uncommon(Cpu::instruction_addx),
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

    pub fn get_address_with_i8_displacement(address: u32, displacement: i8) -> u32 {
        let displacement = Cpu::sign_extend_i8(displacement);
        let address = address.wrapping_add(displacement);

        address
    }

    pub fn get_address_with_i16_displacement(address: u32, displacement: i16) -> u32 {
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

    pub fn print_registers(self: &mut Cpu<'a>) {
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

    fn instruction_lea(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
        ea_format: String,
        register: usize,
        ea: u32,
    ) -> InstructionExecutionResult {
        reg.reg_a[register] = ea;
        let instr_comment = format!("moving {:#010x} into A{}", ea, register);
        InstructionExecutionResult {
            name: String::from("LEA"),
            operands_format: format!("{},A{}", ea_format, register),
            comment: instr_comment,
            op_size: OperationSize::Long,
            pc_result: PcResult::Increment(4),
        }
    }

    fn instruction_bcc(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> InstructionExecutionResult {
        // TODO: Condition codes
        let conditional_test = Cpu::extract_conditional_test(instr_word);
        // let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
        // let operand = instr_bytes.read_i8().unwrap();
        // let operand_ptr = Cpu::sign_extend_i8(operand);

        let displacement_8bit = (instr_word & 0x00ff) as i8;
        let operands_format = format!("[{:?}] {}", conditional_test, displacement_8bit);

        let branch_to_address = match displacement_8bit {
            0 => todo!(),
            -1 => todo!(),
            _ => Cpu::get_address_with_i8_displacement(reg.reg_pc + 2, displacement_8bit),
        };
        if displacement_8bit == 0 || displacement_8bit == -1 {
            panic!("TODO: Word and Long branches")
        }
        match Cpu::evaluate_condition(reg, &conditional_test) {
            true => todo!(),
            false => InstructionExecutionResult {
                name: format!("B{:?}", conditional_test),
                operands_format: format!("{}", displacement_8bit),
                comment: format!("not branching"),
                op_size: OperationSize::Byte,
                pc_result: PcResult::Increment(2),
            },
        }
    }

    fn instruction_moveq(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> InstructionExecutionResult {
        // TODO: Condition codes
        let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
        let operand = instr_bytes.read_i8().unwrap();
        let mut status_register_flags = 0x0000;
        match operand {
            0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
            i8::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
            _ => (),
        }
        let operand = Cpu::sign_extend_i8(operand);
        let operands_format = format!("#{},D{}", operand, register);
        let instr_comment = format!("moving {:#010x} into D{}", operand, register);
        let status_register_mask = 0xfff0;

        reg.reg_d[register] = operand;
        reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;
        InstructionExecutionResult {
            name: String::from("MOVEQ"),
            operands_format: operands_format,
            comment: instr_comment,
            op_size: OperationSize::Long,
            pc_result: PcResult::Increment(2),
        }
    }

    fn instruction_add(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
        ea_format: String,
        ea_opmode: usize,
        register: usize,
        ea: u32,
    ) -> InstructionExecutionResult {
        const BYTE_WITH_DN_AS_DEST: usize = 0b000;
        const WORD_WITH_DN_AS_DEST: usize = 0b001;
        const LONG_WITH_DN_AS_DEST: usize = 0b010;
        const BYTE_WITH_EA_AS_DEST: usize = 0b100;
        const WORD_WITH_EA_AS_DEST: usize = 0b101;
        const LONG_WITH_EA_AS_DEST: usize = 0b110;
        let status_register_mask = 0xffe0;
        // TODO: Condition codes
        match ea_opmode {
            BYTE_WITH_DN_AS_DEST => {
                let in_mem = mem.get_unsigned_byte(ea);
                let in_reg = (reg.reg_d[register] & 0x000000ff) as u8;
                let (in_reg, carry) = in_reg.overflowing_add(in_mem);
                let in_mem_signed = mem.get_signed_byte(ea);
                let in_reg_signed = (reg.reg_d[register] & 0x000000ff) as i8;
                let (in_mem_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
                reg.reg_d[register] = (reg.reg_d[register] & 0xffffff00) | (in_reg as u32);
                let instr_comment = format!("adding {:#04x} to D{}", in_mem, register);

                let mut status_register_flags = 0x0000;
                match carry {
                    true => status_register_flags |= 0b0000000000000001 | STATUS_REGISTER_MASK_EXTEND,
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

                return InstructionExecutionResult {
                    name: String::from("ADD.B"),
                    operands_format: format!("{},D{}", ea_format, register),
                    comment: instr_comment,
                    op_size: OperationSize::Byte,
                    pc_result: PcResult::Increment(2),
                };
            }
            LONG_WITH_DN_AS_DEST => {
                let in_mem = mem.get_unsigned_longword(ea);
                let in_reg = reg.reg_d[register];
                let (in_reg, carry) = in_reg.overflowing_add(in_mem);
                let in_mem_signed = mem.get_signed_longword(ea);
                let in_reg_signed = reg.reg_d[register] as i32;
                let (in_reg_signed, overflow) = in_reg_signed.overflowing_add(in_mem_signed);
                reg.reg_d[register] = in_reg;
                let instr_comment = format!("adding {:#010x} to D{}", in_mem, register);

                let mut status_register_flags = 0x0000;
                match carry {
                    true => status_register_flags |= 0b0000000000000001 | STATUS_REGISTER_MASK_EXTEND,
                    false => (),
                }
                match overflow {
                    true => status_register_flags |= STATUS_REGISTER_MASK_OVERFLOW,
                    false => (),
                }
                match in_mem_signed {
                    0 => status_register_flags |= STATUS_REGISTER_MASK_ZERO,
                    i32::MIN..=-1 => status_register_flags |= STATUS_REGISTER_MASK_NEGATIVE,
                    _ => (),
                }
                reg.reg_sr = (reg.reg_sr & status_register_mask) | status_register_flags;

                return InstructionExecutionResult {
                    name: String::from("ADD.L"),
                    operands_format: format!("{},D{}", ea_format, register),
                    comment: instr_comment,
                    op_size: OperationSize::Long,
                    pc_result: PcResult::Increment(2),
                };
            }
            _ => panic!("Unhandled ea_opmode"),
        }
    }

    fn instruction_addx(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> InstructionExecutionResult {
        println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
        return InstructionExecutionResult {
            name: String::from("ADDX"),
            operands_format: String::from("operands_format"),
            comment: String::from("comment"),
            op_size: OperationSize::Long,
            pc_result: PcResult::Increment(2),
        };
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
