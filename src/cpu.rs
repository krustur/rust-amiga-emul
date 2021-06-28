use crate::mem;
use crate::mem::Mem;

pub struct Cpu {
    memory: Mem
}

impl Cpu {
    pub fn new(mem: Mem) -> Cpu {

        Cpu {
            memory: mem
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