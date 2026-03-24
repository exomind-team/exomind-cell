# D0 VM Architecture

## Overview

The D0 VM is a "soup model" artificial life system. All organisms float in a shared food pool with no spatial coordinates. The core design principle: **organisms must actively maintain themselves or die**.

## Instruction Set (12 instructions)

| Opcode | Mnemonic | Description | Energy Cost |
|--------|----------|-------------|-------------|
| 0x00 | NOP | No operation | 1 |
| 0x01 | INC Rx | Rx += 1 (wrapping) | 1 |
| 0x02 | DEC Rx | Rx -= 1 (wrapping) | 1 |
| 0x03 | CMP Rx Ry | R0 = (Rx > Ry) ? 1 : 0 | 1 |
| 0x04 | JMP offset | IP += offset (relative, wrapping) | 1 |
| 0x05 | JNZ offset | If R0 != 0, IP += offset | 1 |
| 0x06 | LOAD Rx idx | Rx = data[R1 mod data.len()] | 1 |
| 0x07 | STORE Rx idx | data[R1 mod data.len()] = Rx | 1 |
| 0x08 | SENSE_SELF Rx | Rx = current energy | 1 |
| 0x09 | EAT | Take food from pool (capped at E_MAX) | 1 |
| 0x0A | REFRESH | Reset freshness to max (+ refresh_cost) | 1 + refresh_cost |
| 0x0B | DIVIDE | Clone code with mutation, split energy | 1 + divide_cost |

**Key design decisions:**
- EAT + REFRESH are *necessary* for survival (operational closure)
- DIVIDE is *optional* — reproduction is not the core of life
- CMP writes to R0, JNZ reads R0 — simple condition mechanism
- All jumps are relative and wrap around code length

## Organism Structure

```rust
struct Organism {
    code: Vec<Instruction>,   // The program IS the body
    data: Vec<u8>,            // Private data memory (8 bytes)
    registers: [i32; 8],      // R0-R7 general purpose
    ip: usize,                // Instruction pointer (relative)
    energy: i32,              // Capped at E_MAX
    freshness: u8,            // Decays each tick (if enabled)
    alive: bool,
    age: u64,
    generation: u32,          // How many DIVIDEs since seed
}
```

**Death conditions (irreversible):**
- `energy <= 0` — starvation
- `freshness == 0` — body disintegration (only if freshness_decay enabled)

## World Model

```
World {
    organisms: Vec<Organism>  // All organisms in the soup
    food_pool: i32            // Shared food resource
    tick: u64                 // Global clock
}
```

**Each tick:**
1. Food pool += food_per_tick
2. For each alive organism:
   a. If freshness_decay: freshness -= 1
   b. If dead condition: kill, recycle 50% energy to pool
   c. Execute one instruction
   d. Age += 1
3. Add children from DIVIDE
4. Periodic cleanup of dead organisms

**Round-robin scheduling**: every organism executes exactly one instruction per tick.

## Mutation

During DIVIDE:
1. Clone parent's code
2. For each instruction: with probability `mutation_rate`, replace with a random different instruction
3. Split energy 50/50, subtract divide_cost from parent
4. Child starts at IP=0, generation = parent + 1

## Seed Programs

**Seed A** (3 instructions): `EAT, REFRESH, JMP -2` — unconditional survival loop.

**Seed B** (8 instructions): `EAT, REFRESH, SENSE_SELF, CMP, JNZ, JMP, DIVIDE, JMP` — survival + conditional reproduction when energy > threshold.

## Theoretical Basis

The D0 VM implements the **operational closure** concept from Cognitive Life Science: a system whose own operations are necessary and sufficient for its continued existence. The freshness mechanism ensures that merely existing (having code) is not enough — the organism must *execute* specific operations (REFRESH) to maintain its body, creating the self-referential loop that characterizes life.

See the formal specification: `形式化设计-D0虚拟机规格.md` in the ExoMind knowledge base.
