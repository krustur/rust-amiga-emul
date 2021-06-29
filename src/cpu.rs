// use crate::mem;
use crate::mem::Mem;
use std::convert::TryFrom;

pub struct Cpu<'a> {
    reg_pc: u32,
    memory: Mem<'a>
}

impl<'a> Cpu<'a> {
    pub fn new(mem: Mem<'a>) -> Cpu {
        let reg_pc = mem.get_longword_unsigned(0x4).into();
        Cpu {
            reg_pc: reg_pc,
            memory: mem
        }
    }

    pub fn execute_instruction(self: &mut Cpu<'a>) {
        let reg_pc_us = usize::try_from(self.reg_pc).unwrap();
        let instr = self.memory.get_word_unsigned(reg_pc_us);
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
                    let extension_word = self.memory.get_word_unsigned(reg_pc_us + 2);
                    println!("{:#010x} LEA ({}).W,A{}", self.reg_pc, ea_register, register);
                    self.reg_pc = self.reg_pc + 4;    
                }
                else if ea_register == 0b001
                {
                    // (xxx).L
                }
                else if ea_register == 0b010
                {
                    // (d16,PC)
                }
                else if ea_register == 0b011
                {
                    // (d8,PC,Xn)
                }

                panic!("LEA");
            }

            panic!("Unknown LEA addressing mode {} {}", ea_mode, ea_register);
        }
        else{
            panic!("Unknown instruction {:#010x}", instr);
        }

    }
}