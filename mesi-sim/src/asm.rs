/// Simple assembly language
///
/// Registers: R0..R7
/// Instructions:
///   MOV  Rd, <imm>          Rd = immediate value
///   LOAD Rd, <addr>         Rd = mem[addr]
///   LOAD Rd, [Rs]           Rd = mem[Rs]
///   STORE <addr>, Rs        mem[addr] = Rs
///   STORE [Rd], Rs          mem[Rd] = Rs
///   ADD  Rd, Ra, Rb         Rd = Ra + Rb
///   SUB  Rd, Ra, Rb         Rd = Ra - Rb
///   MUL  Rd, Ra, Rb         Rd = Ra * Rb
///   PRINT Rs                print Rs
///   HALT                    stop this CPU

#[derive(Debug, Clone)]
pub enum MemRef {
    Direct(usize),   // literal address
    Indirect(usize), // address held in register
}

#[derive(Debug, Clone)]
pub enum Instr {
    Mov(usize, i64),
    Load(usize, MemRef),
    Store(MemRef, usize),
    Add(usize, usize, usize),
    Sub(usize, usize, usize),
    Mul(usize, usize, usize),
    Print(usize),
    Halt,
}

pub fn parse(src: &str) -> Vec<Instr> {
    src.lines()
        .filter_map(|raw| {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                return None;
            }
            match parse_instr(line) {
                Some(i) => Some(i),
                None => {
                    eprintln!("  [ASM] parse error: '{}'", line);
                    None
                }
            }
        })
        .collect()
}

fn reg(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.len() >= 2 && (s.starts_with('R') || s.starts_with('r')) {
        s[1..].parse().ok()
    } else {
        None
    }
}

fn imm(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn mem_ref(s: &str) -> Option<MemRef> {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        reg(s[1..s.len() - 1].trim()).map(MemRef::Indirect)
    } else {
        imm(s).map(|v| MemRef::Direct(v as usize))
    }
}

fn parse_instr(line: &str) -> Option<Instr> {
    let (op, rest) = match line.split_once(|c: char| c.is_whitespace()) {
        Some((a, b)) => (a.to_uppercase(), b.trim()),
        None => (line.to_uppercase(), ""),
    };

    let args: Vec<&str> = if rest.is_empty() {
        vec![]
    } else {
        rest.split(',').map(str::trim).collect()
    };

    match op.as_str() {
        "MOV" if args.len() == 2 => Some(Instr::Mov(reg(args[0])?, imm(args[1])?)),
        "LOAD" if args.len() == 2 => Some(Instr::Load(reg(args[0])?, mem_ref(args[1])?)),
        "STORE" if args.len() == 2 => Some(Instr::Store(mem_ref(args[0])?, reg(args[1])?)),
        "ADD" if args.len() == 3 => {
            Some(Instr::Add(reg(args[0])?, reg(args[1])?, reg(args[2])?))
        }
        "SUB" if args.len() == 3 => {
            Some(Instr::Sub(reg(args[0])?, reg(args[1])?, reg(args[2])?))
        }
        "MUL" if args.len() == 3 => {
            Some(Instr::Mul(reg(args[0])?, reg(args[1])?, reg(args[2])?))
        }
        "PRINT" if args.len() == 1 => Some(Instr::Print(reg(args[0])?)),
        "HALT" => Some(Instr::Halt),
        _ => None,
    }
}
