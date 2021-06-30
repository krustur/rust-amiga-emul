use byteorder::{BigEndian, ReadBytesExt};
use std::convert::TryInto;
use crate::mem::Mem;
use crate::instruction::Instruction;
use crate::register::Register;

pub struct Cpu<'a> {
    register: Register,
    memory: Mem<'a>,
    instructions: Vec<Instruction<'a>>
}

impl<'a> Cpu<'a> {
    
    pub fn new(mem: Mem<'a>) -> Cpu {        
        let reg_ssp = mem.get_longword_unsigned(0x0); 
        let reg_pc = mem.get_longword_unsigned(0x4);
        let instructions = vec![
            Instruction::new(0xf100, 0x7000, Cpu::execute_moveq),            
            Instruction::new(0xf1c0, 0x41c0, Cpu::execute_lea),
            Instruction::new(0xf130, 0xd100, Cpu::execute_addx),
            Instruction::new(0xf000, 0xd000, Cpu::execute_add),
        ];
        let mut register = Register::new();
        register.reg_a[7] = reg_ssp;
        register.reg_pc = reg_pc;
        let cpu = Cpu {
            register: register,
            memory: mem,
            instructions: instructions
        };
        cpu
    }

    fn sign_extend_i8(address: i8) -> u32
    {
        // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes : [u8; 4] = if address < 0 { [0xff, 0xff, 0xff, address_bytes[0]] } else {[0x00, 0x00, 0x00, address_bytes[0]]};
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    
    fn sign_extend_i16(address: i16) -> u32
    {
        // TODO: Any better way to do this?
        let address_bytes = address.to_be_bytes();
        let fixed_bytes : [u8; 4] = if address < 0 { [0xff, 0xff, address_bytes[0], address_bytes[1]] } else {[0x00, 0x00, address_bytes[0], address_bytes[1]]};
        let mut fixed_bytes_slice = &fixed_bytes[0..4];
        let res = fixed_bytes_slice.read_u32::<BigEndian>().unwrap();
        res
    }

    pub fn print_registers(self: &mut Cpu<'a>)
    {
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

    fn execute_lea(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32 {
        let operand_size = 4;
        let register = (instr_word >> 9) & 0x0007;
        let ea_mode = (instr_word >> 3) & 0x0007;
        let ea_register = instr_word & 0x0007;
        if ea_mode == 0b010
        {
            println!("{:#010x} LEA (A{}),A{}", instr_address, ea_register, register);
            panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_address, ea_mode, ea_register);
            // pc_increment = Some(2);                
        }
        else if ea_mode == 0b111
        {
            if ea_register == 0b000
            {
                // absolute short addressing mode
                // (xxx).W
                let extension_word = mem.get_word_signed(instr_address + 2);
                let extension_word_ptr = Cpu::sign_extend_i16(extension_word);
                let operand = mem.get_longword_unsigned(extension_word_ptr);
                let instr_format = format!("LEA ({:#06x}).W,A{}", extension_word, register);
                let instr_comment = format!("moving {:#010x} into A{}", operand, register);
                println!("{:#010x} {: <30} ; {}", instr_address, instr_format, instr_comment);
                let register_usize : usize = register.try_into().unwrap();
                reg.reg_a[register_usize] = operand;
                return 4
            }
            else if ea_register == 0b001
            {
                // (xxx).L
                panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_address, ea_mode, ea_register);
            }
            else if ea_register == 0b010
            {
                // program counter inderict with displacement mode
                // (d16,PC)
                let extension_word = mem.get_word_signed(instr_address + 2);                    
                let instr_addr_i64 = i64::from(instr_address);
                let extension_word_i64 = i64::from(extension_word);
                let pc_with_displacement = (instr_addr_i64 + extension_word_i64 + 2).try_into().unwrap();
                // println!("instr_addr {:#010x}", instr_address);
                // println!("extension_word {:#010x}", extension_word);
                // println!("pc_with_displacement {:#010x}", pc_with_displacement);
                let operand = mem.get_longword_unsigned(pc_with_displacement);
                let instr_format = format!("LEA ({:#06x},PC),A{}", extension_word, register);
                let instr_comment = format!("moving {:#010x} into A{}", operand, register);
                println!("{:#010x} {: <30} ; {}", instr_address, instr_format, instr_comment);
                let register_usize : usize = register.try_into().unwrap();
                reg.reg_a[register_usize] = operand;
                4;
            }
            else if ea_register == 0b011
            {
                // (d8,PC,Xn)
                panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_address, ea_mode, ea_register);
            }
            else
            {
                panic!("{:#010x} Unknown LEA addressing mode {} {}", instr_address, ea_mode, ea_register);
            }
        }
        else 
        {
            panic!("{:#010x} Unknown LEA addressing mode {} {}", instr_address, ea_mode, ea_register);
        }
        4
    }

    fn execute_moveq(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32 {
        let register = (instr_word >> 9) & 0x0007;
        let mut instr_bytes = &instr_word.to_be_bytes()[1..2];
        let operand = instr_bytes.read_i8().unwrap();
        let operand_ptr = Cpu::sign_extend_i8(operand);
        let instr_format = format!("MOVEQ #{},D{}", operand, register);
        let instr_comment = format!("moving {:#010x} into D{}", operand_ptr, register);
        println!("{:#010x} {: <30} ; {}", instr_address, instr_format, instr_comment);
        let register_usize : usize = register.try_into().unwrap();
        reg.reg_d[register_usize] = operand_ptr;
        2
    }

    fn execute_add(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32 {
        println!("Execute add!");
        2
    }

    fn execute_addx(instr_address: u32, instr_word: u16, reg: &mut Register, mem: &mut Mem<'a>) -> u32 {
        println!("Execute addx!");
        2
    }

    pub fn execute_instruction(self: &mut Cpu<'a>) {
        let instr_addr = self.register.reg_pc;
        // let mut pc_increment : Option<u32> = None;
        let instr_word = self.memory.get_word_unsigned(instr_addr);

        let pos = self.instructions.iter().position(|x| (instr_word & x.mask) == x.opcode);
        let pos = match pos {
            None => panic!("{:#010x} Unknown instruction {:#06x}", instr_addr, instr_word),
            Some(i) => i
        };
        let instruction = &self.instructions[pos];
        let execute_func = &instruction.execute_func;
        let pc_increment = execute_func(instr_addr, instr_word, &mut self.register, &mut self.memory);
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
}