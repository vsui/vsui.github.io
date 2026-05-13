pub struct Memory {
    data: Vec<i64>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory { data: vec![0; size] }
    }

    pub fn read(&self, addr: usize) -> i64 {
        self.data.get(addr).copied().unwrap_or(0)
    }

    pub fn write(&mut self, addr: usize, val: i64) {
        if addr < self.data.len() {
            self.data[addr] = val;
        }
    }
}
