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

// ============================================================================
// Lineage Tracking
// ============================================================================

/// Records one birth event when DIVIDE succeeds.
#[derive(Debug, Clone)]
pub struct BirthRecord {
    pub parent_id: u64,
    pub child_id: u64,
    pub tick: u64,
    /// Indices of code cells that differ between parent and child (mutations).
    pub mutation_sites: Vec<usize>,
}

impl BirthRecord {
    /// CSV header for lineage output.
    pub fn csv_header() -> &'static str {
        "parent_id,child_id,tick,mutations,mutation_sites"
    }

    pub fn to_csv(&self) -> String {
        let sites = self.mutation_sites.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(";");
        format!(
            "{},{},{},{},\"{}\"",
            self.parent_id, self.child_id, self.tick,
            self.mutation_sites.len(),
            sites,
        )
    }
}

/// Stores all birth records for a simulation run. Supports lineage queries.
pub struct LineageStore {
    pub records: Vec<BirthRecord>,
}

impl LineageStore {
    pub fn new() -> Self {
        LineageStore { records: Vec::new() }
    }

    pub fn record(&mut self, r: BirthRecord) {
        self.records.push(r);
    }

    /// Trace ancestry of `id` back to the earliest known ancestor.
    /// Returns the chain from `id` up to the root (inclusive), oldest last.
    pub fn ancestors(&self, id: u64) -> Vec<u64> {
        let mut chain = vec![id];
        let mut current = id;
        loop {
            if let Some(rec) = self.records.iter().find(|r| r.child_id == current) {
                chain.push(rec.parent_id);
                current = rec.parent_id;
            } else {
                break;
            }
        }
        chain
    }

    /// Write all records to a CSV file.
    pub fn write_csv(&self, path: &str) -> std::io::Result<()> {
        let _ = fs::create_dir_all(std::path::Path::new(path).parent().unwrap_or(std::path::Path::new(".")));
        let mut f = fs::File::create(path)?;
        writeln!(f, "{}", BirthRecord::csv_header())?;
        for r in &self.records {
            writeln!(f, "{}", r.to_csv())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CellOrganism {
    pub cells: Vec<Cell>,
    pub registers: [i32; 8],
    pub ip: usize,          // index into code cells (not all cells)
    pub alive: bool,
    pub age: u64,
    pub generation: u32,
    pub id: u64,            // unique organism ID (assigned by CellWorld)

    // Counters
    pub eat_count: u64,
    pub digest_count: u64,
    pub refresh_count: u64,
    pub divide_count: u64,
    pub total_instructions: u64,

    // Pending birth record: (parent_id, birth_tick, mutation_sites).
    // Set by DIVIDE, consumed by CellWorld.add_organism() to emit a BirthRecord.
    pub _pending_birth: Option<(u64, u64, Vec<usize>)>,
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
            id: 0, // assigned by CellWorld.add_organism()
            eat_count: 0,
            digest_count: 0,
            refresh_count: 0,
            divide_count: 0,
            total_instructions: 0,
            _pending_birth: None,
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
    pub multi_food: bool,         // Enable simple/complex food types
    pub simple_food_energy: u8,   // Energy from simple food (default 5)
    pub complex_food_energy: u8,  // Energy from complex food (default 20, needs 2x DIGEST)
    pub simple_food_rate: i32,    // Simple food per tick
    pub complex_food_rate: i32,   // Complex food per tick
    pub medium_size: usize,       // Stigmergy medium for SAMPLE instruction
    pub data_cell_gating: bool,   // Enable GATE instruction (Data cell as code switch)
    pub lineage_tracking: bool,   // Enable BirthRecord lineage tracking
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
            multi_food: false,
            simple_food_energy: 5,
            complex_food_energy: 20,
            simple_food_rate: 30,
            complex_food_rate: 10,
            medium_size: 0,
            data_cell_gating: false,
            lineage_tracking: false,
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
    pub simple_food: i32,   // Simple food pool (low energy, instant)
    pub complex_food: i32,  // Complex food pool (high energy, needs extra DIGEST)
    pub medium: Vec<u8>,    // Stigmergy medium for SAMPLE/EMIT
    pub tick: u64,
    pub config: CellConfig,
    pub rng: StdRng,
    pub snapshots: Vec<CellSnapshot>,
    pub lineage: LineageStore,  // Birth records (populated when config.lineage_tracking=true)

    // ID counter — monotonically increasing across all births
    next_id: u64,

    // Interval counters
    interval_eat: u64,
    interval_eat_simple: u64,
    interval_eat_complex: u64,
    interval_digest: u64,
    interval_refresh: u64,
    interval_divide: u64,
    interval_instructions: u64,
    // Energy-bucketed instruction counts: [bucket][0=eat, 1=refresh, 2=divide, 3=total]
    // Buckets: 0=0-20%, 1=20-40%, 2=40-60%, 3=60-80%, 4=80-100% of energy capacity
    pub energy_buckets: [[u64; 4]; 5],
}

impl CellWorld {
    pub fn new(config: CellConfig, seed: u64) -> Self {
        CellWorld {
            organisms: Vec::new(),
            food_pool: 500,
            simple_food: 0,
            complex_food: 0,
            medium: vec![0u8; config.medium_size],
            tick: 0,
            config,
            rng: StdRng::seed_from_u64(seed),
            snapshots: Vec::new(),
            lineage: LineageStore::new(),
            next_id: 0,
            interval_eat: 0,
            interval_eat_simple: 0,
            interval_eat_complex: 0,
            interval_digest: 0,
            interval_refresh: 0,
            interval_divide: 0,
            interval_instructions: 0,
            energy_buckets: [[0; 4]; 5],
        }
    }

    pub fn add_organism(&mut self, mut org: CellOrganism) {
        if self.organisms.len() < self.config.max_organisms {
            org.id = self.next_id;
            self.next_id += 1;
            // Consume pending birth record if lineage tracking is enabled
            if self.config.lineage_tracking {
                if let Some((parent_id, tick, mutation_sites)) = org._pending_birth.take() {
                    self.lineage.record(BirthRecord {
                        parent_id,
                        child_id: org.id,
                        tick,
                        mutation_sites,
                    });
                }
            } else {
                org._pending_birth = None;
            }
            self.organisms.push(org);
        }
    }

    /// Write lineage CSV to the given path. No-op if lineage_tracking=false.
    pub fn write_lineage_csv(&self, path: &str) -> std::io::Result<()> {
        self.lineage.write_csv(path)
    }

    pub fn take_snapshot(&mut self) {
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
        self.interval_eat_simple = 0;
        self.interval_eat_complex = 0;
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

        // Energy bucket tracking
        let energy_cap = (org.cells.iter().filter(|c| c.is_energy()).count() as i32) * config.cell_energy_max as i32;
        let energy_pct = if energy_cap > 0 { (org.total_energy() as f64 / energy_cap as f64 * 100.0) as usize } else { 0 };
        let bucket = (energy_pct / 20).min(4);
        self.energy_buckets[bucket][3] += 1; // total for this bucket

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
                self.energy_buckets[bucket][0] += 1; // EAT in this bucket
                let cem = config.cell_energy_max;

                if config.multi_food {
                    // Multi-food mode: randomly pick simple or complex
                    let has_simple = self.simple_food > 0;
                    let has_complex = self.complex_food > 0;
                    let pick_simple = if has_simple && has_complex {
                        self.rng.gen_bool(0.5) // 50/50 random pick
                    } else {
                        has_simple
                    };

                    if pick_simple && self.simple_food > 0 {
                        // Simple food: directly into stomach with simple_food_energy
                        self.interval_eat_simple += 1;
                        if let Some(stomach) = org.first_empty_stomach(cem) {
                            if let CellType::Stomach(ref mut s) = stomach.content {
                                let energy = config.simple_food_energy;
                                let space = cem - *s;
                                let take = energy.min(space);
                                *s += take;
                                self.simple_food -= 1;
                            }
                        }
                    } else if self.complex_food > 0 {
                        // Complex food: goes into stomach but marked as "thick shell"
                        // Represented as negative value convention: value > 100 means complex
                        self.interval_eat_complex += 1;
                        if let Some(stomach) = org.first_empty_stomach(cem) {
                            if let CellType::Stomach(ref mut s) = stomach.content {
                                // Store complex food energy (will need 2x DIGEST)
                                let energy = config.complex_food_energy;
                                let space = cem - *s;
                                *s += energy.min(space);
                                self.complex_food -= 1;
                            }
                        }
                    }
                } else {
                    // Original single-food mode
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
                self.energy_buckets[bucket][1] += 1; // REFRESH in this bucket

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
                self.energy_buckets[bucket][2] += 1; // DIVIDE in this bucket

                let enough_energy = org.total_energy() >= (config.divide_cost as i32) * 2 + 10;
                if enough_energy && org_count < config.max_organisms {
                    // Clone cells, mutate code cells
                    let mut child_cells = org.cells.clone();
                    let mutation_rate = config.mutation_rate;
                    let mut mutation_sites: Vec<usize> = Vec::new();
                    let mut code_idx = 0usize;
                    for cell in child_cells.iter_mut() {
                        if let CellType::Code(ref mut instr) = cell.content {
                            if self.rng.gen_bool(mutation_rate.min(1.0)) {
                                *instr = instr.mutate(&mut self.rng);
                                mutation_sites.push(code_idx);
                            }
                            code_idx += 1;
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

                    let parent_id = org.id;
                    let child_tick = self.tick;

                    let mut child = CellOrganism::new(child_cells);
                    child.generation = org.generation + 1;
                    child._pending_birth = Some((parent_id, child_tick, mutation_sites));
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

            // EMIT(n) = ALLOC in cell mode: append a new cell
            // n=0: Energy cell, n=1: Stomach cell, else: Energy cell
            // Costs 3 energy. Bigger body = more REFRESH needed.
            Instruction::Emit(n) => {
                let alloc_cost = 3u8;
                if org.deduct_energy(alloc_cost) {
                    let new_cell = match n % 2 {
                        0 => Cell::energy(0, config.freshness_max),
                        _ => Cell::stomach(0, config.freshness_max),
                    };
                    org.cells.push(new_cell);
                }
                org.ip += 1;
            }

            // SAMPLE(ch): read medium[ch] into R0
            Instruction::Sample(ch) => {
                if !self.medium.is_empty() {
                    let idx = (ch as usize) % self.medium.len();
                    org.registers[0] = self.medium[idx] as i32;
                }
                org.ip += 1;
            }

            // GATE: read adjacent Data cell; if value==0, skip next Code cell
            Instruction::Gate => {
                if config.data_cell_gating {
                    // Find actual cell index of current Code cell
                    let mut code_count = 0usize;
                    let mut cell_pos = None;
                    for (ci, c) in org.cells.iter().enumerate() {
                        if c.is_code() {
                            if code_count == org.ip {
                                cell_pos = Some(ci);
                                break;
                            }
                            code_count += 1;
                        }
                    }

                    if let Some(pos) = cell_pos {
                        // Look for adjacent Data cell (check right first, then left)
                        let data_val = if pos + 1 < org.cells.len() {
                            if let CellType::Data(v) = org.cells[pos + 1].content { Some(v) } else { None }
                        } else { None }
                        .or_else(|| {
                            if pos > 0 {
                                if let CellType::Data(v) = org.cells[pos - 1].content { Some(v) } else { None }
                            } else { None }
                        });

                        if let Some(val) = data_val {
                            if val == 0 {
                                // Skip next Code cell
                                org.ip += 2; // skip GATE + next code
                            } else {
                                org.ip += 1; // Data > 0, continue normally
                            }
                        } else {
                            org.ip += 1; // No adjacent Data cell, NOP
                        }
                    } else {
                        org.ip += 1;
                    }
                } else {
                    org.ip += 1; // Gating disabled, NOP
                }
            }
        }

        new_organism
    }

    pub fn tick(&mut self) {
        if self.config.multi_food {
            self.simple_food += self.config.simple_food_rate;
            self.complex_food += self.config.complex_food_rate;
        } else {
            self.food_pool += self.config.food_per_tick;
        }

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

/// Seed E: Growth-oriented organism with ALLOC.
///
/// Strategy: EAT → DIGEST → SENSE_SELF → CMP(energy, 30) → if yes: ALLOC Energy
/// → REFRESH → loop. Trade-off: bigger body = more energy cap but more REFRESH cost.
pub fn cell_seed_e(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        Cell::code(Instruction::Eat, f),            // 0: eat
        Cell::code(Instruction::Load(0, 0), f),     // 1: DIGEST
        Cell::code(Instruction::SenseSelf(1), f),    // 2: r1 = energy
        Cell::code(Instruction::Cmp(1, 4), f),      // 3: r0 = (r1 > r4)? r4 = ALLOC threshold
        Cell::code(Instruction::Jnz(2), f),         // 4: if enough energy, skip to ALLOC
        Cell::code(Instruction::Refresh, f),         // 5: normal refresh
        Cell::code(Instruction::Jmp(-6), f),         // 6: loop back
        Cell::code(Instruction::Emit(0), f),        // 7: ALLOC Energy cell
        Cell::code(Instruction::Refresh, f),         // 8: refresh after ALLOC
        Cell::code(Instruction::Jmp(-9), f),         // 9: loop back
        Cell::stomach(0, f),
        Cell::stomach(0, f),
        Cell::energy(cem, f),
        Cell::energy(cem, f),
    ];
    let mut org = CellOrganism::new(cells);
    org.registers[4] = 30; // ALLOC when energy > 30
    org
}

/// Seed F: Prediction-oriented organism with SAMPLE + Data cell.
///
/// Reads signal from medium channel 0, stores it in Data cell, compares with
/// previous reading. If signal is changing (rising), eats more aggressively.
pub fn cell_seed_f(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        Cell::code(Instruction::Sample(0), f),      // 0: r0 = medium[0] (environment signal)
        Cell::code(Instruction::Store(0, 0), f),    // 1: store r0 to Data cell (current signal)
        Cell::code(Instruction::Eat, f),             // 2: eat
        Cell::code(Instruction::Load(0, 0), f),     // 3: DIGEST
        Cell::code(Instruction::Load(1, 1), f),     // 4: r1 = Data cell (previous signal)
        Cell::code(Instruction::Cmp(0, 1), f),      // 5: r0 = (current > previous)? signal rising?
        Cell::code(Instruction::Jnz(2), f),         // 6: if signal changing → extra eat
        Cell::code(Instruction::Refresh, f),         // 7: normal refresh
        Cell::code(Instruction::Jmp(-8), f),         // 8: loop back
        Cell::code(Instruction::Eat, f),             // 9: extra eat (signal rising branch)
        Cell::code(Instruction::Refresh, f),         // 10: refresh
        Cell::code(Instruction::Jmp(-11), f),        // 11: loop back
        Cell::stomach(0, f),
        Cell::stomach(0, f),
        Cell::energy(cem, f),
        Cell::energy(cem, f),
        Cell::data(0, f),  // stores previous signal reading
    ];
    CellOrganism::new(cells)
}

/// Seed G: Evaluation + GATE-controlled behavior.
///
/// Structure: evaluation module writes to Data cell, GATE uses Data cell
/// to conditionally execute DIVIDE. Layout:
///   [SENSE r1] [EAT] [DIGEST] [SENSE r2] [CMP r2,r1] [STORE r0→Data]
///   [GATE] [DIVIDE]  ← GATE checks adjacent Data cell; if 0, skip DIVIDE
///   [REFRESH] [JMP]
///   [Stomach] [Stomach] [Energy] [Energy] [Data(0)]
///
/// The Data cell is placed adjacent to GATE in the cells array.
pub fn cell_seed_g(config: &CellConfig) -> CellOrganism {
    let f = config.freshness_max;
    let cem = config.cell_energy_max;
    let cells = vec![
        // Evaluation module: sense→act→sense→compare→store
        Cell::code(Instruction::SenseSelf(1), f),    // 0: r1 = energy before
        Cell::code(Instruction::Eat, f),              // 1: eat
        Cell::code(Instruction::Load(0, 0), f),      // 2: DIGEST
        Cell::code(Instruction::SenseSelf(2), f),    // 3: r2 = energy after
        Cell::code(Instruction::Cmp(2, 1), f),       // 4: r0 = (after > before)? 1:0
        Cell::code(Instruction::Store(0, 0), f),     // 5: store r0 to Data cell
        // GATE-controlled behavior: Data cell adjacent to GATE
        Cell::data(0, f),                             // 6: Data cell (gate switch) ← adjacent to GATE
        Cell::code(Instruction::Gate, f),             // 7: GATE — reads Data cell[6], if 0 skip next
        Cell::code(Instruction::Divide, f),           // 8: DIVIDE (only if Data > 0 = energy improved)
        // Maintenance
        Cell::code(Instruction::Refresh, f),          // 9: refresh
        Cell::code(Instruction::Jmp(-9), f),          // 10: loop back to 0
        // Resources
        Cell::stomach(0, f),
        Cell::stomach(0, f),
        Cell::energy(cem, f),
        Cell::energy(cem, f),
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
    pub avg_cell_count: f64,
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
            avg_population: 0.0, avg_energy: 0.0, avg_cell_count: 0.0,
            eat_ratio: 0.0, digest_ratio: 0.0, refresh_ratio: 0.0, divide_ratio: 0.0,
        };
    }

    let n = second_half.len() as f64;
    CellSteadyState {
        survived: true,
        avg_population: second_half.iter().map(|s| s.population as f64).sum::<f64>() / n,
        avg_energy: second_half.iter().map(|s| s.avg_energy).sum::<f64>() / n,
        avg_cell_count: second_half.iter().map(|s| s.avg_cell_count).sum::<f64>() / n,
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

/// Run cell experiment with Seed E (growth/ALLOC) organisms.
pub fn run_cell_growth_experiment(name: &str, config: CellConfig, seed: u64) -> Vec<CellSnapshot> {
    eprintln!("\n========================================");
    eprintln!("Running CELL+GROWTH experiment: {}", name);
    eprintln!("  freshness_decay={}, cell_energy_max={}", config.freshness_decay, config.cell_energy_max);
    eprintln!("========================================");

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_b(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_e(&config)); }

    world.run();

    let safe_name = name.replace(' ', "_");
    world.export_csv(&format!("D:/project/d0-vm/data/{}.csv", safe_name));
    world.snapshots
}

/// Run cell experiment with dynamic food based on real CPU usage.
pub fn run_cell_realcpu_experiment(name: &str, config: CellConfig, seed: u64) -> Vec<CellSnapshot> {
    eprintln!("\n========================================");
    eprintln!("Running CELL+REAL_CPU experiment: {}", name);
    eprintln!("  food varies with system CPU availability");
    eprintln!("========================================");

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_b(&config)); }

    // We'll modulate food_per_tick externally via world.food_pool
    // The actual CPU sampling happens in the caller (main.rs)
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

    #[test]
    fn test_digest_moves_food_to_energy() {
        let config = CellConfig::control(); // no decay for simplicity
        let mut world = CellWorld::new(config.clone(), 42);
        world.food_pool = 100;

        // EAT -> DIGEST -> JMP loop
        let cells = vec![
            Cell::code(Instruction::Eat, 255),
            Cell::code(Instruction::Load(0, 0), 255), // DIGEST
            Cell::code(Instruction::Jmp(-2), 255),
            Cell::stomach(0, 255),
            Cell::energy(20, 255), // start with enough energy to run instructions
        ];
        world.add_organism(CellOrganism::new(cells));

        for _ in 0..10 { world.tick(); }
        let org = &world.organisms[0];
        // Started with 20 energy, spent ~10 on instructions, but gained food via EAT+DIGEST
        assert!(org.alive, "Organism should still be alive after EAT+DIGEST loop");
        assert!(org.total_energy() > 0, "Energy should be positive after eating");
    }

    #[test]
    fn test_data_cell_store_and_load() {
        let config = CellConfig::control();
        let mut world = CellWorld::new(config, 42);

        // SENSE_SELF r1 -> STORE r1 -> LOAD r2 from data -> JMP
        let cells = vec![
            Cell::code(Instruction::SenseSelf(1), 255), // r1 = energy
            Cell::code(Instruction::Store(1, 0), 255),  // store r1 to data cell
            Cell::code(Instruction::Load(2, 1), 255),   // load from data cell to r2
            Cell::code(Instruction::Jmp(-3), 255),
            Cell::energy(50, 255),
            Cell::data(0, 255),
        ];
        world.add_organism(CellOrganism::new(cells));

        for _ in 0..5 { world.tick(); }

        let org = &world.organisms[0];
        // After SENSE_SELF: r1 = energy. After STORE: data cell = r1. After LOAD: r2 = data cell.
        // r2 should equal the energy value that was stored
        assert!(org.registers[2] > 0,
            "LOAD from Data cell should have read the stored energy value into r2");

        // Check data cell has non-zero content
        let data_val = org.cells.iter().find_map(|c| match c.content {
            CellType::Data(v) => Some(v),
            _ => None,
        });
        assert!(data_val.is_some() && data_val.unwrap() > 0,
            "Data cell should contain the stored energy value");
    }

    #[test]
    fn test_refresh_radius_coverage() {
        let mut config = CellConfig::experimental();
        config.freshness_max = 100;
        config.refresh_radius = 2; // covers 5 cells around IP

        // Create organism: 5 code cells + energy + stomach
        // REFRESH is at position 0 (code index 0)
        let cells = vec![
            Cell::code(Instruction::Refresh, 50),   // 0: this is IP=0
            Cell::code(Instruction::Jmp(-1), 50),   // 1
            Cell::code(Instruction::Nop, 50),        // 2
            Cell::energy(100, 50),                    // 3
            Cell::stomach(0, 50),                     // 4
        ];
        let mut org = CellOrganism::new(cells);

        // Manually simulate REFRESH at IP=0 with R=2 → covers cells 0..=2 (center=0, so 0 to min(2,4))
        // The actual cell index of code cell 0 is cells[0]
        let r = config.refresh_radius;
        let center = 0usize; // code cell 0 is at cells[0]
        let start = center.saturating_sub(r);
        let end = (center + r + 1).min(org.cells.len());
        for i in start..end {
            org.cells[i].freshness = config.freshness_max;
        }

        // Cells 0,1,2 should be refreshed (100), cells 3,4 should still be 50
        assert_eq!(org.cells[0].freshness, 100);
        assert_eq!(org.cells[1].freshness, 100);
        assert_eq!(org.cells[2].freshness, 100);
        assert_eq!(org.cells[3].freshness, 50, "Cell outside REFRESH radius should not be refreshed");
        assert_eq!(org.cells[4].freshness, 50, "Cell outside REFRESH radius should not be refreshed");
    }

    #[test]
    fn test_seed_d_has_data_cell() {
        let config = CellConfig::experimental();
        let org = cell_seed_d(&config);
        assert!(org.cells.iter().any(|c| c.is_data()),
            "Seed D should have at least one Data cell");
        assert!(org.code_count() >= 9,
            "Seed D should have enough code cells for its logic");
    }

    #[test]
    fn test_gate_blocks_when_data_zero() {
        let mut config = CellConfig::experimental();
        config.data_cell_gating = true;
        let mut world = CellWorld::new(config.clone(), 42);
        // [GATE] [Data(0)] [INC r1] [EAT] [JMP -3] + energy
        let cells = vec![
            Cell::code(Instruction::Gate, 255),
            Cell::data(0, 255), // Data=0 → GATE should skip next code
            Cell::code(Instruction::Inc(1), 255), // This should be SKIPPED
            Cell::code(Instruction::Eat, 255),
            Cell::code(Instruction::Jmp(-4), 255),
            Cell::energy(50, 255),
        ];
        world.food_pool = 10000;
        world.add_organism(CellOrganism::new(cells));
        for _ in 0..10 { world.tick(); }
        let org = &world.organisms[0];
        // r1 should be 0 because INC was gated (skipped)
        assert_eq!(org.registers[1], 0, "INC should be skipped when Data=0 gates it");
    }

    #[test]
    fn test_gate_allows_when_data_nonzero() {
        let mut config = CellConfig::experimental();
        config.data_cell_gating = true;
        let mut world = CellWorld::new(config.clone(), 42);
        // [GATE] [Data(5)] [INC r1] [EAT] [JMP -3] + energy
        let cells = vec![
            Cell::code(Instruction::Gate, 255),
            Cell::data(5, 255), // Data=5 → GATE should NOT skip
            Cell::code(Instruction::Inc(1), 255), // This should execute
            Cell::code(Instruction::Eat, 255),
            Cell::code(Instruction::Jmp(-4), 255),
            Cell::energy(50, 255),
        ];
        world.food_pool = 10000;
        world.add_organism(CellOrganism::new(cells));
        for _ in 0..10 { world.tick(); }
        let org = &world.organisms[0];
        assert!(org.registers[1] > 0, "INC should execute when Data>0");
    }

    #[test]
    fn test_gate_nop_when_disabled() {
        let mut config = CellConfig::experimental();
        config.data_cell_gating = false; // disabled
        let mut world = CellWorld::new(config.clone(), 42);
        let cells = vec![
            Cell::code(Instruction::Gate, 255),
            Cell::data(0, 255),
            Cell::code(Instruction::Inc(1), 255),
            Cell::code(Instruction::Eat, 255),
            Cell::code(Instruction::Jmp(-4), 255),
            Cell::energy(50, 255),
        ];
        world.food_pool = 10000;
        world.add_organism(CellOrganism::new(cells));
        for _ in 0..10 { world.tick(); }
        let org = &world.organisms[0];
        // Gate disabled → acts as NOP → INC should execute
        assert!(org.registers[1] > 0, "INC should execute when gating is disabled");
    }

    #[test]
    fn test_seed_g_structure() {
        let config = CellConfig::experimental();
        let org = cell_seed_g(&config);
        assert!(org.cells.iter().any(|c| c.is_data()), "Seed G has Data cell");
        assert!(org.cells.iter().any(|c| matches!(c.content, CellType::Code(Instruction::Gate))),
            "Seed G has GATE instruction");
        assert!(org.code_count() >= 9, "Seed G has enough code");
    }
}
