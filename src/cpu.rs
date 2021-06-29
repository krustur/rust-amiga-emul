use byteorder::{BigEndian, ReadBytesExt};
use std::convert::TryInto;
use crate::mem::Mem;

pub struct Cpu<'a> {
    reg_d: [u32; 8],
    reg_a: [u32; 8],
    reg_pc: u32,
    memory: Mem<'a>
}

impl<'a> Cpu<'a> {
    pub fn new(mem: Mem<'a>) -> Cpu {        
        let reg_ssp = mem.get_longword_unsigned(0x0); 
        let reg_pc = mem.get_longword_unsigned(0x4);
        let mut cpu = Cpu {
            reg_d: [0; 8],
            reg_a: [0; 8],
            reg_pc: reg_pc,
            memory: mem
        };
        cpu.reg_a[7] = reg_ssp;
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
            print!(" D{} {:#010X}", n, self.reg_d[n]);
        }
        println!();
        
        for n in 0..8 {
            print!(" A{} {:#010X}", n, self.reg_a[n]);
        }
        println!();
        
        print!(" PC {:#010X}", self.reg_pc);
        println!();
    }

    pub fn execute_instruction(self: &mut Cpu<'a>) {
        let instr_addr = self.reg_pc;
        let mut pc_increment : Option<u32> = None;
        let instr = self.memory.get_word_unsigned(instr_addr);
        let is_lea = (instr & 0xf1c0) == 0x41c0;
        let is_moveq = (instr & 0xf100) == 0x7000;
        let is_addx = (instr & 0xf130) == 0xd100;
        let is_adda = (instr & 0xf000) == 0xd000;
        let is_add = (instr & 0xf000) == 0xd000;
        if is_lea {
            let operand_size = 4;
            let register = (instr >> 9) & 0x0007;
            let ea_mode = (instr >> 3) & 0x0007;
            let ea_register = instr & 0x0007;
            if ea_mode == 0b010
            {
                println!("{:#010x} LEA (A{}),A{}", instr_addr, ea_register, register);
                panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_addr, ea_mode, ea_register);
                // pc_increment = Some(2);                
            }
            else if ea_mode == 0b111
            {
                if ea_register == 0b000
                {
                    // absolute short addressing mode
                    // (xxx).W
                    let extension_word = self.memory.get_word_signed(instr_addr + 2);
                    let extension_word_ptr = Cpu::sign_extend_i16(extension_word);
                    let operand = self.memory.get_longword_unsigned(extension_word_ptr);
                    println!("{:#010x} LEA ({:#06x}).W,A{}", instr_addr, extension_word, register);
                    println!("           moving {:#010x} into A{}", operand, register);
                    let register_usize : usize = register.try_into().unwrap();
                    self.reg_a[register_usize] = operand;
                    pc_increment = Some(4);
                }
                else if ea_register == 0b001
                {
                    // (xxx).L
                    panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_addr, ea_mode, ea_register);
                }
                else if ea_register == 0b010
                {
                    // program counter inderict with displacement mode
                    // (d16,PC)
                    let extension_word = self.memory.get_word_signed(instr_addr + 2);                    
                    let instr_addr_i64 = i64::from(instr_addr);
                    let extension_word_i64 = i64::from(extension_word);
                    let pc_with_displacement = (instr_addr_i64 + extension_word_i64 + 2).try_into().unwrap();
                    // let extension_word_ptr = Cpu::sign_extend_i16(extension_word);
                    println!("instr_addr {:#010x}", instr_addr);
                    println!("extension_word {:#010x}", extension_word);
                    println!("pc_with_displacement {:#010x}", pc_with_displacement);
                    // let pc_with_displacement = instr_addr + extension_word_ptr;
                    let operand = self.memory.get_longword_unsigned(pc_with_displacement);
                    println!("{:#010x} LEA ({:#06x},PC),A{}", instr_addr, extension_word, register);
                    println!("           moving {:#010x} into A{}", operand, register);
                    let register_usize : usize = register.try_into().unwrap();
                    self.reg_a[register_usize] = operand;
                    pc_increment = Some(4);
                }
                else if ea_register == 0b011
                {
                    // (d8,PC,Xn)
                    panic!("{:#010x} [not implemented] LEA addressing mode {} {}", instr_addr, ea_mode, ea_register);
                }
                else
                {
                    panic!("{:#010x} Unknown LEA addressing mode {} {}", instr_addr, ea_mode, ea_register);
                }
            }
            else 
            {
                panic!("{:#010x} Unknown LEA addressing mode {} {}", instr_addr, ea_mode, ea_register);
            }
        }
        else if is_moveq
        {
            let register = (instr >> 9) & 0x0007;
            // let operand = instr & 0x00ff;
            let mut instr_bytes = &instr.to_be_bytes()[1..2];
            // let instr_bytes = &instr_bytes[1..1];
            let operand = instr_bytes.read_i8().unwrap();
            let operand_ptr = Cpu::sign_extend_i8(operand);
            println!("{:#010x} MOVEQ #{},D{}", instr_addr, operand, register);
            println!("           moving {:#010x} into D{}", operand, register);
            let register_usize : usize = register.try_into().unwrap();
            self.reg_d[register_usize] = operand_ptr;
            // pc_increment = 
        }
        else{
            println!("is_addx {} is_adda {} is_add {}", is_addx, is_adda, is_add);

            panic!("{:#010x} Unknown instruction {:#06x}", instr_addr, instr);
        }

        let pc_increment = pc_increment.unwrap_or(2);
        self.reg_pc = self.reg_pc + pc_increment;
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