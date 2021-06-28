
// #[derive(Copy, Clone)]
pub struct Mem {
    pub start_address: u32,
    pub end_address: u32,
    pub memory: Vec<u8>
}

impl Mem {
    // pub fn new(e0: f64) -> Mem {
    //     Mem { e: [e0, e0, e0] }
    // }

    pub fn from_file(filename: &str) -> Result<Mem, std::io::Error> {
        let vec = std::fs::read(filename)?;
        // let a = match read_result{
        //     Ok(result) => result,
        //     Err(error) => return Err(error),
        // };
        let mem = Mem {
            start_address: 0,
            end_address: 0,
            memory: vec
        };
        Ok(mem)
    }
   
}

