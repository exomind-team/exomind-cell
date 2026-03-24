//! Experiment runner and report generation.

use crate::organism::{seed_a, seed_b, seed_c, Config};
use crate::world::{Snapshot, World};

/// Steady-state averages from the second half of a simulation run.
pub struct SteadyState {
    pub eat_ratio: f64,
    pub refresh_ratio: f64,
    pub divide_ratio: f64,
    pub low_energy_eat_rate: f64,
    pub avg_population: f64,
    pub avg_energy: f64,
    pub survived: bool,
}

pub fn compute_steady_state(snapshots: &[Snapshot]) -> SteadyState {
    let second_half: Vec<&Snapshot> = snapshots
        .iter()
        .filter(|s| s.tick > 50000 && s.population > 0)
        .collect();

    if second_half.is_empty() {
        return SteadyState {
            eat_ratio: 0.0, refresh_ratio: 0.0, divide_ratio: 0.0,
            low_energy_eat_rate: 0.0, avg_population: 0.0, avg_energy: 0.0,
            survived: snapshots.last().map(|s| s.population > 0).unwrap_or(false),
        };
    }

    let n = second_half.len() as f64;
    SteadyState {
        eat_ratio: second_half.iter().map(|s| s.eat_ratio).sum::<f64>() / n,
        refresh_ratio: second_half.iter().map(|s| s.refresh_ratio).sum::<f64>() / n,
        divide_ratio: second_half.iter().map(|s| s.divide_ratio).sum::<f64>() / n,
        low_energy_eat_rate: second_half.iter().map(|s| s.low_energy_eat_rate).sum::<f64>() / n,
        avg_population: second_half.iter().map(|s| s.population as f64).sum::<f64>() / n,
        avg_energy: second_half.iter().map(|s| s.avg_energy).sum::<f64>() / n,
        survived: true,
    }
}

/// Run one experiment with a given config and seed.
pub fn run_experiment(name: &str, config: Config, seed: u64) -> Vec<Snapshot> {
    eprintln!("\n========================================");
    eprintln!("Running experiment: {}", name);
    eprintln!("  freshness_decay = {}", config.freshness_decay);
    eprintln!("  max_organisms = {}", config.max_organisms);
    eprintln!("  food_per_tick = {}", config.food_per_tick);
    eprintln!("  mutation_rate = {}", config.mutation_rate);
    eprintln!("  total_ticks = {}", config.total_ticks);
    eprintln!("========================================");

    let mut world = World::new(config.clone(), seed);

    for _ in 0..10 { world.add_organism(seed_a(&config)); }
    for _ in 0..10 { world.add_organism(seed_b(&config)); }

    world.run();

    let safe_name = name.replace(' ', "_");
    let csv_path = format!("D:/project/d0-vm/{}.csv", safe_name);
    world.export_csv(&csv_path);

    if !world.genome_dumps.is_empty() {
        let genome_path = format!("D:/project/d0-vm/{}_genomes.txt", safe_name);
        world.export_genomes(&genome_path);
    }

    world.snapshots
}

/// Run a stigmergy experiment with seed A + B + C organisms.
pub fn run_stigmergy_experiment(name: &str, config: Config, seed: u64) -> Vec<Snapshot> {
    eprintln!("\n========================================");
    eprintln!("Running stigmergy experiment: {}", name);
    eprintln!("  medium_size = {}", config.medium_size);
    eprintln!("  freshness_decay = {}", config.freshness_decay);
    eprintln!("  total_ticks = {}", config.total_ticks);
    eprintln!("========================================");

    let mut world = World::new(config.clone(), seed);

    for _ in 0..5 { world.add_organism(seed_a(&config)); }
    for _ in 0..5 { world.add_organism(seed_b(&config)); }
    for _ in 0..10 { world.add_organism(seed_c(&config)); }

    world.run();

    let safe_name = name.replace(' ', "_");
    world.export_csv(&format!("D:/project/d0-vm/{}.csv", safe_name));
    if !world.genome_dumps.is_empty() {
        world.export_genomes(&format!("D:/project/d0-vm/{}_genomes.txt", safe_name));
    }

    // Export medium state at end
    let medium_nonzero: usize = world.medium.iter().filter(|&&v| v > 0).count();
    let medium_sum: u64 = world.medium.iter().map(|&v| v as u64).sum();
    eprintln!("  Medium: {} non-zero channels, sum={}", medium_nonzero, medium_sum);

    world.snapshots
}

/// Generate a single-seed analysis report (used for detailed seed 42 reporting).
pub fn analyze_and_report(exp_snapshots: &[Snapshot], ctrl_snapshots: &[Snapshot]) -> String {
    let mut report = String::new();

    report.push_str("# D0 Virtual Machine — Experiment Results\n\n");
    report.push_str("## Experiment Overview\n\n");
    report.push_str("**Hypothesis**: Freshness decay (operational closure constraint) drives evolution of conditional survival-priority behavior.\n\n");
    report.push_str("| Parameter | Value |\n");
    report.push_str("|-----------|-------|\n");
    report.push_str("| Population cap | 100 |\n");
    report.push_str("| Initial organisms | 10 Seed A + 10 Seed B |\n");
    report.push_str("| Food per tick | 50 |\n");
    report.push_str("| Mutation rate | 0.001 |\n");
    report.push_str("| Total ticks | 100,000 |\n");
    report.push_str("| Freshness max | 255 |\n");
    report.push_str("| Eat energy | 10 |\n");
    report.push_str("| Instruction cost | 1 |\n");
    report.push_str("| E_MAX | 1000 |\n\n");

    // Experimental group
    report_group(&mut report, "Experimental Group (freshness_decay = true)", exp_snapshots);
    // Control group
    report_group(&mut report, "Control Group (freshness_decay = false)", ctrl_snapshots);

    // Comparative analysis
    report.push_str("\n## Comparative Analysis\n\n");

    let exp_ss = compute_steady_state(exp_snapshots);
    let ctrl_ss = compute_steady_state(ctrl_snapshots);

    if exp_ss.survived && ctrl_ss.survived {
        report.push_str("### Steady-State Averages (tick 50k-100k)\n\n");
        report.push_str("| Metric | Experimental | Control | Difference |\n");
        report.push_str("|--------|-------------|---------|------------|\n");
        report.push_str(&format!("| EAT ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_ss.eat_ratio, ctrl_ss.eat_ratio, exp_ss.eat_ratio - ctrl_ss.eat_ratio));
        report.push_str(&format!("| REFRESH ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_ss.refresh_ratio, ctrl_ss.refresh_ratio, exp_ss.refresh_ratio - ctrl_ss.refresh_ratio));
        report.push_str(&format!("| DIVIDE ratio | {:.4} | {:.4} | {:.4} |\n",
            exp_ss.divide_ratio, ctrl_ss.divide_ratio, exp_ss.divide_ratio - ctrl_ss.divide_ratio));
        report.push_str(&format!("| Low-energy EAT rate | {:.4} | {:.4} | {:.4} |\n",
            exp_ss.low_energy_eat_rate, ctrl_ss.low_energy_eat_rate,
            exp_ss.low_energy_eat_rate - ctrl_ss.low_energy_eat_rate));
        report.push_str(&format!("| Avg population | {:.1} | {:.1} | {:.1} |\n",
            exp_ss.avg_population, ctrl_ss.avg_population, exp_ss.avg_population - ctrl_ss.avg_population));
        report.push_str(&format!("| Avg energy | {:.1} | {:.1} | {:.1} |\n",
            exp_ss.avg_energy, ctrl_ss.avg_energy, exp_ss.avg_energy - ctrl_ss.avg_energy));
    }

    report.push_str("\n---\n\n");
    report.push_str("*Generated by D0 VM v0.2.0 — Cognitive Life Science operational closure experiment*\n");

    report
}

fn report_group(report: &mut String, title: &str, snapshots: &[Snapshot]) {
    report.push_str(&format!("## {}\n\n", title));
    if let Some(last) = snapshots.last() {
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
    let step = if snapshots.len() > 20 { snapshots.len() / 20 } else { 1 };
    for s in snapshots.iter().step_by(step) {
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.4} | {:.4} | {:.4} |\n",
            s.tick, s.population, s.avg_energy, s.avg_code_length,
            s.eat_ratio * 100.0, s.refresh_ratio * 100.0, s.divide_ratio * 100.0,
        ));
    }
    report.push('\n');
}
