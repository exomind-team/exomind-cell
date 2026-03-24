//! D0 Virtual Machine — Operational Closure Experiment
//!
//! Implements a minimal artificial life system to test the hypothesis:
//! "Freshness decay (operational closure constraint) drives the evolution
//!  of conditional survival-priority behavior."
//!
//! Based on the Cognitive Life Science D0 spec v2.

mod instruction;
mod organism;
mod world;
mod experiment;

use std::fs;
use organism::Config;
use experiment::{run_experiment, run_stigmergy_experiment, analyze_and_report, compute_steady_state, SteadyState};

fn main() {
    eprintln!("D0 Virtual Machine — Operational Closure Experiment v2");
    eprintln!("======================================================");
    eprintln!("  E_MAX = 1000, 5 seeds, genome dumps every 10k ticks\n");

    let seeds: Vec<u64> = vec![42, 137, 256, 999, 2026];
    let num_seeds = seeds.len();

    let mut exp_stats: Vec<SteadyState> = Vec::new();
    let mut ctrl_stats: Vec<SteadyState> = Vec::new();

    let mut first_exp_snapshots = None;
    let mut first_ctrl_snapshots = None;

    for (i, &seed) in seeds.iter().enumerate() {
        eprintln!("\n>>> Seed {}/{}: {}", i + 1, num_seeds, seed);

        let exp_config = Config::experimental();
        let exp_name = format!("experimental_seed_{}", seed);
        let exp_snap = run_experiment(&exp_name, exp_config, seed);
        exp_stats.push(compute_steady_state(&exp_snap));
        if first_exp_snapshots.is_none() {
            first_exp_snapshots = Some(exp_snap);
        }

        let ctrl_config = Config::control();
        let ctrl_name = format!("control_seed_{}", seed);
        let ctrl_snap = run_experiment(&ctrl_name, ctrl_config, seed);
        ctrl_stats.push(compute_steady_state(&ctrl_snap));
        if first_ctrl_snapshots.is_none() {
            first_ctrl_snapshots = Some(ctrl_snap);
        }
    }

    // Detailed report for seed 42
    let single_report = analyze_and_report(
        first_exp_snapshots.as_ref().unwrap(),
        first_ctrl_snapshots.as_ref().unwrap(),
    );

    // Multi-seed summary
    let mut report = String::new();
    report.push_str("# D0 Virtual Machine — Multi-Seed Experiment Results (v2)\n\n");
    report.push_str("## Changes from v1\n\n");
    report.push_str("- **E_MAX = 1000**: Energy cap prevents unbounded accumulation\n");
    report.push_str("- **5 random seeds**: Statistical validation across seeds 42, 137, 256, 999, 2026\n");
    report.push_str("- **Genome dumps**: Both oldest and most-evolved organisms saved every 10k ticks\n\n");

    report.push_str("## Multi-Seed Summary (steady-state averages, tick 50k-100k)\n\n");
    report.push_str("### Per-Seed Results\n\n");
    report.push_str("| Seed | Group | Survived | EAT% | REFRESH% | DIVIDE% | Low-E EAT% | Avg Pop | Avg Energy |\n");
    report.push_str("|------|-------|----------|------|----------|---------|-----------|---------|------------|\n");
    for (i, &seed) in seeds.iter().enumerate() {
        let e = &exp_stats[i];
        let c = &ctrl_stats[i];
        report.push_str(&format!(
            "| {} | Exp | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.0} |\n",
            seed, if e.survived { "YES" } else { "NO" },
            e.eat_ratio * 100.0, e.refresh_ratio * 100.0, e.divide_ratio * 100.0,
            e.low_energy_eat_rate * 100.0, e.avg_population, e.avg_energy,
        ));
        report.push_str(&format!(
            "| {} | Ctrl | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.0} |\n",
            seed, if c.survived { "YES" } else { "NO" },
            c.eat_ratio * 100.0, c.refresh_ratio * 100.0, c.divide_ratio * 100.0,
            c.low_energy_eat_rate * 100.0, c.avg_population, c.avg_energy,
        ));
    }

    // Cross-seed averages
    let avg = |stats: &[SteadyState], f: fn(&SteadyState) -> f64| -> f64 {
        let vals: Vec<f64> = stats.iter().filter(|s| s.survived).map(|s| f(s)).collect();
        if vals.is_empty() { 0.0 } else { vals.iter().sum::<f64>() / vals.len() as f64 }
    };
    let std_dev = |stats: &[SteadyState], f: fn(&SteadyState) -> f64| -> f64 {
        let vals: Vec<f64> = stats.iter().filter(|s| s.survived).map(|s| f(s)).collect();
        if vals.len() < 2 { return 0.0; }
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (vals.len() - 1) as f64;
        var.sqrt()
    };

    let exp_survived = exp_stats.iter().filter(|s| s.survived).count();
    let ctrl_survived = ctrl_stats.iter().filter(|s| s.survived).count();

    report.push_str("\n### Cross-Seed Averages (mean +/- std dev)\n\n");
    report.push_str(&format!("Survived: Exp {}/{}, Ctrl {}/{}\n\n", exp_survived, num_seeds, ctrl_survived, num_seeds));
    report.push_str("| Metric | Experimental | Control | Delta |\n");
    report.push_str("|--------|-------------|---------|-------|\n");

    let metrics: Vec<(&str, fn(&SteadyState) -> f64)> = vec![
        ("EAT ratio", |s: &SteadyState| s.eat_ratio),
        ("REFRESH ratio", |s: &SteadyState| s.refresh_ratio),
        ("DIVIDE ratio", |s: &SteadyState| s.divide_ratio),
        ("Low-E EAT rate", |s: &SteadyState| s.low_energy_eat_rate),
        ("Avg population", |s: &SteadyState| s.avg_population),
        ("Avg energy", |s: &SteadyState| s.avg_energy),
    ];

    for (name, f) in &metrics {
        let e_avg = avg(&exp_stats, *f);
        let e_sd = std_dev(&exp_stats, *f);
        let c_avg = avg(&ctrl_stats, *f);
        let c_sd = std_dev(&ctrl_stats, *f);
        let is_pct = name.contains("ratio") || name.contains("rate");
        if is_pct {
            report.push_str(&format!(
                "| {} | {:.1}% +/- {:.1}% | {:.1}% +/- {:.1}% | {:.1}% |\n",
                name, e_avg * 100.0, e_sd * 100.0, c_avg * 100.0, c_sd * 100.0, (e_avg - c_avg) * 100.0,
            ));
        } else {
            report.push_str(&format!(
                "| {} | {:.1} +/- {:.1} | {:.1} +/- {:.1} | {:.1} |\n",
                name, e_avg, e_sd, c_avg, c_sd, e_avg - c_avg,
            ));
        }
    }

    // Competition experiments: vary food pressure
    report.push_str("\n---\n\n");
    report.push_str("## Competition Experiment: Varying Food Pressure\n\n");
    report.push_str("Testing how food scarcity affects population dynamics with freshness_decay=true.\n\n");
    report.push_str("| Food/tick | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-----------|----------|---------|-----------|------|----------|--------|\n");

    for food in [10, 20, 30, 40, 50, 75, 100] {
        let config = Config::competition(food);
        let name = format!("competition_food_{}", food);
        let snaps = run_experiment(&name, config, 42);
        let ss = compute_steady_state(&snaps);
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.0} | {:.1} | {:.1} | {:.1} |\n",
            food, if ss.survived { "YES" } else { "NO" },
            ss.avg_population, ss.avg_energy,
            ss.eat_ratio * 100.0, ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
        ));
    }

    // Stigmergy experiment
    report.push_str("\n---\n\n");
    report.push_str("## Stigmergy Experiment: Indirect Communication via Shared Medium\n\n");
    report.push_str("Testing whether organisms evolve to use EMIT/SAMPLE for indirect coordination.\n");
    report.push_str("Setup: 5 Seed A + 5 Seed B + 10 Seed C (stigmergy-capable), medium_size=256.\n\n");

    // With stigmergy vs without (medium_size=0)
    let stig_config = Config::stigmergy();
    let stig_snaps = run_stigmergy_experiment("stigmergy_with_medium", stig_config, 42);
    let stig_ss = compute_steady_state(&stig_snaps);

    let mut no_stig_config = Config::stigmergy();
    no_stig_config.medium_size = 0; // Disable medium
    let no_stig_snaps = run_stigmergy_experiment("stigmergy_no_medium", no_stig_config, 42);
    let no_stig_ss = compute_steady_state(&no_stig_snaps);

    report.push_str("| Metric | With Medium | No Medium | Delta |\n");
    report.push_str("|--------|------------|-----------|-------|\n");
    report.push_str(&format!("| Survived | {} | {} | — |\n",
        if stig_ss.survived { "YES" } else { "NO" },
        if no_stig_ss.survived { "YES" } else { "NO" }));
    report.push_str(&format!("| Avg population | {:.1} | {:.1} | {:.1} |\n",
        stig_ss.avg_population, no_stig_ss.avg_population,
        stig_ss.avg_population - no_stig_ss.avg_population));
    report.push_str(&format!("| Avg energy | {:.0} | {:.0} | {:.0} |\n",
        stig_ss.avg_energy, no_stig_ss.avg_energy,
        stig_ss.avg_energy - no_stig_ss.avg_energy));
    report.push_str(&format!("| EAT% | {:.1} | {:.1} | {:.1} |\n",
        stig_ss.eat_ratio * 100.0, no_stig_ss.eat_ratio * 100.0,
        (stig_ss.eat_ratio - no_stig_ss.eat_ratio) * 100.0));
    report.push_str(&format!("| REFRESH% | {:.1} | {:.1} | {:.1} |\n",
        stig_ss.refresh_ratio * 100.0, no_stig_ss.refresh_ratio * 100.0,
        (stig_ss.refresh_ratio - no_stig_ss.refresh_ratio) * 100.0));
    report.push_str(&format!("| DIVIDE% | {:.1} | {:.1} | {:.1} |\n",
        stig_ss.divide_ratio * 100.0, no_stig_ss.divide_ratio * 100.0,
        (stig_ss.divide_ratio - no_stig_ss.divide_ratio) * 100.0));

    report.push_str("\n---\n\n");
    report.push_str("## Detailed Results (Seed 42)\n\n");
    report.push_str(&single_report);

    fs::write("D:/project/d0-vm/RESULTS.md", &report).expect("Failed to write RESULTS.md");
    eprintln!("\nResults written to RESULTS.md");

    println!("{}", report);
}
