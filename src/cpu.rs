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
            println!("I'm executing a LEA!");
            self.reg_pc = self.reg_pc + 2;
        }
        else{
            panic!("I only know LEA!");
        }

    }
    // pub fn get_ray(&self, s: f64, t: f64) -> Ray {
    //     let rd = self.lens_radius*vec3::random_in_unit_sphere();
    //     let offset = self.u*rd.x() + self.v*rd.y();
    //     Ray::new(
    //         self.origin + offset,
    //         self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
    //     )
    // }
}