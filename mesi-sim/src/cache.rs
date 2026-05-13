/// MESI cache coherence protocol - direct-mapped cache

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Modified,
    Exclusive,
    Shared,
    Invalid,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                State::Modified => "M",
                State::Exclusive => "E",
                State::Shared => "S",
                State::Invalid => "I",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct Line {
    state: State,
    tag: usize,
    data: i64,
}

impl Line {
    const INVALID: Line = Line {
        state: State::Invalid,
        tag: 0,
        data: 0,
    };
}

/// Result returned to the bus after snooping a cache line.
pub enum Snoop {
    NotPresent,
    WasShared,
    WasExclusive,
    WasModified(i64), // carries dirty data for write-back
}

pub struct Cache {
    pub id: usize,
    lines: Vec<Line>,
}

impl Cache {
    pub fn new(id: usize, num_lines: usize) -> Self {
        Cache {
            id,
            lines: vec![Line::INVALID; num_lines],
        }
    }

    fn idx(&self, addr: usize) -> usize {
        addr % self.lines.len()
    }

    fn is_hit(&self, addr: usize) -> bool {
        let i = self.idx(addr);
        self.lines[i].state != State::Invalid && self.lines[i].tag == addr
    }

    pub fn state(&self, addr: usize) -> State {
        if self.is_hit(addr) {
            self.lines[self.idx(addr)].state
        } else {
            State::Invalid
        }
    }

    pub fn read_data(&self, addr: usize) -> i64 {
        self.lines[self.idx(addr)].data
    }

    pub fn install(&mut self, addr: usize, data: i64, state: State) {
        let i = self.idx(addr);
        self.lines[i] = Line { state, tag: addr, data };
    }

    pub fn set_state(&mut self, addr: usize, state: State) {
        if self.is_hit(addr) {
            let i = self.idx(addr);
            self.lines[i].state = state;
        }
    }

    pub fn update_data(&mut self, addr: usize, data: i64) {
        let i = self.idx(addr);
        self.lines[i].data = data;
    }

    /// Returns (evict_addr, dirty_data) if the line at addr's index holds a
    /// different dirty address that must be written back before installing addr.
    pub fn dirty_eviction(&self, addr: usize) -> Option<(usize, i64)> {
        let i = self.idx(addr);
        let l = &self.lines[i];
        if l.state == State::Modified && l.tag != addr {
            Some((l.tag, l.data))
        } else {
            None
        }
    }

    // --- Snoop handlers (called by bus on behalf of another CPU) ---

    pub fn snoop_bus_rd(&mut self, addr: usize) -> Snoop {
        if !self.is_hit(addr) {
            return Snoop::NotPresent;
        }
        let i = self.idx(addr);
        match self.lines[i].state {
            State::Modified => {
                let data = self.lines[i].data;
                self.lines[i].state = State::Shared;
                Snoop::WasModified(data)
            }
            State::Exclusive => {
                self.lines[i].state = State::Shared;
                Snoop::WasExclusive
            }
            State::Shared => Snoop::WasShared,
            State::Invalid => Snoop::NotPresent,
        }
    }

    pub fn snoop_bus_rdx(&mut self, addr: usize) -> Snoop {
        if !self.is_hit(addr) {
            return Snoop::NotPresent;
        }
        let i = self.idx(addr);
        let snoop = match self.lines[i].state {
            State::Modified => Snoop::WasModified(self.lines[i].data),
            State::Exclusive => Snoop::WasExclusive,
            State::Shared => Snoop::WasShared,
            State::Invalid => Snoop::NotPresent,
        };
        self.lines[i].state = State::Invalid;
        snoop
    }

    pub fn snoop_bus_upgr(&mut self, addr: usize) {
        if self.is_hit(addr) {
            let i = self.idx(addr);
            if self.lines[i].state == State::Shared {
                self.lines[i].state = State::Invalid;
            }
        }
    }

    pub fn dump(&self) {
        let valid: Vec<_> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.state != State::Invalid)
            .collect();
        if valid.is_empty() {
            println!("  CPU {} cache: (empty)", self.id);
        } else {
            for (i, l) in valid {
                println!(
                    "  CPU {} cache[{:2}]: addr={:3}  state={}  data={}",
                    self.id, i, l.tag, l.state, l.data
                );
            }
        }
    }
}
