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
        let reg_pc = mem.get_longword_unsigned(0x4).into();
        Cpu {
            reg_d: [0; 8],
            reg_a: [0; 8],
            reg_pc: reg_pc,
            memory: mem
        }
    }

    fn sign_extend_i16(address: i16) -> u32
    {
        // TODO: Tests
        // TODO: Negative tests
        let address : u32 = address as u16 as u32;
        address
    }

    fn print_registers(self: &mut Cpu<'a>)
    {
        for n in 0..8 {
            print!(" D{} {:#010x}", n, self.reg_d[n]);
        }
        println!();
        
        for n in 0..8 {
            print!(" A{} {:#010x}", n, self.reg_a[n]);
        }
        println!();
        
        print!(" PC {:#010x}", self.reg_pc);
        println!();
    }

    pub fn execute_instruction(self: &mut Cpu<'a>) {
        let instr = self.memory.get_word_unsigned(self.reg_pc);
        let is_lea = (instr & 0xf1c0) == 0x41c0;
        if is_lea {
            let operand_size = 4;
            let register = (instr >> 9) & 0x0007;
            let ea_mode = (instr >> 3) & 0x0007;
            let ea_register = instr & 0x0007;
            if ea_mode == 0b010
            {
                println!("{:#010x} LEA (A{}),A{}", self.reg_pc, ea_register, register);
                self.reg_pc = self.reg_pc + 2;
                panic!("LEA");
            }
            else if ea_mode == 0b111
            {
                if ea_register == 0b000
                {
                    let extension_word = self.memory.get_word_signed(self.reg_pc + 2);
                    let extension_word_ptr = Cpu::sign_extend_i16(extension_word);
                    let operand = self.memory.get_longword_unsigned(extension_word_ptr);
                    println!("{:#010x} LEA ({:#06x}).W,A{}", self.reg_pc, extension_word, register);
                    println!("           moving {:#010x} into A{}", operand, register);
                    let register_usize : usize = register.try_into().unwrap();
                    self.reg_a[register_usize] = operand;
                    self.reg_pc = self.reg_pc + 4;
                    self.print_registers();
                }
                else if ea_register == 0b001
                {
                    // (xxx).L
                    panic!("LEA");
                }
                else if ea_register == 0b010
                {
                    // (d16,PC)
                    panic!("LEA");
                }
                else if ea_register == 0b011
                {
                    // (d8,PC,Xn)
                    panic!("LEA");
                }
                else
                {
                    panic!("Unknown LEA addressing mode {} {}", ea_mode, ea_register);
                }
            }
            else 
            {
                panic!("Unknown LEA addressing mode {} {}", ea_mode, ea_register);
            }
        }
        else{
            panic!("Unknown instruction {:#010x}", instr);
        }

    }
}