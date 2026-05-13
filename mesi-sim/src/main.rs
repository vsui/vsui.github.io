mod asm;
mod cache;
mod cpu;
mod memory;
mod sim;

use sim::Sim;

// Demo: 2 CPUs sharing a bus, 8-line direct-mapped caches, 64-word memory.
//
// The interleaved execution (round-robin) produces every MESI transition:
//
//  Step  CPU  Instruction          Expected cache event
//  ----  ---  -----------          --------------------
//    1    0   MOV R0, 42           no memory access
//    2    1   LOAD R0, 10          MISS I->E  (nobody has addr 10)
//    3    0   STORE 10, R0         MISS I  BusRdX -> CPU1 E->I, CPU0 M
//    4    1   PRINT R0             prints 0  (was loaded before CPU0 wrote)
//    5    0   LOAD R1, 10          HIT  M
//    6    1   LOAD R1, 10          MISS I  BusRd -> CPU0 M->S writeback, CPU1 S
//    7    0   PRINT R1             prints 42
//    8    1   PRINT R1             prints 42
//    9    0   LOAD R2, 5           MISS I->E  (different address, no conflict)
//   10    1   MOV R2, 99
//   11    0   MOV R3, 7
//   12    1   STORE 10, R2         HIT  S  BusUpgr -> CPU0 S->I, CPU1 M
//   13    0   STORE 10, R3         MISS I  BusRdX -> CPU1 M->I writeback, CPU0 M
//   14    1   HALT
//   15    0   PRINT R0             prints 42
//   16    0   HALT

fn main() {
    let mut sim = Sim::new(2, 8, 64);

    sim.load(
        0,
        "
        MOV  R0, 42
        STORE 10, R0    # MISS I -> BusRdX -> M
        LOAD  R1, 10    # HIT  M
        PRINT R1        # 42
        LOAD  R2, 5     # MISS I -> E  (addr 5 not in any cache)
        MOV   R3, 7
        STORE 10, R3    # MISS I -> BusRdX -> M  (CPU1 had M, writeback)
        PRINT R0        # 42
        HALT
    ",
    );

    sim.load(
        1,
        "
        LOAD  R0, 10    # MISS I -> E  (before CPU0 writes)
        PRINT R0        # 0  (memory default, CPU0 hasn't written yet)
        LOAD  R1, 10    # MISS I -> S  (BusRd; CPU0 M->S writeback)
        PRINT R1        # 42
        MOV   R2, 99
        STORE 10, R2    # HIT  S -> BusUpgr -> M
        HALT
    ",
    );

    sim.run();
}
