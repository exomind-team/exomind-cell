//! D0 Virtual Machine — Operational Closure Experiment
//!
//! Implements a minimal artificial life system to test the hypothesis:
//! "Freshness decay (operational closure constraint) drives the evolution
//!  of conditional survival-priority behavior."
//!
//! Based on the Cognitive Life Science D0 spec v2.

use rand::prelude::*;
use std::fmt;
use std::fs;
use std::io::Write;

// ============================================================================
// Instruction Set
// ============================================================================

/// The D0 instruction set — minimal but sufficient for operational closure.
///
/// Design rationale: EAT + REFRESH are *necessary* for survival (operational closure).
/// DIVIDE is *optional* — reproduction is not the core of life, persistence is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Instruction {
    Nop,            // 0x00: No operation
    Inc(u8),        // 0x01: Register += 1
    Dec(u8),        // 0x02: Register -= 1
    Cmp(u8, u8),    // 0x03: Compare two registers, set flag in R3
    Jmp(i16),       // 0x04: Unconditional jump (relative offset)
    Jnz(i16),       // 0x05: Jump if R0 != 0
    Load(u8, u8),   // 0x06: Load from data[reg] to register
    Store(u8, u8),  // 0x07: Store register to data[reg]
    SenseSelf(u8),  // 0x08: Read own energy into register
    Eat,            // 0x09: Consume food from pool
    Refresh,        // 0x0A: Reset freshness (whole organism, simplified)
    Divide,         // 0x0B: Self-replicate with mutation
}

impl Instruction {
    /// Total number of instruction variants (for random generation/mutation).
    const VARIANT_COUNT: usize = 12;

    /// Generate a random instruction.
    fn random(rng: &mut impl Rng) -> Self {
        let variant = rng.gen_range(0..Self::VARIANT_COUNT);
        Self::from_variant(variant, rng)
    }

    /// Create instruction from variant index with random operands.
    fn from_variant(variant: usize, rng: &mut impl Rng) -> Self {
        match variant {
            0 => Instruction::Nop,
            1 => Instruction::Inc(rng.gen_range(0..8)),
            2 => Instruction::Dec(rng.gen_range(0..8)),
            3 => Instruction::Cmp(rng.gen_range(0..8), rng.gen_range(0..8)),
            4 => Instruction::Jmp(rng.gen_range(-16..16)),
            5 => Instruction::Jnz(rng.gen_range(-16..16)),
            6 => Instruction::Load(rng.gen_range(0..8), rng.gen_range(0..8)),
            7 => Instruction::Store(rng.gen_range(0..8), rng.gen_range(0..8)),
            8 => Instruction::SenseSelf(rng.gen_range(0..8)),
            9 => Instruction::Eat,
            10 => Instruction::Refresh,
            11 => Instruction::Divide,
            _ => Instruction::Nop,
        }
    }

    /// Mutate this instruction into a random different one.
    fn mutate(&self, rng: &mut impl Rng) -> Self {
        // Pick a different variant
        loop {
            let new = Self::random(rng);
            if std::mem::discriminant(&new) != std::mem::discriminant(self) {
                return new;
            }
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Inc(r) => write!(f, "INC r{}", r),
            Instruction::Dec(r) => write!(f, "DEC r{}", r),
            Instruction::Cmp(a, b) => write!(f, "CMP r{} r{}", a, b),
            Instruction::Jmp(off) => write!(f, "JMP {}", off),
            Instruction::Jnz(off) => write!(f, "JNZ {}", off),
            Instruction::Load(r, idx) => write!(f, "LOAD r{} d{}", r, idx),
            Instruction::Store(r, idx) => write!(f, "STORE r{} d{}", r, idx),
            Instruction::SenseSelf(r) => write!(f, "SENSE_SELF r{}", r),
            Instruction::Eat => write!(f, "EAT"),
            Instruction::Refresh => write!(f, "REFRESH"),
            Instruction::Divide => write!(f, "DIVIDE"),
        }
    }
}

// ============================================================================
// Organism
// ============================================================================

/// An organism in the D0 soup. Its code IS its body.
///
/// Operational closure: the organism must execute REFRESH to maintain freshness
/// and EAT to maintain energy. Failure in either → irreversible death.
#[derive(Debug, Clone)]
struct Organism {
    code: Vec<Instruction>,
    data: Vec<u8>,
    registers: [i32; 8],
    ip: usize,
    energy: i32,
    freshness: u8,       // Simplified: whole-organism freshness
    alive: bool,
    age: u64,
    generation: u32,

    // Per-tick instruction execution counters (for statistics)
    eat_count: u64,
    refresh_count: u64,
    divide_count: u64,
    total_instructions: u64,
}

impl Organism {
    fn new(code: Vec<Instruction>, energy: i32, freshness: u8) -> Self {
        Organism {
            code,
            data: vec![0u8; 8],
            registers: [0i32; 8],
            ip: 0,
            energy,
            freshness,
            alive: true,
            age: 0,
            generation: 0,
            eat_count: 0,
            refresh_count: 0,
            divide_count: 0,
            total_instructions: 0,
        }
    }
}

// ============================================================================
// World Configuration
// ============================================================================

#[derive(Debug, Clone)]
struct Config {
    max_organisms: usize,
    food_per_tick: i32,
    freshness_max: u8,
    freshness_decay: bool,  // KEY SWITCH: experimental (true) vs control (false)
    mutation_rate: f64,
    eat_energy: i32,
    refresh_cost: i32,
    divide_cost: i32,
    instruction_cost: i32,
    initial_energy: i32,
    total_ticks: u64,
    snapshot_interval: u64, // How often to record statistics
}

impl Config {
    /// Default experimental configuration.
    fn experimental() -> Self {
        Config {
            max_organisms: 100,
            food_per_tick: 5,
            freshness_max: 255,
            freshness_decay: true,  // EXPERIMENTAL: freshness decays
            mutation_rate: 0.001,
            eat_energy: 10,
            refresh_cost: 1,
            divide_cost: 30,
            instruction_cost: 1,
            initial_energy: 100,
            total_ticks: 100_000,
            snapshot_interval: 1000,
        }
    }

    /// Control group: identical except freshness never decays.
    fn control() -> Self {
        let mut c = Self::experimental();
        c.freshness_decay = false;
        c
    }
}

// ============================================================================
// Statistics Snapshot
// ============================================================================

/// A snapshot of world state at a given tick, used for analysis.
#[derive(Debug, Clone)]
struct Snapshot {
    tick: u64,
    population: usize,
    avg_energy: f64,
    avg_code_length: f64,
    avg_age: f64,
    avg_freshness: f64,
    total_eat: u64,
    total_refresh: u64,
    total_divide: u64,
    total_instructions: u64,
    // Behavioral ratios
    eat_ratio: f64,
    refresh_ratio: f64,
    divide_ratio: f64,
    // Conditional behavior metrics
    // Fraction of EAT executed when energy < 20% of initial
    low_energy_eat_rate: f64,
    // Fraction of REFRESH executed when freshness < 50
    low_freshness_refresh_rate: f64,
    max_generation: u32,
}

impl Snapshot {
    fn header() -> String {
        "tick,population,avg_energy,avg_code_length,avg_age,avg_freshness,\
         total_eat,total_refresh,total_divide,total_instructions,\
         eat_ratio,refresh_ratio,divide_ratio,\
         low_energy_eat_rate,low_freshness_refresh_rate,max_generation"
            .to_string()
    }

    fn to_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            self.tick,
            self.population,
            self.avg_energy,
            self.avg_code_length,
            self.avg_age,
            self.avg_freshness,
            self.total_eat,
            self.total_refresh,
            self.total_divide,
            self.total_instructions,
            self.eat_ratio,
            self.refresh_ratio,
            self.divide_ratio,
            self.low_energy_eat_rate,
            self.low_freshness_refresh_rate,
            self.max_generation,
        )
    }
}

// ============================================================================
// World
// ============================================================================

struct World {
    organisms: Vec<Organism>,
    food_pool: i32,
    tick: u64,
    config: Config,
    rng: StdRng,
    snapshots: Vec<Snapshot>,

    // Accumulated counters for snapshot interval
    interval_eat: u64,
    interval_refresh: u64,
    interval_divide: u64,
    interval_instructions: u64,
    // Conditional behavior tracking within interval
    low_energy_eats: u64,
    low_energy_total_instructions: u64,
    low_freshness_refreshes: u64,
    low_freshness_total_instructions: u64,
}

impl World {
    fn new(config: Config, seed: u64) -> Self {
        World {
            organisms: Vec::new(),
            food_pool: 100, // Initial food pool
            tick: 0,
            config,
            rng: StdRng::seed_from_u64(seed),
            snapshots: Vec::new(),
            interval_eat: 0,
            interval_refresh: 0,
            interval_divide: 0,
            interval_instructions: 0,
            low_energy_eats: 0,
            low_energy_total_instructions: 0,
            low_freshness_refreshes: 0,
            low_freshness_total_instructions: 0,
        }
    }

    /// Add an organism to the world.
    fn add_organism(&mut self, org: Organism) {
        if self.organisms.len() < self.config.max_organisms {
            self.organisms.push(org);
        }
    }

    /// Take a statistics snapshot.
    fn take_snapshot(&mut self) {
        let alive: Vec<&Organism> = self.organisms.iter().filter(|o| o.alive).collect();
        let n = alive.len();

        if n == 0 {
            self.snapshots.push(Snapshot {
                tick: self.tick,
                population: 0,
                avg_energy: 0.0,
                avg_code_length: 0.0,
                avg_age: 0.0,
                avg_freshness: 0.0,
                total_eat: self.interval_eat,
                total_refresh: self.interval_refresh,
                total_divide: self.interval_divide,
                total_instructions: self.interval_instructions,
                eat_ratio: 0.0,
                refresh_ratio: 0.0,
                divide_ratio: 0.0,
                low_energy_eat_rate: 0.0,
                low_freshness_refresh_rate: 0.0,
                max_generation: 0,
            });
        } else {
            let total_instr = self.interval_instructions.max(1) as f64;
            let low_e_total = self.low_energy_total_instructions.max(1) as f64;
            let low_f_total = self.low_freshness_total_instructions.max(1) as f64;

            self.snapshots.push(Snapshot {
                tick: self.tick,
                population: n,
                avg_energy: alive.iter().map(|o| o.energy as f64).sum::<f64>() / n as f64,
                avg_code_length: alive.iter().map(|o| o.code.len() as f64).sum::<f64>() / n as f64,
                avg_age: alive.iter().map(|o| o.age as f64).sum::<f64>() / n as f64,
                avg_freshness: alive.iter().map(|o| o.freshness as f64).sum::<f64>() / n as f64,
                total_eat: self.interval_eat,
                total_refresh: self.interval_refresh,
                total_divide: self.interval_divide,
                total_instructions: self.interval_instructions,
                eat_ratio: self.interval_eat as f64 / total_instr,
                refresh_ratio: self.interval_refresh as f64 / total_instr,
                divide_ratio: self.interval_divide as f64 / total_instr,
                low_energy_eat_rate: self.low_energy_eats as f64 / low_e_total,
                low_freshness_refresh_rate: self.low_freshness_refreshes as f64 / low_f_total,
                max_generation: alive.iter().map(|o| o.generation).max().unwrap_or(0),
            });
        }

        // Reset interval counters
        self.interval_eat = 0;
        self.interval_refresh = 0;
        self.interval_divide = 0;
        self.interval_instructions = 0;
        self.low_energy_eats = 0;
        self.low_energy_total_instructions = 0;
        self.low_freshness_refreshes = 0;
        self.low_freshness_total_instructions = 0;
    }

    /// Execute one instruction for a given organism. Returns an optional new organism (from DIVIDE).
    fn execute_instruction(&mut self, org_idx: usize) -> Option<Organism> {
        // Cache values we need before taking a mutable borrow
        let org_count = self.organisms.len();
        let org = &mut self.organisms[org_idx];

        if !org.alive || org.code.is_empty() {
            return None;
        }

        // Wrap IP
        org.ip %= org.code.len();
        let instr = org.code[org.ip];

        // Track conditional behavior: is the organism in a stressed state?
        let low_energy = org.energy < (self.config.initial_energy / 5); // < 20%
        let low_freshness = org.freshness < 50;

        if low_energy {
            self.low_energy_total_instructions += 1;
        }
        if low_freshness {
            self.low_freshness_total_instructions += 1;
        }

        // Energy cost for executing any instruction
        org.energy -= self.config.instruction_cost;
        org.total_instructions += 1;
        self.interval_instructions += 1;

        let mut new_organism: Option<Organism> = None;

        match instr {
            Instruction::Nop => {
                org.ip += 1;
            }

            Instruction::Inc(r) => {
                let r = (r as usize) % 8;
                org.registers[r] = org.registers[r].wrapping_add(1);
                org.ip += 1;
            }

            Instruction::Dec(r) => {
                let r = (r as usize) % 8;
                org.registers[r] = org.registers[r].wrapping_sub(1);
                org.ip += 1;
            }

            Instruction::Cmp(ra, rb) => {
                let a = org.registers[(ra as usize) % 8];
                let b = org.registers[(rb as usize) % 8];
                // R0 = (a > b) as i32 — so JNZ triggers when a > b
                org.registers[0] = if a > b { 1 } else { 0 };
                org.ip += 1;
            }

            Instruction::Jmp(offset) => {
                let new_ip = org.ip as i64 + offset as i64;
                let len = org.code.len() as i64;
                org.ip = ((new_ip % len + len) % len) as usize;
            }

            Instruction::Jnz(offset) => {
                if org.registers[0] != 0 {
                    let new_ip = org.ip as i64 + offset as i64;
                    let len = org.code.len() as i64;
                    org.ip = ((new_ip % len + len) % len) as usize;
                } else {
                    org.ip += 1;
                }
            }

            Instruction::Load(r, _idx) => {
                let r = (r as usize) % 8;
                if !org.data.is_empty() {
                    let idx = (org.registers[1] as usize) % org.data.len();
                    org.registers[r] = org.data[idx] as i32;
                }
                org.ip += 1;
            }

            Instruction::Store(r, _idx) => {
                let r = (r as usize) % 8;
                if !org.data.is_empty() {
                    let idx = (org.registers[1] as usize) % org.data.len();
                    org.data[idx] = (org.registers[r] & 0xFF) as u8;
                }
                org.ip += 1;
            }

            Instruction::SenseSelf(r) => {
                let r = (r as usize) % 8;
                org.registers[r] = org.energy;
                org.ip += 1;
            }

            Instruction::Eat => {
                org.eat_count += 1;
                self.interval_eat += 1;
                if low_energy {
                    self.low_energy_eats += 1;
                }
                if self.food_pool >= self.config.eat_energy {
                    self.food_pool -= self.config.eat_energy;
                    org.energy += self.config.eat_energy;
                } else if self.food_pool > 0 {
                    org.energy += self.food_pool;
                    self.food_pool = 0;
                }
                org.ip += 1;
            }

            Instruction::Refresh => {
                org.refresh_count += 1;
                self.interval_refresh += 1;
                if low_freshness {
                    self.low_freshness_refreshes += 1;
                }
                org.energy -= self.config.refresh_cost;
                org.freshness = self.config.freshness_max;
                org.ip += 1;
            }

            Instruction::Divide => {
                org.divide_count += 1;
                self.interval_divide += 1;

                // Use cached org_count to avoid re-borrowing self.organisms
                if org.energy >= self.config.divide_cost * 2
                    && org_count < self.config.max_organisms
                {
                    // Clone code and prepare child data while holding mutable borrow
                    let mut child_code = org.code.clone();
                    let child_energy = org.energy / 2;
                    let child_generation = org.generation + 1;
                    org.energy -= child_energy;
                    org.energy -= self.config.divide_cost;

                    // Mutate child code
                    let mutation_rate = self.config.mutation_rate;
                    for instr in child_code.iter_mut() {
                        if self.rng.gen_bool(mutation_rate.min(1.0)) {
                            *instr = instr.mutate(&mut self.rng);
                        }
                    }

                    let mut child = Organism::new(
                        child_code,
                        child_energy,
                        self.config.freshness_max,
                    );
                    child.generation = child_generation;
                    new_organism = Some(child);
                }
                org.ip += 1;
            }
        }

        new_organism
    }

    /// Run one tick of the simulation.
    fn tick(&mut self) {
        // 1. Add food to the pool
        self.food_pool += self.config.food_per_tick;

        // 2. For each alive organism:
        let n = self.organisms.len();
        let mut new_organisms: Vec<Organism> = Vec::new();

        for i in 0..n {
            if !self.organisms[i].alive {
                continue;
            }

            // a. Freshness decay (if enabled)
            if self.config.freshness_decay {
                self.organisms[i].freshness = self.organisms[i].freshness.saturating_sub(1);
            }

            // b. Check death conditions
            if self.organisms[i].freshness == 0 || self.organisms[i].energy <= 0 {
                self.organisms[i].alive = false;
                // Return energy to food pool (partial recycling)
                let recycled = self.organisms[i].energy.max(0) / 2;
                self.food_pool += recycled;
                continue;
            }

            // c. Execute one instruction
            if let Some(child) = self.execute_instruction(i) {
                new_organisms.push(child);
            }

            // d. Increment age
            self.organisms[i].age += 1;

            // e. Post-execution death check (energy might have gone negative)
            if self.organisms[i].energy <= 0 {
                self.organisms[i].alive = false;
                self.food_pool += 0; // already depleted
            }
        }

        // 3. Add children
        for child in new_organisms {
            self.add_organism(child);
        }

        // 4. Clean up dead organisms (keep the vec manageable)
        if self.tick % 100 == 0 {
            self.organisms.retain(|o| o.alive);
        }

        self.tick += 1;

        // 5. Take snapshot if needed
        if self.tick % self.config.snapshot_interval == 0 {
            self.take_snapshot();
        }
    }

    /// Run the full simulation.
    fn run(&mut self) {
        let total = self.config.total_ticks;
        // Take initial snapshot
        self.take_snapshot();

        for t in 0..total {
            self.tick();

            // Progress reporting every 10%
            if t % (total / 10) == 0 && t > 0 {
                let alive = self.organisms.iter().filter(|o| o.alive).count();
                eprintln!(
                    "  tick {}/{} ({:.0}%) — {} alive, food_pool={}",
                    t,
                    total,
                    (t as f64 / total as f64) * 100.0,
                    alive,
                    self.food_pool
                );
            }

            // Early termination if everything is dead
            if self.organisms.iter().all(|o| !o.alive) && self.tick > 1000 {
                eprintln!("  All organisms dead at tick {}. Ending early.", self.tick);
                self.take_snapshot();
                break;
            }
        }

        // Final snapshot
        if self.tick % self.config.snapshot_interval != 0 {
            self.take_snapshot();
        }
    }

    /// Export snapshots to CSV.
    fn export_csv(&self, path: &str) {
        let mut file = fs::File::create(path).expect("Failed to create CSV file");
        writeln!(file, "{}", Snapshot::header()).unwrap();
        for s in &self.snapshots {
            writeln!(file, "{}", s.to_csv()).unwrap();
        }
        eprintln!("  Exported {} snapshots to {}", self.snapshots.len(), path);
    }
}

// ============================================================================
// Seed Organisms
// ============================================================================

/// Seed A: Minimal self-sustaining loop.
///
/// ```text
/// loop:
///   SenseSelf r0    // read energy into r0
///   Cmp r0, r1      // compare energy (r0) with r1 (initially 0, acts as threshold)
///   Jnz skip_eat    // if energy > threshold (r0 > r1), skip eating
///   Eat              // eat to get energy
/// skip_eat:
///   Refresh          // refresh freshness
///   Jmp loop         // back to start
/// ```
fn seed_a(config: &Config) -> Organism {
    let code = vec![
        Instruction::SenseSelf(0),  // 0: r0 = energy
        Instruction::Cmp(0, 1),     // 1: compare r0 with r1 (r1=0 initially)
        Instruction::Jnz(2),       // 2: if r0 > r1, skip to index 4 (ip+2)
        Instruction::Eat,           // 3: eat
        Instruction::Refresh,       // 4: refresh freshness
        Instruction::Jmp(-5),       // 5: jump back to 0 (ip + (-5) = 0)
    ];
    Organism::new(code, config.initial_energy, config.freshness_max)
}

/// Seed B: Self-sustaining + conditional division.
///
/// ```text
/// loop:
///   SenseSelf r0    // read energy
///   Cmp r0, r2      // compare energy with r2 (threshold for eating, initially 0)
///   Jnz has_energy
///   Eat
///   Refresh
///   Jmp loop
/// has_energy:
///   Cmp r0, r3      // compare energy with r3 (threshold for dividing)
///   Jnz can_divide
///   Eat
///   Refresh
///   Jmp loop
/// can_divide:
///   Divide
///   Refresh
///   Jmp loop
/// ```
fn seed_b(config: &Config) -> Organism {
    let code = vec![
        Instruction::SenseSelf(0),  // 0: r0 = energy
        Instruction::Inc(3),        // 1: r3++ (builds up divide threshold over time)
        Instruction::Cmp(0, 2),     // 2: compare r0 with r2
        Instruction::Jnz(3),       // 3: if r0 > r2, jump to 6 (has_energy)
        Instruction::Eat,           // 4: eat
        Instruction::Refresh,       // 5: refresh
        Instruction::Jmp(-6),       // 6: would go to 0, but this is also has_energy target
        // has_energy:
        Instruction::Cmp(0, 3),     // 7: compare r0 with r3
        Instruction::Jnz(3),       // 8: if r0 > r3, jump to 11 (can_divide)
        Instruction::Eat,           // 9: eat
        Instruction::Refresh,       // 10: refresh
        Instruction::Jmp(-11),      // 11: jump back to 0
        // can_divide:
        Instruction::Divide,        // 12: divide!
        Instruction::Refresh,       // 13: refresh after dividing
        Instruction::Jmp(-14),      // 14: jump back to 0
    ];

    let mut org = Organism::new(code, config.initial_energy, config.freshness_max);
    // Set initial register values for better thresholds
    org.registers[2] = 3;   // eat threshold: eat when energy <= 3
    org.registers[3] = 10;  // divide threshold: divide when energy > 10
    org
}

// ============================================================================
// Experiment Runner
// ============================================================================

/// Run one experiment (either experimental or control group).
fn run_experiment(name: &str, config: Config, seed: u64) -> Vec<Snapshot> {
    eprintln!("\n========================================");
    eprintln!("Running experiment: {}", name);
    eprintln!("  freshness_decay = {}", config.freshness_decay);
    eprintln!("  max_organisms = {}", config.max_organisms);
    eprintln!("  food_per_tick = {}", config.food_per_tick);
    eprintln!("  mutation_rate = {}", config.mutation_rate);
    eprintln!("  total_ticks = {}", config.total_ticks);
    eprintln!("========================================");

    let mut world = World::new(config.clone(), seed);

    // Seed the world with 10 seed A + 10 seed B organisms
    for _ in 0..10 {
        world.add_organism(seed_a(&config));
    }
    for _ in 0..10 {
        world.add_organism(seed_b(&config));
    }

    world.run();

    // Export CSV
    let csv_path = format!("D:/project/d0-vm/{}.csv", name.replace(' ', "_"));
    world.export_csv(&csv_path);

    world.snapshots
}

// ============================================================================
// Analysis and Report Generation
// ============================================================================

fn analyze_and_report(
    exp_snapshots: &[Snapshot],
    ctrl_snapshots: &[Snapshot],
) -> String {
    let mut report = String::new();

    report.push_str("# D0 Virtual Machine — Experiment Results\n\n");
    report.push_str("## Experiment Overview\n\n");
    report.push_str("**Hypothesis**: Freshness decay (operational closure constraint) drives evolution of conditional survival-priority behavior.\n\n");
    report.push_str("| Parameter | Value |\n");
    report.push_str("|-----------|-------|\n");
    report.push_str("| Population cap | 100 |\n");
    report.push_str("| Initial organisms | 10 Seed A + 10 Seed B |\n");
    report.push_str("| Food per tick | 5 |\n");
    report.push_str("| Mutation rate | 0.001 |\n");
    report.push_str("| Total ticks | 100,000 |\n");
    report.push_str("| Freshness max | 255 |\n");
    report.push_str("| Eat energy | 10 |\n");
    report.push_str("| Instruction cost | 1 |\n\n");

    // --- Experimental Group Summary ---
    report.push_str("## Experimental Group (freshness_decay = true)\n\n");
    if let Some(last) = exp_snapshots.last() {
        report.push_str(&format!("- **Final tick**: {}\n", last.tick));
        report.push_str(&format!("- **Final population**: {}\n", last.population));
        report.push_str(&format!("- **Average energy**: {:.2}\n", last.avg_energy));
        report.push_str(&format!("- **Average code length**: {:.2}\n", last.avg_code_length));
        report.push_str(&format!("- **Average freshness**: {:.2}\n", last.avg_freshness));
        report.push_str(&format!("- **Max generation**: {}\n", last.max_generation));
    }
    report.push_str("\n### Population Dynamics\n\n");
    report.push_str("| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|------|-----------|-----------|-------------|------|----------|--------|\n");
    for s in exp_snapshots.iter().step_by(
        if exp_snapshots.len() > 20 { exp_snapshots.len() / 20 } else { 1 }
    ) {
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} |\n",
            s.tick, s.population, s.avg_energy, s.avg_code_length,
            s.eat_ratio * 100.0, s.refresh_ratio * 100.0, s.divide_ratio * 100.0,
        ));
    }
    // Always include the last snapshot
    if let Some(last) = exp_snapshots.last() {
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} |\n",
            last.tick, last.population, last.avg_energy, last.avg_code_length,
            last.eat_ratio * 100.0, last.refresh_ratio * 100.0, last.divide_ratio * 100.0,
        ));
    }

    // --- Control Group Summary ---
    report.push_str("\n## Control Group (freshness_decay = false)\n\n");
    if let Some(last) = ctrl_snapshots.last() {
        report.push_str(&format!("- **Final tick**: {}\n", last.tick));
        report.push_str(&format!("- **Final population**: {}\n", last.population));
        report.push_str(&format!("- **Average energy**: {:.2}\n", last.avg_energy));
        report.push_str(&format!("- **Average code length**: {:.2}\n", last.avg_code_length));
        report.push_str(&format!("- **Average freshness**: {:.2}\n", last.avg_freshness));
        report.push_str(&format!("- **Max generation**: {}\n", last.max_generation));
    }
    report.push_str("\n### Population Dynamics\n\n");
    report.push_str("| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|------|-----------|-----------|-------------|------|----------|--------|\n");
    for s in ctrl_snapshots.iter().step_by(
        if ctrl_snapshots.len() > 20 { ctrl_snapshots.len() / 20 } else { 1 }
    ) {
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} |\n",
            s.tick, s.population, s.avg_energy, s.avg_code_length,
            s.eat_ratio * 100.0, s.refresh_ratio * 100.0, s.divide_ratio * 100.0,
        ));
    }
    if let Some(last) = ctrl_snapshots.last() {
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} |\n",
            last.tick, last.population, last.avg_energy, last.avg_code_length,
            last.eat_ratio * 100.0, last.refresh_ratio * 100.0, last.divide_ratio * 100.0,
        ));
    }

    // --- Comparative Analysis ---
    report.push_str("\n## Comparative Analysis\n\n");

    // Calculate averages over the second half (steady state)
    let exp_second_half: Vec<&Snapshot> = exp_snapshots
        .iter()
        .filter(|s| s.tick > 50000 && s.population > 0)
        .collect();
    let ctrl_second_half: Vec<&Snapshot> = ctrl_snapshots
        .iter()
        .filter(|s| s.tick > 50000 && s.population > 0)
        .collect();

    if !exp_second_half.is_empty() && !ctrl_second_half.is_empty() {
        let exp_avg_eat: f64 = exp_second_half.iter().map(|s| s.eat_ratio).sum::<f64>()
            / exp_second_half.len() as f64;
        let exp_avg_refresh: f64 = exp_second_half.iter().map(|s| s.refresh_ratio).sum::<f64>()
            / exp_second_half.len() as f64;
        let exp_avg_divide: f64 = exp_second_half.iter().map(|s| s.divide_ratio).sum::<f64>()
            / exp_second_half.len() as f64;
        let exp_avg_low_e_eat: f64 = exp_second_half.iter().map(|s| s.low_energy_eat_rate).sum::<f64>()
            / exp_second_half.len() as f64;
        let exp_avg_low_f_ref: f64 = exp_second_half.iter().map(|s| s.low_freshness_refresh_rate).sum::<f64>()
            / exp_second_half.len() as f64;

        let ctrl_avg_eat: f64 = ctrl_second_half.iter().map(|s| s.eat_ratio).sum::<f64>()
            / ctrl_second_half.len() as f64;
        let ctrl_avg_refresh: f64 = ctrl_second_half.iter().map(|s| s.refresh_ratio).sum::<f64>()
            / ctrl_second_half.len() as f64;
        let ctrl_avg_divide: f64 = ctrl_second_half.iter().map(|s| s.divide_ratio).sum::<f64>()
            / ctrl_second_half.len() as f64;
        let ctrl_avg_low_e_eat: f64 = ctrl_second_half.iter().map(|s| s.low_energy_eat_rate).sum::<f64>()
            / ctrl_second_half.len() as f64;

        report.push_str("### Steady-State Averages (tick 50k-100k)\n\n");
        report.push_str("| Metric | Experimental | Control | Difference |\n");
        report.push_str("|--------|-------------|---------|------------|\n");
        report.push_str(&format!(
            "| EAT ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_avg_eat, ctrl_avg_eat, exp_avg_eat - ctrl_avg_eat
        ));
        report.push_str(&format!(
            "| REFRESH ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_avg_refresh, ctrl_avg_refresh, exp_avg_refresh - ctrl_avg_refresh
        ));
        report.push_str(&format!(
            "| DIVIDE ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_avg_divide, ctrl_avg_divide, exp_avg_divide - ctrl_avg_divide
        ));
        report.push_str(&format!(
            "| Low-energy EAT rate | {:.4} | {:.4} | {:.4} |\n",
            exp_avg_low_e_eat, ctrl_avg_low_e_eat, exp_avg_low_e_eat - ctrl_avg_low_e_eat
        ));
        report.push_str(&format!(
            "| Low-freshness REFRESH rate | {:.4} | N/A | — |\n",
            exp_avg_low_f_ref
        ));

        // Average population
        let exp_avg_pop: f64 = exp_second_half.iter().map(|s| s.population as f64).sum::<f64>()
            / exp_second_half.len() as f64;
        let ctrl_avg_pop: f64 = ctrl_second_half.iter().map(|s| s.population as f64).sum::<f64>()
            / ctrl_second_half.len() as f64;
        report.push_str(&format!(
            "| Avg population | {:.1} | {:.1} | {:.1} |\n",
            exp_avg_pop, ctrl_avg_pop, exp_avg_pop - ctrl_avg_pop
        ));

        let exp_avg_gen: f64 = exp_second_half.iter().map(|s| s.max_generation as f64).sum::<f64>()
            / exp_second_half.len() as f64;
        let ctrl_avg_gen: f64 = ctrl_second_half.iter().map(|s| s.max_generation as f64).sum::<f64>()
            / ctrl_second_half.len() as f64;
        report.push_str(&format!(
            "| Max generation | {:.1} | {:.1} | {:.1} |\n",
            exp_avg_gen, ctrl_avg_gen, exp_avg_gen - ctrl_avg_gen
        ));
    }

    // --- Key Questions ---
    report.push_str("\n### Key Experimental Questions\n\n");

    report.push_str("**Q1: Did the experimental group evolve \"low energy → prioritize EAT/REFRESH over DIVIDE\" behavior?**\n\n");
    if let Some(last_exp) = exp_snapshots.last() {
        if last_exp.population > 0 {
            if last_exp.eat_ratio > 0.05 && last_exp.refresh_ratio > 0.05 {
                report.push_str("YES — Both EAT and REFRESH remain significant in the instruction mix, ");
                report.push_str("indicating organisms evolved to maintain both energy and freshness.\n\n");
            } else if last_exp.eat_ratio > 0.05 {
                report.push_str("PARTIAL — EAT is significant but REFRESH frequency is low. ");
                report.push_str("Organisms may have evolved to eat frequently enough that freshness rarely drops.\n\n");
            } else {
                report.push_str("UNCLEAR — Instruction ratios do not show a clear survival-priority pattern.\n\n");
            }
        } else {
            report.push_str("POPULATION EXTINCT — Cannot assess evolved behavior.\n\n");
        }
    }

    report.push_str("**Q2: Does the control group lack this conditional behavior?**\n\n");
    if let Some(last_ctrl) = ctrl_snapshots.last() {
        if last_ctrl.population > 0 {
            if last_ctrl.refresh_ratio < 0.01 {
                report.push_str("YES — Control group has negligible REFRESH usage, confirming that ");
                report.push_str("without freshness decay, REFRESH provides no survival advantage and is not selected for.\n\n");
            } else {
                report.push_str("NO — Control group still shows REFRESH usage. This may indicate ");
                report.push_str("REFRESH is being retained for other reasons (e.g., genetic drift).\n\n");
            }
        } else {
            report.push_str("POPULATION EXTINCT in control group.\n\n");
        }
    }

    report.push_str("**Q3: Population dynamics differences?**\n\n");
    let exp_survived = exp_snapshots.last().map(|s| s.population > 0).unwrap_or(false);
    let ctrl_survived = ctrl_snapshots.last().map(|s| s.population > 0).unwrap_or(false);
    report.push_str(&format!(
        "- Experimental group survived to end: **{}**\n",
        if exp_survived { "YES" } else { "NO" }
    ));
    report.push_str(&format!(
        "- Control group survived to end: **{}**\n",
        if ctrl_survived { "YES" } else { "NO" }
    ));

    report.push_str("\n**Q4: Code evolution direction differences?**\n\n");
    if let (Some(first_exp), Some(last_exp)) = (exp_snapshots.first(), exp_snapshots.last()) {
        report.push_str(&format!(
            "- Experimental: code length {:.1} → {:.1}\n",
            first_exp.avg_code_length, last_exp.avg_code_length
        ));
    }
    if let (Some(first_ctrl), Some(last_ctrl)) = (ctrl_snapshots.first(), ctrl_snapshots.last()) {
        report.push_str(&format!(
            "- Control: code length {:.1} → {:.1}\n",
            first_ctrl.avg_code_length, last_ctrl.avg_code_length
        ));
    }

    report.push_str("\n---\n\n");
    report.push_str("## Methodology Notes\n\n");
    report.push_str("- Both groups use the same random seed for reproducibility\n");
    report.push_str("- Seed A = minimal self-sustaining (EAT + REFRESH loop)\n");
    report.push_str("- Seed B = self-sustaining + conditional DIVIDE\n");
    report.push_str("- Mutation: per-instruction replacement with probability 0.001 during DIVIDE\n");
    report.push_str("- Statistics sampled every 1000 ticks\n");
    report.push_str("- CSV data files available for detailed analysis\n\n");

    report.push_str("## Raw Data Files\n\n");
    report.push_str("- `experimental_group.csv` — Experimental group snapshots\n");
    report.push_str("- `control_group.csv` — Control group snapshots\n\n");

    report.push_str("---\n\n");
    report.push_str("*Generated by D0 VM v0.1.0 — Cognitive Life Science operational closure experiment*\n");

    report
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    eprintln!("D0 Virtual Machine — Operational Closure Experiment");
    eprintln!("====================================================\n");

    let seed = 42u64; // Fixed seed for reproducibility

    // Experiment 1: Experimental group (freshness_decay = true)
    let exp_config = Config::experimental();
    let exp_snapshots = run_experiment("experimental_group", exp_config, seed);

    // Experiment 2: Control group (freshness_decay = false)
    let ctrl_config = Config::control();
    let ctrl_snapshots = run_experiment("control_group", ctrl_config, seed);

    // Generate analysis report
    let report = analyze_and_report(&exp_snapshots, &ctrl_snapshots);

    // Write RESULTS.md
    fs::write("D:/project/d0-vm/RESULTS.md", &report).expect("Failed to write RESULTS.md");
    eprintln!("\nResults written to RESULTS.md");

    // Print summary to stdout
    println!("{}", report);
}
