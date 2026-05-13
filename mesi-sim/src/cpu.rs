pub struct Cpu {
    pub id: usize,
    pub regs: [i64; 8],
    pub pc: usize,
    pub halted: bool,
}

impl Cpu {
    pub fn new(id: usize) -> Self {
        Cpu {
            id,
            regs: [0; 8],
            pc: 0,
            halted: false,
        }
    }
}
