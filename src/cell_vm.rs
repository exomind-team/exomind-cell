//! Cell-based D0 VM v3 — unified Cell type system with per-cell freshness.
//!
//! Key change from v2: organisms are composed of heterogeneous Cells
//! (Code/Energy/Stomach), each with independent freshness decay.
//! Energy is distributed across Energy cells, not a global counter.

use rand::prelude::*;
use std::fs;
use std::io::Write as IoWrite;
use std::fmt;

use crate::instruction::Instruction;

// ============================================================================
// Cell Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellType {
    Code(Instruction),
    Energy(u8),    // stored energy (0..cell_energy_max)
    Stomach(u8),   // undigested food (0..cell_energy_max)
    Data(u8),      // writable storage (for experience/memory, D2 prep)
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub content: CellType,
    pub freshness: u8,
}

impl Cell {
    pub fn code(instr: Instruction, freshness: u8) -> Self {
        Cell { content: CellType::Code(instr), freshness }
    }
    pub fn energy(amount: u8, freshness: u8) -> Self {
        Cell { content: CellType::Energy(amount), freshness }
    }
    pub fn stomach(amount: u8, freshness: u8) -> Self {
        Cell { content: CellType::Stomach(amount), freshness }
    }
    pub fn data(val: u8, freshness: u8) -> Self {
        Cell { content: CellType::Data(val), freshness }
    }
    pub fn is_code(&self) -> bool { matches!(self.content, CellType::Code(_)) }
    pub fn is_energy(&self) -> bool { matches!(self.content, CellType::Energy(_)) }
    pub fn is_stomach(&self) -> bool { matches!(self.content, CellType::Stomach(_)) }
    pub fn is_data(&self) -> bool { matches!(self.content, CellType::Data(_)) }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            CellType::Code(instr) => write!(f, "Code({}) f={}", instr, self.freshness),
            CellType::Energy(e) => write!(f, "Energy({}) f={}", e, self.freshness),
            CellType::Stomach(s) => write!(f, "Stomach({}) f={}", s, self.freshness),
            CellType::Data(d) => write!(f, "Data({}) f={}", d, self.freshness),
        }
    }
}

// ============================================================================
// Organism v3
// ============================================================================

#[derive(Debug, Clone)]
pub struct CellOrganism {
    pub cells: Vec<Cell>,
    pub registers: [i32; 8],
    pub ip: usize,          // index into code cells (not all cells)
    pub alive: bool,
    pub age: u64,
    pub generation: u32,

    // Counters
    pub eat_count: u64,
    pub digest_count: u64,
    pub refresh_count: u64,
    pub divide_count: u64,
    pub total_instructions: u64,
}

impl CellOrganism {
    pub fn new(cells: Vec<Cell>) -> Self {
        CellOrganism {
            cells,
            registers: [0i32; 8],
            ip: 0,
            alive: true,
            age: 0,
            generation: 0,
            eat_count: 0,
            digest_count: 0,
            refresh_count: 0,
            divide_count: 0,
            total_instructions: 0,
        }
    }

    /// Total energy across all Energy cells.
    pub fn total_energy(&self) -> i32 {
        self.cells.iter().filter_map(|c| match c.content {
            CellType::Energy(e) => Some(e as i32),
            _ => None,
        }).sum()
    }

    /// Number of Code cells.
    pub fn code_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_code()).count()
    }

    /// Get the nth Code cell's instruction.
    fn nth_code_instruction(&self, n: usize) -> Option<Instruction> {
        self.cells.iter()
            .filter(|c| c.is_code())
            .nth(n)
            .and_then(|c| match c.content {
                CellType::Code(instr) => Some(instr),
                _ => None,
            })
    }

    /// Min freshness across all cells.
    pub fn min_freshness(&self) -> u8 {
        self.cells.iter().map(|c| c.freshness).min().unwrap_or(0)
    }

    /// Deduct 1 energy from the first non-empty Energy cell. Returns false if no energy.
    fn deduct_energy(&mut self, amount: u8) -> bool {
        for cell in self.cells.iter_mut() {
            if let CellType::Energy(ref mut e) = cell.content {
                if *e >= amount {
                    *e -= amount;
                    return true;
                }
            }
        }
        // Try partial: deduct across multiple energy cells
        let mut remaining = amount;
        for cell in self.cells.iter_mut() {
            if remaining == 0 { break; }
            if let CellType::Energy(ref mut e) = cell.content {
                let take = (*e).min(remaining);
                *e -= take;
                remaining -= take;
            }
        }
        remaining == 0
    }

    /// Find first Stomach cell with space.
    fn first_empty_stomach(&mut self, max: u8) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| match c.content {
            CellType::Stomach(s) => s < max,
            _ => false,
        })
    }

    /// Find first non-empty Stomach cell.
    fn first_full_stomach(&mut self) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| match c.content {
            CellType::Stomach(s) => s > 0,
            _ => false,
        })
    }

    /// Find first Energy cell with space.
    fn first_available_energy(&mut self, max: u8) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| match c.content {
            CellType::Energy(e) => e < max,
            _ => false,
        })
    }
}

// ============================================================================
// Config v3
// ============================================================================

#[derive(Debug, Clone)]
pub struct CellConfig {
    pub max_organisms: usize,
    pub food_per_tick: i32,
    pub freshness_max: u8,
    pub freshness_decay: bool,
    pub mutation_rate: f64,
    pub cell_energy_max: u8,
    pub refresh_radius: usize,   // REFRESH covers ip-R to ip+R
    pub instruction_cost: u8,    // energy per instruction
    pub divide_cost: u8,         // extra energy for DIVIDE
    pub total_ticks: u64,
    pub snapshot_interval: u64,
    pub genome_dump_interval: u64,
}

impl CellConfig {
    pub fn experimental() -> Self {
        CellConfig {
            max_organisms: 100,
            food_per_tick: 50,
            freshness_max: 255,
            freshness_decay: true,
            mutation_rate: 0.001,
            cell_energy_max: 20,
            refresh_radius: 5,       // covers ip-5 to ip+5 = 11 cells (enough for small organisms)
            instruction_cost: 1,
            divide_cost: 5,
            total_ticks: 500_000,
            snapshot_interval: 1000,
            genome_dump_interval: 50_000,
        }
    }

    pub fn control() -> Self {
        let mut c = Self::experimental();
        c.freshness_decay = false;
        c
    }
}

// ============================================================================
// Snapshot
// ============================================================================

#[derive(Debug, Clone)]
pub struct CellSnapshot {
    pub tick: u64,
    pub population: usize,
    pub avg_energy: f64,
    pub avg_cell_count: f64,
    pub avg_code_count: f64,
    pub avg_freshness: f64,
    pub max_generation: u32,
    pub eat_ratio: f64,
    pub digest_ratio: f64,
    pub refresh_ratio: f64,
    pub divide_ratio: f64,
    pub total_instructions: u64,
}

impl CellSnapshot {
    pub fn header() -> String {
        "tick,population,avg_energy,avg_cell_count,avg_code_count,avg_freshness,\
         max_generation,eat_ratio,digest_ratio,refresh_ratio,divide_ratio,total_instructions"
            .to_string()
    }

    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{},{:.6},{:.6},{:.6},{:.6},{}",
            self.tick, self.population, self.avg_energy, self.avg_cell_count,
            self.avg_code_count, self.avg_freshness, self.max_generation,
            self.eat_ratio, self.digest_ratio, self.refresh_ratio, self.divide_ratio,
            self.total_instructions,
        )
    }
}

// ============================================================================
// World v3
// ============================================================================

pub struct CellWorld {
    pub organisms: Vec<CellOrganism>,
    pub food_pool: i32,
    pub tick: u64,
    pub config: CellConfig,
    pub rng: StdRng,
    pub snapshots: Vec<CellSnapshot>,

    // Interval counters
    interval_eat: u64,
    interval_digest: u64,
    interval_refresh: u64,
    interval_divide: u64,
    interval_instructions: u64,
}

impl CellWorld {
    pub fn new(config: CellConfig, seed: u64) -> Self {
        CellWorld {
            organisms: Vec::new(),
            food_pool: 500,
            tick: 0,
            config,
            rng: StdRng::seed_from_u64(seed),
            snapshots: Vec::new(),
            interval_eat: 0,
            interval_digest: 0,
            interval_refresh: 0,
            interval_divide: 0,
            interval_instructions: 0,
        }
    }

    pub fn add_organism(&mut self, org: CellOrganism) {
        if self.organisms.len() < self.config.max_organisms {
            self.organisms.push(org);
        }
    }

    fn take_snapshot(&mut self) {
        let alive: Vec<&CellOrganism> = self.organisms.iter().filter(|o| o.alive).collect();
        let n = alive.len();
        let total = self.interval_instructions.max(1) as f64;

        if n == 0 {
            self.snapshots.push(CellSnapshot {
                tick: self.tick, population: 0, avg_energy: 0.0,
                avg_cell_count: 0.0, avg_code_count: 0.0, avg_freshness: 0.0,
                max_generation: 0, eat_ratio: 0.0, digest_ratio: 0.0,
                refresh_ratio: 0.0, divide_ratio: 0.0, total_instructions: self.interval_instructions,
            });
        } else {
            self.snapshots.push(CellSnapshot {
                tick: self.tick,
                population: n,
                avg_energy: alive.iter().map(|o| o.total_energy() as f64).sum::<f64>() / n as f64,
                avg_cell_count: alive.iter().map(|o| o.cells.len() as f64).sum::<f64>() / n as f64,
                avg_code_count: alive.iter().map(|o| o.code_count() as f64).sum::<f64>() / n as f64,
                avg_freshness: alive.iter().map(|o| o.min_freshness() as f64).sum::<f64>() / n as f64,
                max_generation: alive.iter().map(|o| o.generation).max().unwrap_or(0),
                eat_ratio: self.interval_eat as f64 / total,
                digest_ratio: self.interval_digest as f64 / total,
                refresh_ratio: self.interval_refresh as f64 / total,
                divide_ratio: self.interval_divide as f64 / total,
                total_instructions: self.interval_instructions,
            });
        }

        self.interval_eat = 0;
        self.interval_digest = 0;
        self.interval_refresh = 0;
        self.interval_divide = 0;
        self.interval_instructions = 0;
    }

    /// Execute one instruction for organism at index.
    fn execute_instruction(&mut self, org_idx: usize) -> Option<CellOrganism> {
        let org_count = self.organisms.len();
        let config = self.config.clone();
        let org = &mut self.organisms[org_idx];

        if !org.alive || org.code_count() == 0 { return None; }

        // Get current instruction
        let code_n = org.code_count();
        org.ip %= code_n;
        let instr = match org.nth_code_instruction(org.ip) {
            Some(i) => i,
            None => return None,
        };

        // Deduct instruction cost
        if !org.deduct_energy(config.instruction_cost) {
            org.alive = false;
            return None;
        }
        org.total_instructions += 1;
        self.interval_instructions += 1;

        let mut new_organism: Option<CellOrganism> = None;

        match instr {
            Instruction::Nop => { org.ip += 1; }

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
                org.registers[0] = if a > b { 1 } else { 0 };
                org.ip += 1;
            }

            Instruction::Jmp(offset) => {
                let new_ip = org.ip as i64 + offset as i64;
                let len = code_n as i64;
                org.ip = ((new_ip % len + len) % len) as usize;
            }

            Instruction::Jnz(offset) => {
                if org.registers[0] != 0 {
                    let new_ip = org.ip as i64 + offset as i64;
                    let len = code_n as i64;
                    org.ip = ((new_ip % len + len) % len) as usize;
                } else {
                    org.ip += 1;
                }
            }

            Instruction::SenseSelf(r) => {
                let r = (r as usize) % 8;
                org.registers[r] = org.total_energy();
                org.ip += 1;
            }

            Instruction::Eat => {
                org.eat_count += 1;
                self.interval_eat += 1;
                // Food pool -> first available Stomach cell
                let cem = config.cell_energy_max;
                if self.food_pool > 0 {
                    if let Some(stomach) = org.first_empty_stomach(cem) {
                        if let CellType::Stomach(ref mut s) = stomach.content {
                            let space = cem - *s;
                            let take = (self.food_pool as u8).min(space);
                            *s += take;
                            self.food_pool -= take as i32;
                        }
                    }
                }
                org.ip += 1;
            }

            // LOAD(r, 0) = DIGEST: Stomach -> Energy cell
            // LOAD(r, 1+) = read from first Data cell into register r
            Instruction::Load(r, idx) => {
                if idx > 0 {
                    // Read from Data cell
                    let r = (r as usize) % 8;
                    if let Some(data_cell) = org.cells.iter().find(|c| c.is_data()) {
                        if let CellType::Data(val) = data_cell.content {
                            org.registers[r] = val as i32;
                        }
                    }
                    org.ip += 1;
                } else {
                // DIGEST mode
                org.digest_count += 1;
                self.interval_digest += 1;
                let cem = config.cell_energy_max;

                // Extract food from first non-empty stomach
                let stomach_val = {
                    let mut val = 0u8;
                    for cell in org.cells.iter_mut() {
                        if let CellType::Stomach(ref mut s) = cell.content {
                            if *s > 0 {
                                val = *s;
                                *s = 0;
                                break;
                            }
                        }
                    }
                    val
                };

                // Deposit into first available energy cell
                if stomach_val > 0 {
                    let mut remaining = stomach_val;
                    for cell in org.cells.iter_mut() {
                        if remaining == 0 { break; }
                        if let CellType::Energy(ref mut e) = cell.content {
                            let space = cem.saturating_sub(*e);
                            let deposit = remaining.min(space);
                            *e += deposit;
                            remaining -= deposit;
                        }
                    }
                }
                org.ip += 1;
                } // end else (DIGEST mode)
            }

            Instruction::Refresh => {
                org.refresh_count += 1;
                self.interval_refresh += 1;

                // Refresh cells around current IP position (radius R)
                // First, find the actual cell index of the current Code cell
                let mut code_idx = 0usize;
                let mut cell_pos = None;
                for (i, c) in org.cells.iter().enumerate() {
                    if c.is_code() {
                        if code_idx == org.ip {
                            cell_pos = Some(i);
                            break;
                        }
                        code_idx += 1;
                    }
                }

                if let Some(center) = cell_pos {
                    let r = config.refresh_radius;
                    let start = center.saturating_sub(r);
                    let end = (center + r + 1).min(org.cells.len());
                    for i in start..end {
                        org.cells[i].freshness = config.freshness_max;
                    }
                }
                org.ip += 1;
            }

            Instruction::Divide => {
                org.divide_count += 1;
                self.interval_divide += 1;

                let enough_energy = org.total_energy() >= (config.divide_cost as i32) * 2 + 10;
                if enough_energy && org_count < config.max_organisms {
                    // Clone cells, mutate code cells
                    let mut child_cells = org.cells.clone();
                    let mutation_rate = config.mutation_rate;
                    for cell in child_cells.iter_mut() {
                        if let CellType::Code(ref mut instr) = cell.content {
                            if self.rng.gen_bool(mutation_rate.min(1.0)) {
                                *instr = instr.mutate(&mut self.rng);
                            }
                        }
                        cell.freshness = config.freshness_max; // fresh child
                    }

                    // Split energy: halve each energy cell
                    for (parent_cell, child_cell) in org.cells.iter_mut().zip(child_cells.iter_mut()) {
                        if let (CellType::Energy(ref mut pe), CellType::Energy(ref mut ce)) =
                            (&mut parent_cell.content, &mut child_cell.content) {
                            let half = *pe / 2;
                            *ce = half;
                            *pe -= half;
                        }
                        if let (CellType::Stomach(ref mut ps), CellType::Stomach(ref mut cs)) =
                            (&mut parent_cell.content, &mut child_cell.content) {
                            *cs = 0;
                            let _ = ps; // parent keeps stomach contents
                        }
                    }

                    // Deduct divide cost from parent
                    org.deduct_energy(config.divide_cost);

                    let mut child = CellOrganism::new(child_cells);
                    child.generation = org.generation + 1;
                    new_organism = Some(child);
                }
                org.ip += 1;
            }

            // STORE(r, _): write register r value to first Data cell
            Instruction::Store(r, _) => {
                let r = (r as usize) % 8;
                let val = (org.registers[r] & 0xFF) as u8;
                if let Some(data_cell) = org.cells.iter_mut().find(|c| c.is_data()) {
                    data_cell.content = CellType::Data(val);
                }
                org.ip += 1;
            }

            // EMIT/SAMPLE: not implemented in cell mode
            Instruction::Emit(_) | Instruction::Sample(_) => {
                org.ip += 1;
            }
        }

        new_organism
    }

    pub fn tick(&mut self) {
        self.food_pool += self.config.food_per_tick;

        let n = self.organisms.len();
        let mut new_organisms: Vec<CellOrganism> = Vec::new();

        for i in 0..n {
            if !self.organisms[i].alive { continue; }

            // Per-cell freshness decay
            if self.config.freshness_decay {
                for cell in self.organisms[i].cells.iter_mut() {
                    cell.freshness = cell.freshness.saturating_sub(1);
                }
                // Remove dead cells (freshness = 0)
                let had_code = self.organisms[i].code_count() > 0;
                self.organisms[i].cells.retain(|c| c.freshness > 0);
                // If all code cells are gone, organism dies
                if had_code && self.organisms[i].code_count() == 0 {
                    self.organisms[i].alive = false;
                    let recycled = self.organisms[i].total_energy() / 2;
                    self.food_pool += recycled;
                    continue;
                }
            }

            // Check death: no energy cells or no code cells
            if self.organisms[i].total_energy() <= 0 && self.organisms[i].cells.iter().all(|c| !c.is_energy() || matches!(c.content, CellType::Energy(0))) {
                // Check if there's truly zero energy
                if self.organisms[i].total_energy() <= 0 {
                    self.organisms[i].alive = false;
                    continue;
                }
            }

            if let Some(child) = self.execute_instruction(i) {
                new_organisms.push(child);
            }

            self.organisms[i].age += 1;
        }

        for child in new_organisms {
            self.add_organism(child);
        }

        if self.tick % 100 == 0 {
            self.organisms.retain(|o| o.alive);
        }

        self.tick += 1;

        if self.tick % self.config.snapshot_interval == 0 {
            self.take_snapshot();
        }
    }

    pub fn run(&mut self) {
        let total = self.config.total_ticks;
        self.take_snapshot();

        for t in 0..total {
            self.tick();

            if t % (total / 10) == 0 && t > 0 {
                let alive = self.organisms.iter().filter(|o| o.alive).count();
                eprintln!(
                    "  tick {}/{} ({:.0}%) — {} alive, food_pool={}",
                    t, total, (t as f64 / total as f64) * 100.0, alive, self.food_pool
                );
            }

            if self.organisms.iter().all(|o| !o.alive) && self.tick > 1000 {
                eprintln!("  All organisms dead at tick {}. Ending early.", self.tick);
                self.take_snapshot();
                break;
            }
        }

        if self.tick % self.config.snapshot_interval != 0 {
            self.take_snapshot();
        }
    }

    pub fn export_csv(&self, path: &str) {
        let mut file = fs::File::create(path).expect("Failed to create CSV file");
        writeln!(file, "{}", CellSnapshot::header()).unwrap();
        for s in &self.snapshots {
            writeln!(file, "{}", s.to_csv()).unwrap();
        }
        eprintln!("  Exported {} snapshots to {}", self.snapshots.len(), path);
    }
}

// ============================================================================
// Seeds v3
// ============================================================================

pub fn cell_seed_a(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        Cell::code(Instruction::Eat, f),
        Cell::code(Instruction::Load(0, 0), f),  // DIGEST (repurposed LOAD)
        Cell::code(Instruction::Refresh, f),
        Cell::code(Instruction::Jmp(-3), f),
        Cell::stomach(0, f),
        Cell::energy(cem, f),  // Start with full energy cell
        Cell::energy(cem, f),  // Two energy cells for buffer
    ];
    CellOrganism::new(cells)
}

pub fn cell_seed_b(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        Cell::code(Instruction::Eat, f),
        Cell::code(Instruction::Load(0, 0), f),  // DIGEST
        Cell::code(Instruction::Refresh, f),      // REFRESH early to cover more cells
        Cell::code(Instruction::SenseSelf(1), f),
        Cell::code(Instruction::Cmp(1, 5), f),
        Cell::code(Instruction::Jnz(2), f),
        Cell::code(Instruction::Jmp(-6), f),
        Cell::code(Instruction::Divide, f),
        Cell::code(Instruction::Jmp(-8), f),
        Cell::stomach(0, f),
        Cell::stomach(0, f),  // Two stomachs for faster eating
        Cell::energy(cem, f),
        Cell::energy(cem, f),
        Cell::energy(cem / 2, f),
    ];
    let mut org = CellOrganism::new(cells);
    org.registers[5] = (cem as i32) * 2; // divide when total energy > 2 full cells
    org
}

/// Seed D: Self-sustaining + Data cell for experience storage.
///
/// Strategy: EAT, DIGEST, REFRESH, then SENSE_SELF into r1, STORE r1 to Data cell
/// (recording energy level). Next loop: LOAD from Data cell into r2 (previous energy),
/// SENSE_SELF into r1 (current energy), CMP r1 r2 (is energy increasing?).
/// If energy increasing (r0=1), try DIVIDE. If not, just loop.
/// This creates a primitive "experience": the organism compares current state to
/// a remembered past state to make decisions.
pub fn cell_seed_d(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        Cell::code(Instruction::Eat, f),             // 0: eat
        Cell::code(Instruction::Load(0, 0), f),      // 1: DIGEST (load idx=0)
        Cell::code(Instruction::Refresh, f),          // 2: refresh
        Cell::code(Instruction::Load(2, 1), f),      // 3: r2 = Data cell (previous energy)
        Cell::code(Instruction::SenseSelf(1), f),     // 4: r1 = current energy
        Cell::code(Instruction::Store(1, 0), f),     // 5: store current energy to Data cell
        Cell::code(Instruction::Cmp(1, 2), f),       // 6: r0 = (current > previous)?
        Cell::code(Instruction::Jnz(2), f),          // 7: if improving, skip to DIVIDE
        Cell::code(Instruction::Jmp(-8), f),          // 8: loop back
        Cell::code(Instruction::Divide, f),           // 9: divide (energy increasing!)
        Cell::code(Instruction::Jmp(-10), f),         // 10: loop back
        Cell::stomach(0, f),
        Cell::stomach(0, f),
        Cell::energy(cem, f),
        Cell::energy(cem, f),
        Cell::energy(cem / 2, f),
        Cell::data(0, f),  // Data cell: stores previous energy reading
    ];
    CellOrganism::new(cells)
}

// ============================================================================
// Experiment runner
// ============================================================================

pub struct CellSteadyState {
    pub survived: bool,
    pub avg_population: f64,
    pub avg_energy: f64,
    pub eat_ratio: f64,
    pub digest_ratio: f64,
    pub refresh_ratio: f64,
    pub divide_ratio: f64,
}

pub fn cell_compute_steady_state(snapshots: &[CellSnapshot]) -> CellSteadyState {
    let max_tick = snapshots.iter().map(|s| s.tick).max().unwrap_or(100_000);
    let half = max_tick / 2;
    let second_half: Vec<&CellSnapshot> = snapshots
        .iter()
        .filter(|s| s.tick > half && s.population > 0)
        .collect();

    if second_half.is_empty() {
        return CellSteadyState {
            survived: snapshots.last().map(|s| s.population > 0).unwrap_or(false),
            avg_population: 0.0, avg_energy: 0.0,
            eat_ratio: 0.0, digest_ratio: 0.0, refresh_ratio: 0.0, divide_ratio: 0.0,
        };
    }

    let n = second_half.len() as f64;
    CellSteadyState {
        survived: true,
        avg_population: second_half.iter().map(|s| s.population as f64).sum::<f64>() / n,
        avg_energy: second_half.iter().map(|s| s.avg_energy).sum::<f64>() / n,
        eat_ratio: second_half.iter().map(|s| s.eat_ratio).sum::<f64>() / n,
        digest_ratio: second_half.iter().map(|s| s.digest_ratio).sum::<f64>() / n,
        refresh_ratio: second_half.iter().map(|s| s.refresh_ratio).sum::<f64>() / n,
        divide_ratio: second_half.iter().map(|s| s.divide_ratio).sum::<f64>() / n,
    }
}

pub fn run_cell_experiment(name: &str, config: CellConfig, seed: u64) -> Vec<CellSnapshot> {
    eprintln!("\n========================================");
    eprintln!("Running CELL experiment: {}", name);
    eprintln!("  freshness_decay={}, cell_energy_max={}, refresh_radius={}",
        config.freshness_decay, config.cell_energy_max, config.refresh_radius);
    eprintln!("  total_ticks={}", config.total_ticks);
    eprintln!("========================================");

    let mut world = CellWorld::new(config.clone(), seed);

    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    world.run();

    let safe_name = name.replace(' ', "_");
    world.export_csv(&format!("D:/project/d0-vm/data/{}.csv", safe_name));

    world.snapshots
}

/// Run cell experiment with Seed D (Data cell) organisms included.
pub fn run_cell_data_experiment(name: &str, config: CellConfig, seed: u64) -> Vec<CellSnapshot> {
    eprintln!("\n========================================");
    eprintln!("Running CELL+DATA experiment: {}", name);
    eprintln!("  freshness_decay={}, cell_energy_max={}, refresh_radius={}",
        config.freshness_decay, config.cell_energy_max, config.refresh_radius);
    eprintln!("========================================");

    let mut world = CellWorld::new(config.clone(), seed);

    for _ in 0..5 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..5 { world.add_organism(cell_seed_b(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_d(&config)); }

    world.run();

    let safe_name = name.replace(' ', "_");
    world.export_csv(&format!("D:/project/d0-vm/data/{}.csv", safe_name));

    world.snapshots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_seed_a_survives_100_ticks() {
        let config = CellConfig::experimental();
        let mut world = CellWorld::new(config.clone(), 42);
        world.add_organism(cell_seed_a(&config));
        for _ in 0..100 { world.tick(); }
        assert!(world.organisms.iter().any(|o| o.alive),
            "Cell Seed A should survive at least 100 ticks");
    }

    #[test]
    fn test_cell_organism_energy() {
        let config = CellConfig::experimental();
        let org = cell_seed_a(&config);
        // Seed A: 4 code + 1 stomach + 2 energy (each = cell_energy_max=20)
        assert_eq!(org.total_energy(), 40); // 20 + 20
        assert_eq!(org.code_count(), 4);
        assert_eq!(org.cells.len(), 7);
    }

    #[test]
    fn test_cell_freshness_decay_kills() {
        let mut config = CellConfig::experimental();
        config.freshness_max = 10;
        let mut world = CellWorld::new(config.clone(), 42);

        // Organism with only NOP — no REFRESH
        let cells = vec![
            Cell::code(Instruction::Nop, 10),
            Cell::energy(100, 10),
        ];
        world.add_organism(CellOrganism::new(cells));

        for _ in 0..20 { world.tick(); }
        assert!(world.organisms.iter().all(|o| !o.alive),
            "Organism without REFRESH should die from per-cell freshness decay");
    }

    #[test]
    fn test_no_decay_survives_without_refresh() {
        let mut config = CellConfig::control();
        config.freshness_max = 10;
        let mut world = CellWorld::new(config.clone(), 42);
        world.food_pool = 100_000;

        let cells = vec![
            Cell::code(Instruction::Eat, 10),
            Cell::code(Instruction::Load(0, 0), 10), // DIGEST
            Cell::code(Instruction::Jmp(-2), 10),
            Cell::stomach(0, 10),
            Cell::energy(50, 10),
        ];
        world.add_organism(CellOrganism::new(cells));

        for _ in 0..100 { world.tick(); }
        assert!(world.organisms.iter().any(|o| o.alive),
            "Without freshness decay, organism should survive without REFRESH");
    }
}
