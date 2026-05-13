use crate::asm::{parse, Instr, MemRef};
use crate::cache::{Cache, Snoop, State};
use crate::cpu::Cpu;
use crate::memory::Memory;

pub struct Sim {
    cpus: Vec<Cpu>,
    caches: Vec<Cache>,
    memory: Memory,
    programs: Vec<Vec<Instr>>,
    step: usize,
}

impl Sim {
    pub fn new(num_cpus: usize, cache_lines: usize, mem_size: usize) -> Self {
        Sim {
            cpus: (0..num_cpus).map(Cpu::new).collect(),
            caches: (0..num_cpus).map(|i| Cache::new(i, cache_lines)).collect(),
            memory: Memory::new(mem_size),
            programs: vec![Vec::new(); num_cpus],
            step: 0,
        }
    }

    pub fn load(&mut self, cpu: usize, src: &str) {
        self.programs[cpu] = parse(src);
    }

    // -------------------------------------------------------------------------
    // Bus transactions
    // -------------------------------------------------------------------------

    /// BusRd: another cache wants to read addr.
    /// Returns (data, others_had_it). Snooping caches downgrade M/E -> S.
    fn bus_read(&mut self, req: usize, addr: usize) -> (i64, bool) {
        let n = self.caches.len();
        let mut others = false;
        let mut supply: Option<i64> = None;

        for i in 0..n {
            if i == req {
                continue;
            }
            match self.caches[i].snoop_bus_rd(addr) {
                Snoop::WasModified(data) => {
                    println!(
                        "    [Cache {}] BusRd snoop addr={}: M->S  writeback={}",
                        i, addr, data
                    );
                    self.memory.write(addr, data);
                    supply = Some(data);
                    others = true;
                }
                Snoop::WasExclusive => {
                    println!("    [Cache {}] BusRd snoop addr={}: E->S", i, addr);
                    others = true;
                }
                Snoop::WasShared => {
                    others = true;
                }
                Snoop::NotPresent => {}
            }
        }

        let data = supply.unwrap_or_else(|| self.memory.read(addr));
        (data, others)
    }

    /// BusRdX: requester wants exclusive write access.
    /// All other caches invalidate, dirty owners write back.
    fn bus_rdx(&mut self, req: usize, addr: usize) {
        let n = self.caches.len();
        for i in 0..n {
            if i == req {
                continue;
            }
            match self.caches[i].snoop_bus_rdx(addr) {
                Snoop::WasModified(data) => {
                    println!(
                        "    [Cache {}] BusRdX snoop addr={}: M->I  writeback={}",
                        i, addr, data
                    );
                    self.memory.write(addr, data);
                }
                Snoop::WasExclusive => {
                    println!("    [Cache {}] BusRdX snoop addr={}: E->I", i, addr);
                }
                Snoop::WasShared => {
                    println!("    [Cache {}] BusRdX snoop addr={}: S->I", i, addr);
                }
                Snoop::NotPresent => {}
            }
        }
    }

    /// BusUpgr: requester already holds S and upgrades to M.
    /// Other S holders simply invalidate.
    fn bus_upgr(&mut self, req: usize, addr: usize) {
        let n = self.caches.len();
        for i in 0..n {
            if i == req {
                continue;
            }
            if self.caches[i].state(addr) == State::Shared {
                self.caches[i].snoop_bus_upgr(addr);
                println!("    [Cache {}] BusUpgr snoop addr={}: S->I", i, addr);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Processor-side read/write (state machine + bus calls)
    // -------------------------------------------------------------------------

    fn proc_read(&mut self, cpu: usize, addr: usize) -> i64 {
        let st = self.caches[cpu].state(addr);

        if st != State::Invalid {
            let data = self.caches[cpu].read_data(addr);
            println!(
                "  [CPU {}] READ  addr={:3} | HIT  ({}) -> {}",
                cpu, addr, st, data
            );
            return data;
        }

        println!(
            "  [CPU {}] READ  addr={:3} | MISS (I) -> BusRd",
            cpu, addr
        );

        if let Some((e_addr, e_data)) = self.caches[cpu].dirty_eviction(addr) {
            println!(
                "    [Cache {}] Evict dirty addr={} data={} -> memory",
                cpu, e_addr, e_data
            );
            self.memory.write(e_addr, e_data);
        }

        let (data, shared) = self.bus_read(cpu, addr);
        let new_st = if shared { State::Shared } else { State::Exclusive };
        self.caches[cpu].install(addr, data, new_st);
        println!(
            "  [CPU {}] READ  addr={:3} | I->{}  data={}",
            cpu, addr, new_st, data
        );
        data
    }

    fn proc_write(&mut self, cpu: usize, addr: usize, val: i64) {
        let st = self.caches[cpu].state(addr);
        match st {
            State::Modified => {
                println!(
                    "  [CPU {}] WRITE addr={:3} val={} | HIT (M) -> M",
                    cpu, addr, val
                );
                self.caches[cpu].update_data(addr, val);
            }
            State::Exclusive => {
                println!(
                    "  [CPU {}] WRITE addr={:3} val={} | HIT (E) -> M  (silent upgrade)",
                    cpu, addr, val
                );
                self.caches[cpu].update_data(addr, val);
                self.caches[cpu].set_state(addr, State::Modified);
            }
            State::Shared => {
                println!(
                    "  [CPU {}] WRITE addr={:3} val={} | HIT (S) -> BusUpgr -> M",
                    cpu, addr, val
                );
                self.bus_upgr(cpu, addr);
                self.caches[cpu].update_data(addr, val);
                self.caches[cpu].set_state(addr, State::Modified);
            }
            State::Invalid => {
                println!(
                    "  [CPU {}] WRITE addr={:3} val={} | MISS (I) -> BusRdX -> M",
                    cpu, addr, val
                );
                if let Some((e_addr, e_data)) = self.caches[cpu].dirty_eviction(addr) {
                    println!(
                        "    [Cache {}] Evict dirty addr={} data={} -> memory",
                        cpu, e_addr, e_data
                    );
                    self.memory.write(e_addr, e_data);
                }
                self.bus_rdx(cpu, addr);
                self.caches[cpu].install(addr, val, State::Modified);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Instruction execution
    // -------------------------------------------------------------------------

    fn resolve_addr(&self, cpu: usize, mref: &MemRef) -> usize {
        match mref {
            MemRef::Direct(a) => *a,
            MemRef::Indirect(r) => self.cpus[cpu].regs[*r] as usize,
        }
    }

    fn exec_one(&mut self, cpu: usize) -> bool {
        if self.cpus[cpu].halted {
            return false;
        }
        let pc = self.cpus[cpu].pc;
        if pc >= self.programs[cpu].len() {
            self.cpus[cpu].halted = true;
            return false;
        }

        let instr = self.programs[cpu][pc].clone();
        self.cpus[cpu].pc += 1;
        self.step += 1;

        println!("\n[Step {:3}] CPU {} | {:?}", self.step, cpu, instr);

        match instr {
            Instr::Mov(rd, v) => {
                self.cpus[cpu].regs[rd] = v;
                println!("  [CPU {}] R{} = {}", cpu, rd, v);
            }
            Instr::Load(rd, mref) => {
                let addr = self.resolve_addr(cpu, &mref);
                let val = self.proc_read(cpu, addr);
                self.cpus[cpu].regs[rd] = val;
            }
            Instr::Store(mref, rs) => {
                let addr = self.resolve_addr(cpu, &mref);
                let val = self.cpus[cpu].regs[rs];
                self.proc_write(cpu, addr, val);
            }
            Instr::Add(rd, r1, r2) => {
                let (a, b) = (self.cpus[cpu].regs[r1], self.cpus[cpu].regs[r2]);
                let val = a.wrapping_add(b);
                self.cpus[cpu].regs[rd] = val;
                println!(
                    "  [CPU {}] R{} = R{}({}) + R{}({}) = {}",
                    cpu, rd, r1, a, r2, b, val
                );
            }
            Instr::Sub(rd, r1, r2) => {
                let (a, b) = (self.cpus[cpu].regs[r1], self.cpus[cpu].regs[r2]);
                let val = a.wrapping_sub(b);
                self.cpus[cpu].regs[rd] = val;
                println!(
                    "  [CPU {}] R{} = R{}({}) - R{}({}) = {}",
                    cpu, rd, r1, a, r2, b, val
                );
            }
            Instr::Mul(rd, r1, r2) => {
                let (a, b) = (self.cpus[cpu].regs[r1], self.cpus[cpu].regs[r2]);
                let val = a.wrapping_mul(b);
                self.cpus[cpu].regs[rd] = val;
                println!(
                    "  [CPU {}] R{} = R{}({}) * R{}({}) = {}",
                    cpu, rd, r1, a, r2, b, val
                );
            }
            Instr::Print(rs) => {
                println!("  >>> CPU {} R{} = {}", cpu, rs, self.cpus[cpu].regs[rs]);
            }
            Instr::Halt => {
                self.cpus[cpu].halted = true;
                println!("  [CPU {}] HALTED", cpu);
                return false;
            }
        }

        !self.cpus[cpu].halted
    }

    // -------------------------------------------------------------------------
    // Simulation loop
    // -------------------------------------------------------------------------

    pub fn run(&mut self) {
        println!("=== MESI Cache Coherence Simulator ===");
        println!("CPUs: {}  Scheduling: round-robin\n", self.cpus.len());

        let n = self.cpus.len();
        let mut cursor = 0usize;

        loop {
            if self.cpus.iter().all(|c| c.halted) {
                break;
            }
            let cpu = cursor % n;
            cursor += 1;
            if !self.cpus[cpu].halted {
                self.exec_one(cpu);
            }
        }

        self.print_final();
    }

    fn print_final(&self) {
        println!("\n=== Final State ===");

        println!("Registers:");
        for cpu in &self.cpus {
            let regs: Vec<_> = cpu
                .regs
                .iter()
                .enumerate()
                .filter(|(_, &v)| v != 0)
                .map(|(i, &v)| format!("R{}={}", i, v))
                .collect();
            if regs.is_empty() {
                println!("  CPU {}: (all zero)", cpu.id);
            } else {
                println!("  CPU {}: {}", cpu.id, regs.join("  "));
            }
        }

        println!("Caches:");
        for cache in &self.caches {
            cache.dump();
        }
    }
}
