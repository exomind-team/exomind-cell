//! Organism (Actor) and world configuration.

use crate::instruction::Instruction;

/// An organism in the D0 soup. Its code IS its body.
///
/// Operational closure: the organism must execute REFRESH to maintain freshness
/// and EAT to maintain energy. Failure in either -> irreversible death.
#[derive(Debug, Clone)]
pub struct Organism {
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
    pub registers: [i32; 8],
    pub ip: usize,
    pub energy: i32,
    pub freshness: u8,
    pub alive: bool,
    pub age: u64,
    pub generation: u32,

    // Per-tick instruction execution counters (for statistics)
    pub eat_count: u64,
    pub refresh_count: u64,
    pub divide_count: u64,
    pub total_instructions: u64,
}

impl Organism {
    pub fn new(code: Vec<Instruction>, energy: i32, freshness: u8) -> Self {
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

/// World configuration parameters.
#[derive(Debug, Clone)]
pub struct Config {
    pub max_organisms: usize,
    pub food_per_tick: i32,
    pub freshness_max: u8,
    pub freshness_decay: bool,  // KEY SWITCH: experimental (true) vs control (false)
    pub mutation_rate: f64,
    pub eat_energy: i32,
    pub refresh_cost: i32,
    pub divide_cost: i32,
    pub instruction_cost: i32,
    pub initial_energy: i32,
    pub e_max: i32,             // Energy cap
    pub total_ticks: u64,
    pub snapshot_interval: u64,
    pub genome_dump_interval: u64,
}

impl Config {
    /// Default experimental configuration (freshness_decay = true).
    pub fn experimental() -> Self {
        Config {
            max_organisms: 100,
            food_per_tick: 50,
            freshness_max: 255,
            freshness_decay: true,
            mutation_rate: 0.001,
            eat_energy: 10,
            refresh_cost: 1,
            divide_cost: 30,
            instruction_cost: 1,
            initial_energy: 100,
            e_max: 1000,
            total_ticks: 100_000,
            snapshot_interval: 1000,
            genome_dump_interval: 10_000,
        }
    }

    /// Control group: identical except freshness never decays.
    pub fn control() -> Self {
        let mut c = Self::experimental();
        c.freshness_decay = false;
        c
    }
}

/// Seed A: Minimal self-sustaining loop (EAT -> REFRESH -> JMP).
pub fn seed_a(config: &Config) -> Organism {
    let code = vec![
        Instruction::Eat,
        Instruction::Refresh,
        Instruction::Jmp(-2),
    ];
    Organism::new(code, config.initial_energy, config.freshness_max)
}

/// Seed B: Self-sustaining + conditional division.
pub fn seed_b(config: &Config) -> Organism {
    let code = vec![
        Instruction::Eat,
        Instruction::Refresh,
        Instruction::SenseSelf(1),
        Instruction::Cmp(1, 5),
        Instruction::Jnz(2),
        Instruction::Jmp(-5),
        Instruction::Divide,
        Instruction::Jmp(-7),
    ];
    let mut org = Organism::new(code, config.initial_energy, config.freshness_max);
    org.registers[5] = 80; // divide energy threshold
    org
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organism_new() {
        let org = Organism::new(vec![Instruction::Nop], 100, 255);
        assert!(org.alive);
        assert_eq!(org.energy, 100);
        assert_eq!(org.freshness, 255);
        assert_eq!(org.age, 0);
        assert_eq!(org.generation, 0);
        assert_eq!(org.code.len(), 1);
    }

    #[test]
    fn test_seed_a() {
        let config = Config::experimental();
        let org = seed_a(&config);
        assert_eq!(org.code.len(), 3);
        assert_eq!(org.energy, config.initial_energy);
        assert!(matches!(org.code[0], Instruction::Eat));
        assert!(matches!(org.code[1], Instruction::Refresh));
        assert!(matches!(org.code[2], Instruction::Jmp(-2)));
    }

    #[test]
    fn test_seed_b() {
        let config = Config::experimental();
        let org = seed_b(&config);
        assert_eq!(org.code.len(), 8);
        assert_eq!(org.registers[5], 80);
        assert!(matches!(org.code[6], Instruction::Divide));
    }

    #[test]
    fn test_config_control_only_differs_in_freshness_decay() {
        let exp = Config::experimental();
        let ctrl = Config::control();
        assert!(exp.freshness_decay);
        assert!(!ctrl.freshness_decay);
        assert_eq!(exp.max_organisms, ctrl.max_organisms);
        assert_eq!(exp.e_max, ctrl.e_max);
        assert_eq!(exp.mutation_rate, ctrl.mutation_rate);
    }
}
