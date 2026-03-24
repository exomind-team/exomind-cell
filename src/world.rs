//! World simulation engine — the "soup" containing all organisms.

use rand::prelude::*;
use std::fs;
use std::io::Write;

use crate::instruction::Instruction;
use crate::organism::{Config, Organism};

/// A snapshot of world state at a given tick, used for analysis.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tick: u64,
    pub population: usize,
    pub avg_energy: f64,
    pub avg_code_length: f64,
    pub avg_age: f64,
    pub avg_freshness: f64,
    pub total_eat: u64,
    pub total_refresh: u64,
    pub total_divide: u64,
    pub total_instructions: u64,
    pub eat_ratio: f64,
    pub refresh_ratio: f64,
    pub divide_ratio: f64,
    pub low_energy_eat_rate: f64,
    pub low_freshness_refresh_rate: f64,
    pub max_generation: u32,
}

impl Snapshot {
    pub fn header() -> String {
        "tick,population,avg_energy,avg_code_length,avg_age,avg_freshness,\
         total_eat,total_refresh,total_divide,total_instructions,\
         eat_ratio,refresh_ratio,divide_ratio,\
         low_energy_eat_rate,low_freshness_refresh_rate,max_generation"
            .to_string()
    }

    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            self.tick, self.population, self.avg_energy, self.avg_code_length,
            self.avg_age, self.avg_freshness,
            self.total_eat, self.total_refresh, self.total_divide, self.total_instructions,
            self.eat_ratio, self.refresh_ratio, self.divide_ratio,
            self.low_energy_eat_rate, self.low_freshness_refresh_rate, self.max_generation,
        )
    }
}

/// A snapshot of an organism's genome at a point in time.
#[derive(Debug, Clone)]
pub struct GenomeDump {
    pub tick: u64,
    pub label: String,
    pub age: u64,
    pub energy: i32,
    pub freshness: u8,
    pub generation: u32,
    pub code: Vec<Instruction>,
}

/// The world: a soup of organisms competing for food.
pub struct World {
    pub organisms: Vec<Organism>,
    pub food_pool: i32,
    pub tick: u64,
    pub config: Config,
    pub rng: StdRng,
    pub snapshots: Vec<Snapshot>,
    pub genome_dumps: Vec<GenomeDump>,

    // Accumulated counters for snapshot interval
    interval_eat: u64,
    interval_refresh: u64,
    interval_divide: u64,
    interval_instructions: u64,
    low_energy_eats: u64,
    low_energy_total_instructions: u64,
    low_freshness_refreshes: u64,
    low_freshness_total_instructions: u64,
}

impl World {
    pub fn new(config: Config, seed: u64) -> Self {
        World {
            organisms: Vec::new(),
            food_pool: 500,
            tick: 0,
            config,
            rng: StdRng::seed_from_u64(seed),
            snapshots: Vec::new(),
            genome_dumps: Vec::new(),
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

    pub fn add_organism(&mut self, org: Organism) {
        if self.organisms.len() < self.config.max_organisms {
            self.organisms.push(org);
        }
    }

    /// Take a statistics snapshot.
    pub fn take_snapshot(&mut self) {
        let alive: Vec<&Organism> = self.organisms.iter().filter(|o| o.alive).collect();
        let n = alive.len();

        if n == 0 {
            self.snapshots.push(Snapshot {
                tick: self.tick, population: 0, avg_energy: 0.0, avg_code_length: 0.0,
                avg_age: 0.0, avg_freshness: 0.0,
                total_eat: self.interval_eat, total_refresh: self.interval_refresh,
                total_divide: self.interval_divide, total_instructions: self.interval_instructions,
                eat_ratio: 0.0, refresh_ratio: 0.0, divide_ratio: 0.0,
                low_energy_eat_rate: 0.0, low_freshness_refresh_rate: 0.0, max_generation: 0,
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

        self.interval_eat = 0;
        self.interval_refresh = 0;
        self.interval_divide = 0;
        self.interval_instructions = 0;
        self.low_energy_eats = 0;
        self.low_energy_total_instructions = 0;
        self.low_freshness_refreshes = 0;
        self.low_freshness_total_instructions = 0;
    }

    /// Dump genomes of the oldest and most-evolved organisms.
    pub fn dump_genome(&mut self) {
        if let Some(oldest) = self.organisms.iter().filter(|o| o.alive).max_by_key(|o| o.age) {
            self.genome_dumps.push(GenomeDump {
                tick: self.tick, label: "oldest".to_string(),
                age: oldest.age, energy: oldest.energy, freshness: oldest.freshness,
                generation: oldest.generation, code: oldest.code.clone(),
            });
        }
        if let Some(most_evolved) = self.organisms.iter().filter(|o| o.alive).max_by_key(|o| o.generation) {
            self.genome_dumps.push(GenomeDump {
                tick: self.tick, label: "most_evolved".to_string(),
                age: most_evolved.age, energy: most_evolved.energy, freshness: most_evolved.freshness,
                generation: most_evolved.generation, code: most_evolved.code.clone(),
            });
        }
    }

    /// Execute one instruction for a given organism. Returns optional child.
    fn execute_instruction(&mut self, org_idx: usize) -> Option<Organism> {
        let org_count = self.organisms.len();
        let org = &mut self.organisms[org_idx];

        if !org.alive || org.code.is_empty() {
            return None;
        }

        org.ip %= org.code.len();
        let instr = org.code[org.ip];

        let low_energy = org.energy < (self.config.initial_energy / 5);
        let low_freshness = org.freshness < 50;

        if low_energy { self.low_energy_total_instructions += 1; }
        if low_freshness { self.low_freshness_total_instructions += 1; }

        org.energy -= self.config.instruction_cost;
        org.total_instructions += 1;
        self.interval_instructions += 1;

        let mut new_organism: Option<Organism> = None;

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
                if low_energy { self.low_energy_eats += 1; }
                let can_absorb = (self.config.e_max - org.energy).max(0);
                if can_absorb > 0 {
                    let want = self.config.eat_energy.min(can_absorb);
                    if self.food_pool >= want {
                        self.food_pool -= want;
                        org.energy += want;
                    } else if self.food_pool > 0 {
                        let take = self.food_pool.min(can_absorb);
                        org.energy += take;
                        self.food_pool -= take;
                    }
                }
                org.energy = org.energy.min(self.config.e_max);
                org.ip += 1;
            }

            Instruction::Refresh => {
                org.refresh_count += 1;
                self.interval_refresh += 1;
                if low_freshness { self.low_freshness_refreshes += 1; }
                org.energy -= self.config.refresh_cost;
                org.freshness = self.config.freshness_max;
                org.ip += 1;
            }

            Instruction::Divide => {
                org.divide_count += 1;
                self.interval_divide += 1;

                if org.energy >= self.config.divide_cost * 2
                    && org_count < self.config.max_organisms
                {
                    let mut child_code = org.code.clone();
                    let child_energy = org.energy / 2;
                    let child_generation = org.generation + 1;
                    org.energy -= child_energy;
                    org.energy -= self.config.divide_cost;

                    let mutation_rate = self.config.mutation_rate;
                    for instr in child_code.iter_mut() {
                        if self.rng.gen_bool(mutation_rate.min(1.0)) {
                            *instr = instr.mutate(&mut self.rng);
                        }
                    }

                    let mut child = Organism::new(
                        child_code, child_energy, self.config.freshness_max,
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
    pub fn tick(&mut self) {
        self.food_pool += self.config.food_per_tick;

        let n = self.organisms.len();
        let mut new_organisms: Vec<Organism> = Vec::new();

        for i in 0..n {
            if !self.organisms[i].alive { continue; }

            if self.config.freshness_decay {
                self.organisms[i].freshness = self.organisms[i].freshness.saturating_sub(1);
            }

            if self.organisms[i].freshness == 0 || self.organisms[i].energy <= 0 {
                self.organisms[i].alive = false;
                let recycled = self.organisms[i].energy.max(0) / 2;
                self.food_pool += recycled;
                continue;
            }

            if let Some(child) = self.execute_instruction(i) {
                new_organisms.push(child);
            }

            self.organisms[i].age += 1;

            if self.organisms[i].energy <= 0 {
                self.organisms[i].alive = false;
            }
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

        if self.config.genome_dump_interval > 0 && self.tick % self.config.genome_dump_interval == 0 {
            self.dump_genome();
        }
    }

    /// Run the full simulation.
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
        writeln!(file, "{}", Snapshot::header()).unwrap();
        for s in &self.snapshots {
            writeln!(file, "{}", s.to_csv()).unwrap();
        }
        eprintln!("  Exported {} snapshots to {}", self.snapshots.len(), path);
    }

    pub fn export_genomes(&self, path: &str) {
        let mut file = fs::File::create(path).expect("Failed to create genome dump file");
        for dump in &self.genome_dumps {
            writeln!(file, "=== [{}] Tick {} | Age {} | Energy {} | Freshness {} | Gen {} | Code len {} ===",
                dump.label, dump.tick, dump.age, dump.energy, dump.freshness, dump.generation, dump.code.len()
            ).unwrap();
            for (i, instr) in dump.code.iter().enumerate() {
                writeln!(file, "  [{:3}] {}", i, instr).unwrap();
            }
            writeln!(file).unwrap();
        }
        eprintln!("  Exported {} genome dumps to {}", self.genome_dumps.len(), path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organism::{seed_a, seed_b};

    #[test]
    fn test_seed_a_survives_100_ticks() {
        let config = Config::experimental();
        let mut world = World::new(config.clone(), 42);
        world.add_organism(seed_a(&config));
        for _ in 0..100 {
            world.tick();
        }
        assert!(world.organisms.iter().any(|o| o.alive),
            "Seed A should survive at least 100 ticks");
    }

    #[test]
    fn test_seed_b_survives_100_ticks() {
        let config = Config::experimental();
        let mut world = World::new(config.clone(), 42);
        world.add_organism(seed_b(&config));
        for _ in 0..100 {
            world.tick();
        }
        assert!(world.organisms.iter().any(|o| o.alive),
            "Seed B should survive at least 100 ticks");
    }

    #[test]
    fn test_energy_cap() {
        let config = Config::experimental();
        let mut world = World::new(config.clone(), 42);
        world.food_pool = 100_000;
        world.add_organism(seed_a(&config));
        for _ in 0..500 {
            world.tick();
        }
        for org in &world.organisms {
            if org.alive {
                assert!(org.energy <= config.e_max,
                    "Energy {} should not exceed E_MAX {}", org.energy, config.e_max);
            }
        }
    }

    #[test]
    fn test_freshness_decay_kills() {
        let mut config = Config::experimental();
        config.freshness_max = 10; // Very short freshness
        let mut world = World::new(config.clone(), 42);

        // An organism that only does NOP (no REFRESH)
        let org = Organism::new(vec![Instruction::Nop], 1000, 10);
        world.add_organism(org);

        for _ in 0..20 {
            world.tick();
        }
        assert!(world.organisms.iter().all(|o| !o.alive),
            "Organism without REFRESH should die from freshness decay");
    }

    #[test]
    fn test_no_freshness_decay_survives() {
        let mut config = Config::control(); // freshness_decay = false
        config.freshness_max = 10;
        let mut world = World::new(config.clone(), 42);

        // An organism that only does EAT (no REFRESH, but decay is off)
        let org = Organism::new(vec![Instruction::Eat], 100, 10);
        world.add_organism(org);
        world.food_pool = 100_000;

        for _ in 0..100 {
            world.tick();
        }
        assert!(world.organisms.iter().any(|o| o.alive),
            "Without freshness decay, organism should survive even without REFRESH");
    }

    #[test]
    fn test_divide_creates_child() {
        let config = Config::experimental();
        let mut world = World::new(config.clone(), 42);

        let mut org = Organism::new(vec![Instruction::Divide, Instruction::Eat, Instruction::Refresh, Instruction::Jmp(-3)], 200, 255);
        org.energy = 200; // Enough to divide (need >= divide_cost * 2 = 60)
        world.add_organism(org);
        world.food_pool = 10000;

        let initial_count = world.organisms.len();
        world.tick();

        assert!(world.organisms.len() > initial_count,
            "DIVIDE should create a child organism");
    }
}
