use crate::instruction::{EaInstruction, Instruction};
use crate::mem::Mem;
use crate::register::Register;
use byteorder::{BigEndian, ReadBytesExt};
use std::convert::TryInto;

pub struct Cpu<'a> {
    register: Register,
    pub memory: Mem<'a>,
    instructions: Vec<Instruction<'a>>,
    ea_instructions: Vec<EaInstruction<'a>>,
}

impl<'a> Cpu<'a> {
    pub fn new(mem: Mem<'a>) -> Cpu {
        let reg_ssp = mem.get_unsigned_longword(0x0);
        let reg_pc = mem.get_unsigned_longword(0x4);
        let instructions = vec![
            Instruction::new(0xf100, 0x7000, Cpu::execute_moveq),
            // Instruction::new(0xf1c0, 0x41c0, Cpu::execute_lea),
            Instruction::new(0xf130, 0xd100, Cpu::execute_addx),
            // Instruction::new(0xf000, 0xd000, Cpu::execute_add),
        ];
        let ea_instructions = vec![
            // EaInstruction::new(0xf100, 0x7000, Cpu::execute_moveq),
            EaInstruction::new(0xf1c0, 0x41c0, Cpu::execute_lea_ea),
            // EaInstruction::new(0xf130, 0xd100, Cpu::execute_addx_ea),
            EaInstruction::new(0xf000, 0xd000, Cpu::execute_add_ea),
        ];
        let mut register = Register::new();
        register.reg_a[7] = reg_ssp;
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            register: register,
            memory: mem,
            instructions: instructions,
            ea_instructions: ea_instructions,
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

    fn execute_lea_ea(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> u32 {
        // let operand_size = 4;
        let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        let ea_mode = (instr_word >> 3) & 0x0007;
        let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
        if ea_mode == 0b010 {
            println!(
                "{:#010x} LEA (A{}),A{}",
                instr_address, ea_register, register
            );
            panic!(
                "{:#010x} [not implemented] LEA addressing mode {} {}",
                instr_address, ea_mode, ea_register
            );
            // pc_increment = Some(2);
        } else if ea_mode == 0b111 {
            if ea_register == 0b000 {
                // absolute short addressing mode
                // (xxx).W
                let extension_word = mem.get_signed_word(instr_address + 2);
                let operand = mem.get_unsigned_longword_from_i16(extension_word);
                reg.reg_a[register] = operand;

                let instr_format = format!("LEA ({:#06x}).W,A{}", extension_word, register);
                let instr_comment = format!("moving {:#010x} into A{}", operand, register);
                println!(
                    "{:#010x} {: <30} ; {}",
                    instr_address, instr_format, instr_comment
                );

                return 4;
            } else if ea_register == 0b001 {
                // (xxx).L
                panic!(
                    "{:#010x} [not implemented] LEA addressing mode {} {}",
                    instr_address, ea_mode, ea_register
                );
            } else if ea_register == 0b010 {
                // program counter inderict with displacement mode
                // (d16,PC)
                let extension_word = mem.get_signed_word(instr_address + 2);
                let operand = mem
                    .get_unsigned_longword_with_i16_displacement(instr_address + 2, extension_word);
                reg.reg_a[register] = operand;

                let instr_format = format!("LEA ({:#06x},PC),A{}", extension_word, register);
                let instr_comment = format!("moving {:#010x} into A{}", operand, register);
                println!(
                    "{:#010x} {: <30} ; {}",
                    instr_address, instr_format, instr_comment
                );

                4;
            } else if ea_register == 0b011 {
                // (d8,PC,Xn)
                panic!(
                    "{:#010x} [not implemented] LEA addressing mode {} {}",
                    instr_address, ea_mode, ea_register
                );
            } else {
                panic!(
                    "{:#010x} Unknown LEA addressing mode {} {}",
                    instr_address, ea_mode, ea_register
                );
            }
        } else {
            panic!(
                "{:#010x} Unknown LEA addressing mode {} {}",
                instr_address, ea_mode, ea_register
            );
        }
        4
    }

    fn execute_moveq(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> u32 {
        // TODO: Condition codes
        let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
        let operand = instr_bytes.read_i8().unwrap();
        let operand_ptr = Cpu::sign_extend_i8(operand);
        let instr_format = format!("MOVEQ #{},D{}", operand, register);
        let instr_comment = format!("moving {:#010x} into D{}", operand_ptr, register);
        println!(
            "{:#010x} {: <30} ; {}",
            instr_address, instr_format, instr_comment
        );
        reg.reg_d[register] = operand_ptr;
        2
    }

    fn execute_add_ea(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> u32 {
        // TODO: Condition codes
        let register = Cpu::extract_register_index_from_bit_pos(instr_word, 9);
        let ea_mode = (instr_word >> 3) & 0x0007;
        let ea_register = Cpu::extract_register_index_from_bit_pos_0(instr_word);
        let op_mode = (instr_word >> 6) & 0x0007;
        println!("Execute add: {:#010x} {:#06x}", instr_address, instr_word);
        println!(
            "register {} ea_mode {:#05b} ea_register {} op_mode {:#05b} ",
            register, ea_mode, ea_register, op_mode
        );
        println!("will be: ADD.L (A0)+,D5");
        // SWITCH ON EA_MODE FIRST - WILL BE HANDLED OUTSIDE LATER
        2
    }

    fn execute_addx(
        instr_address: u32,
        instr_word: u16,
        reg: &mut Register,
        mem: &mut Mem<'a>,
    ) -> u32 {
        println!("Execute addx: {:#010x} {:#06x}", instr_address, instr_word);
        2
    }

    pub fn execute_next_instruction(self: &mut Cpu<'a>) {
        let instr_addr = self.register.reg_pc;
        let instr_word = self.memory.get_unsigned_word(instr_addr);

        let instr_pc_increment = self.execute_instruction(instr_word, instr_addr);
        match instr_pc_increment {
            None => (),
            Some(i) => {
                self.register.reg_pc = self.register.reg_pc + i;
                return;
            }
        }

        let instr_ea_pc_increment = self.execute_ea_instruction(instr_word, instr_addr);
        match instr_ea_pc_increment {
            None => (),
            Some(i) => {
                self.register.reg_pc = self.register.reg_pc + i;
                return;
            }
        }

        panic!(
            "{:#010x} Unknown instruction {:#06x}",
            instr_addr, instr_word
        );
    }

    fn execute_instruction(self: &mut Cpu<'a>, instr_word: u16, instr_addr: u32) -> Option<u32> {
        let instruction_pos = self
            .instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode)?;

        let instruction = &self.instructions[instruction_pos];
        let execute_func = &instruction.execute_func;
        let pc_increment =
            execute_func(instr_addr, instr_word, &mut self.register, &mut self.memory);

        Some(pc_increment)
    }

    fn execute_ea_instruction(self: &mut Cpu<'a>, instr_word: u16, instr_addr: u32) -> Option<u32> {
        let instruction_pos = self
            .ea_instructions
            .iter()
            .position(|x| (instr_word & x.mask) == x.opcode)?;

        let instruction = &self.ea_instructions[instruction_pos];
        let execute_func = &instruction.execute_func;
        let pc_increment =
            execute_func(instr_addr, instr_word, &mut self.register, &mut self.memory);

        Some(pc_increment)
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
