//! Single-site knockout analysis: replace each Code cell with NOP,
//! test survival in sandbox, classify the effect.

use crate::cell_vm::{Cell, CellConfig, CellOrganism, CellType, CellWorld};
use crate::instruction::Instruction;

/// Result of knocking out one code position.
#[derive(Debug)]
pub struct KnockoutResult {
    pub position: usize,         // index into code cells (not all cells)
    pub cell_index: usize,       // index into cells array
    pub original: Instruction,
    pub category: KnockoutCategory,
    pub survival_ticks: u64,     // how long the organism survived (max = test_ticks)
    pub final_energy: i32,       // energy at end (or at death)
    pub final_pop: usize,        // population at end
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KnockoutCategory {
    Lethal,          // population goes to 0
    SevereDefect,    // survives but pop < 25% of wildtype
    MildDefect,      // survives, pop 25-75% of wildtype
    Neutral,         // survives, pop 75-125% of wildtype
    Beneficial,      // survives, pop > 125% of wildtype
}

impl KnockoutCategory {
    pub fn label(&self) -> &str {
        match self {
            KnockoutCategory::Lethal => "lethal",
            KnockoutCategory::SevereDefect => "severe_defect",
            KnockoutCategory::MildDefect => "mild_defect",
            KnockoutCategory::Neutral => "neutral",
            KnockoutCategory::Beneficial => "beneficial",
        }
    }
}

/// Run wildtype (unmodified) organism to get baseline.
fn run_wildtype(organism: &CellOrganism, config: &CellConfig, seed: u64, test_ticks: u64) -> (u64, i32, usize) {
    let mut world = CellWorld::new(config.clone(), seed);
    world.add_organism(organism.clone());
    world.food_pool = 100_000;

    let mut survived_until = test_ticks;
    for t in 0..test_ticks {
        world.tick();
        if world.organisms.iter().all(|o| !o.alive) {
            survived_until = t;
            break;
        }
    }

    let alive_count = world.organisms.iter().filter(|o| o.alive).count();
    let total_energy: i32 = world.organisms.iter()
        .filter(|o| o.alive)
        .map(|o| o.total_energy())
        .sum();

    (survived_until, total_energy, alive_count)
}

/// Run knockout: replace code cell at position `code_idx` with NOP.
fn run_knockout(organism: &CellOrganism, code_idx: usize, config: &CellConfig, seed: u64, test_ticks: u64) -> (u64, i32, usize) {
    let mut mutant = organism.clone();

    // Find the actual cell index of the nth Code cell
    let mut count = 0;
    for (i, cell) in mutant.cells.iter_mut().enumerate() {
        if cell.is_code() {
            if count == code_idx {
                cell.content = CellType::Code(Instruction::Nop);
                break;
            }
            count += 1;
        }
    }

    let mut world = CellWorld::new(config.clone(), seed);
    world.add_organism(mutant);
    world.food_pool = 100_000;

    let mut survived_until = test_ticks;
    for t in 0..test_ticks {
        world.tick();
        if world.organisms.iter().all(|o| !o.alive) {
            survived_until = t;
            break;
        }
    }

    let alive_count = world.organisms.iter().filter(|o| o.alive).count();
    let total_energy: i32 = world.organisms.iter()
        .filter(|o| o.alive)
        .map(|o| o.total_energy())
        .sum();

    (survived_until, total_energy, alive_count)
}

/// Run full knockout analysis on an organism.
pub fn analyze_knockout(organism: &CellOrganism, config: &CellConfig, seed: u64, test_ticks: u64) -> Vec<KnockoutResult> {
    // First run wildtype
    let (wt_ticks, wt_energy, wt_pop) = run_wildtype(organism, config, seed, test_ticks);

    let code_count = organism.code_count();
    let mut results = Vec::new();

    for code_idx in 0..code_count {
        // Find original instruction
        let original = organism.cells.iter()
            .filter(|c| c.is_code())
            .nth(code_idx)
            .and_then(|c| match c.content {
                CellType::Code(instr) => Some(instr),
                _ => None,
            })
            .unwrap_or(Instruction::Nop);

        // Find cell array index
        let cell_index = organism.cells.iter().enumerate()
            .filter(|(_, c)| c.is_code())
            .nth(code_idx)
            .map(|(i, _)| i)
            .unwrap_or(0);

        let (ko_ticks, ko_energy, ko_pop) = run_knockout(organism, code_idx, config, seed, test_ticks);

        // Classify
        let category = if ko_pop == 0 || ko_ticks < test_ticks / 2 {
            KnockoutCategory::Lethal
        } else if wt_pop == 0 {
            // Wildtype also dead — can't compare
            if ko_pop > 0 { KnockoutCategory::Beneficial } else { KnockoutCategory::Neutral }
        } else {
            let ratio = ko_pop as f64 / wt_pop.max(1) as f64;
            if ratio < 0.25 {
                KnockoutCategory::SevereDefect
            } else if ratio < 0.75 {
                KnockoutCategory::MildDefect
            } else if ratio > 1.25 {
                KnockoutCategory::Beneficial
            } else {
                KnockoutCategory::Neutral
            }
        };

        results.push(KnockoutResult {
            position: code_idx,
            cell_index,
            original,
            category,
            survival_ticks: ko_ticks,
            final_energy: ko_energy,
            final_pop: ko_pop,
        });
    }

    results
}

/// Format knockout results as CSV string.
pub fn results_to_csv(results: &[KnockoutResult]) -> String {
    let mut csv = String::from("position,cell_index,original_instruction,knockout_result,survival_ticks,energy_level,population\n");
    for r in results {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            r.position, r.cell_index, r.original, r.category.label(),
            r.survival_ticks, r.final_energy, r.final_pop,
        ));
    }
    csv
}

/// Summary statistics from knockout analysis.
pub fn summarize(results: &[KnockoutResult]) -> String {
    let total = results.len();
    let lethal = results.iter().filter(|r| r.category == KnockoutCategory::Lethal).count();
    let severe = results.iter().filter(|r| r.category == KnockoutCategory::SevereDefect).count();
    let mild = results.iter().filter(|r| r.category == KnockoutCategory::MildDefect).count();
    let neutral = results.iter().filter(|r| r.category == KnockoutCategory::Neutral).count();
    let beneficial = results.iter().filter(|r| r.category == KnockoutCategory::Beneficial).count();

    let essential = lethal + severe; // "essential" = lethal + severe defect

    format!(
        "Knockout Summary ({} code cells):\n\
         - Lethal: {} ({:.0}%)\n\
         - Severe defect: {} ({:.0}%)\n\
         - Mild defect: {} ({:.0}%)\n\
         - Neutral: {} ({:.0}%)\n\
         - Beneficial: {} ({:.0}%)\n\
         - Essential (lethal+severe): {} ({:.0}%)\n\
         - Minimum instruction set: {} of {} code cells are essential\n",
        total,
        lethal, lethal as f64 / total as f64 * 100.0,
        severe, severe as f64 / total as f64 * 100.0,
        mild, mild as f64 / total as f64 * 100.0,
        neutral, neutral as f64 / total as f64 * 100.0,
        beneficial, beneficial as f64 / total as f64 * 100.0,
        essential, essential as f64 / total as f64 * 100.0,
        essential, total,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_vm::{cell_seed_a, CellConfig};

    #[test]
    fn test_knockout_seed_a() {
        let config = CellConfig::experimental();
        let org = cell_seed_a(&config);
        let results = analyze_knockout(&org, &config, 42, 1000);

        assert_eq!(results.len(), org.code_count());
        // Seed A has EAT, DIGEST, REFRESH, JMP — at least EAT and REFRESH should be essential
        let lethal_count = results.iter().filter(|r| r.category == KnockoutCategory::Lethal).count();
        assert!(lethal_count >= 1, "At least one instruction should be lethal when knocked out");
    }
}
