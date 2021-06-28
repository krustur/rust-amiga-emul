
// #[derive(Copy, Clone)]
pub struct Mem {
    pub start_address: usize,
    pub end_address: usize,
    pub memory: Vec<u8>
}

impl Mem {
    // pub fn new(e0: f64) -> Mem {
    //     Mem { e: [e0, e0, e0] }
    // }

    pub fn from_file(start_address: usize, filename: &str) -> Result<Mem, std::io::Error> {
        let vec = std::fs::read(filename)?;

        let mem = Mem {
            start_address: start_address,
            end_address: 0,
            memory: vec
        };
        Ok(mem)
    }

    fn remap_address_to_index(self: &Mem, address: usize) -> usize {
        // TODO: Implement
        return address;
    }

    pub fn get_longword_unsigned(self: &Mem, address: usize) -> usize {
        let index = self.remap_address_to_index(address);
        let b0 : usize = self.memory[index].into();
        let b1 : usize = self.memory[index + 1].into();
        let b2 : usize = self.memory[index + 2].into();
        let b3 : usize = self.memory[index + 3].into();
        let longword_unsigned = (b0 << 24) + (b1 << 16) + (b2 << 8) + b3;
        longword_unsigned
    }


   
}

