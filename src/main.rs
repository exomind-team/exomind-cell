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
mod tui;
mod cell_vm;

use std::fs;
use organism::Config;
use experiment::{run_experiment, analyze_and_report, compute_steady_state, SteadyState};
use cell_vm::{CellConfig, run_cell_experiment, cell_compute_steady_state, CellSteadyState};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // TUI mode (v2 only; cell v3 TUI is TODO)
    if args.iter().any(|a| a == "--tui") {
        if args.iter().any(|a| a == "--cell") {
            eprintln!("ERROR: --tui --cell combination not yet supported.");
            eprintln!("TUI currently works with v2 mode only. Cell v3 TUI is TODO.");
            return;
        }
        let mut config = Config::experimental();
        if args.iter().any(|a| a == "--no-decay") {
            config.freshness_decay = false;
        }
        if !args.iter().any(|a| a == "--stigmergy") {
            config.medium_size = 0;
        }
        config.total_ticks = 200_000;
        config.snapshot_interval = 100;
        config.genome_dump_interval = 0;

        eprintln!("Starting TUI mode (v2)...");
        if let Err(e) = tui::run_tui(config) {
            eprintln!("TUI error: {}", e);
        }
        return;
    }

    // Cell VM mode: cargo run -- --cell
    if args.iter().any(|a| a == "--cell") {
        run_cell_experiments();
        return;
    }

    eprintln!("D0 Virtual Machine — Operational Closure Experiment v3");
    eprintln!("======================================================");
    eprintln!("  500k ticks, E_MAX=1000, 5 seeds");
    eprintln!("  Use --tui for real-time visualization");
    eprintln!("  Use --cell for cell-based v3 experiments\n");

    let seeds: Vec<u64> = vec![42, 137, 256, 999, 2026];
    let num_seeds = seeds.len();

    // Helper closures for statistics
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

    let metrics: Vec<(&str, fn(&SteadyState) -> f64)> = vec![
        ("EAT ratio", |s: &SteadyState| s.eat_ratio),
        ("REFRESH ratio", |s: &SteadyState| s.refresh_ratio),
        ("DIVIDE ratio", |s: &SteadyState| s.divide_ratio),
        ("Low-E EAT rate", |s: &SteadyState| s.low_energy_eat_rate),
        ("Avg population", |s: &SteadyState| s.avg_population),
        ("Avg energy", |s: &SteadyState| s.avg_energy),
    ];

    // =========================================================================
    // Part 1: 5-seed multi-run (500k ticks)
    // =========================================================================
    let mut exp_stats: Vec<SteadyState> = Vec::new();
    let mut ctrl_stats: Vec<SteadyState> = Vec::new();
    let mut first_exp_snapshots = None;
    let mut first_ctrl_snapshots = None;

    for (i, &seed) in seeds.iter().enumerate() {
        eprintln!("\n>>> Seed {}/{}: {}", i + 1, num_seeds, seed);

        let exp_config = Config::experimental();
        let exp_snap = run_experiment(&format!("v3_exp_{}", seed), exp_config, seed);
        exp_stats.push(compute_steady_state(&exp_snap));
        if first_exp_snapshots.is_none() { first_exp_snapshots = Some(exp_snap); }

        let ctrl_config = Config::control();
        let ctrl_snap = run_experiment(&format!("v3_ctrl_{}", seed), ctrl_config, seed);
        ctrl_stats.push(compute_steady_state(&ctrl_snap));
        if first_ctrl_snapshots.is_none() { first_ctrl_snapshots = Some(ctrl_snap); }
    }

    // =========================================================================
    // Part 2: E_MAX impact analysis (seed=42 only)
    // =========================================================================
    eprintln!("\n>>> E_MAX Impact Analysis (seed=42)");

    // E_MAX=1000 (already run as seed 42 above, reuse)
    let emax_1000_exp = &exp_stats[0]; // seed 42 experimental
    let emax_1000_ctrl = &ctrl_stats[0]; // seed 42 control

    // E_MAX=unlimited (i32::MAX)
    let mut unlimited_exp_config = Config::experimental();
    unlimited_exp_config.e_max = i32::MAX;
    let unlimited_exp_snap = run_experiment("v3_emax_unlimited_exp", unlimited_exp_config, 42);
    let emax_unlimited_exp = compute_steady_state(&unlimited_exp_snap);

    let mut unlimited_ctrl_config = Config::control();
    unlimited_ctrl_config.e_max = i32::MAX;
    let unlimited_ctrl_snap = run_experiment("v3_emax_unlimited_ctrl", unlimited_ctrl_config, 42);
    let emax_unlimited_ctrl = compute_steady_state(&unlimited_ctrl_snap);

    // =========================================================================
    // Generate Report
    // =========================================================================
    let single_report = analyze_and_report(
        first_exp_snapshots.as_ref().unwrap(),
        first_ctrl_snapshots.as_ref().unwrap(),
    );

    let mut report = String::new();
    report.push_str("# D0 Virtual Machine — Experiment Results v3 (500k ticks)\n\n");
    report.push_str("## Changes from v2\n\n");
    report.push_str("- **500k ticks** (was 100k) — 5x longer runs for deeper evolutionary divergence\n");
    report.push_str("- **E_MAX impact analysis** — comparing E_MAX=1000 vs unlimited to isolate energy cap effects\n");
    report.push_str("- **Steady-state window**: tick 250k-500k (second half)\n\n");

    // Per-seed table
    report.push_str("## Multi-Seed Summary (steady-state, tick 250k-500k)\n\n");
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
    let exp_survived = exp_stats.iter().filter(|s| s.survived).count();
    let ctrl_survived = ctrl_stats.iter().filter(|s| s.survived).count();

    report.push_str("\n### Cross-Seed Averages (mean +/- std dev)\n\n");
    report.push_str(&format!("Survived: Exp {}/{}, Ctrl {}/{}\n\n", exp_survived, num_seeds, ctrl_survived, num_seeds));
    report.push_str("| Metric | Experimental | Control | Delta |\n");
    report.push_str("|--------|-------------|---------|-------|\n");

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

    // E_MAX impact analysis
    report.push_str("\n---\n\n");
    report.push_str("## E_MAX Impact Analysis (seed=42, 500k ticks)\n\n");
    report.push_str("Comparing E_MAX=1000 (capped) vs E_MAX=unlimited to determine if the energy cap\n");
    report.push_str("is responsible for the low-energy EAT rate inversion seen in some v2 seeds.\n\n");
    report.push_str("| Metric | E_MAX=1000 Exp | E_MAX=1000 Ctrl | E_MAX=unlim Exp | E_MAX=unlim Ctrl |\n");
    report.push_str("|--------|---------------|----------------|----------------|------------------|\n");

    let all_emax: [(&SteadyState, &str); 4] = [
        (emax_1000_exp, "1k_exp"),
        (emax_1000_ctrl, "1k_ctrl"),
        (&emax_unlimited_exp, "inf_exp"),
        (&emax_unlimited_ctrl, "inf_ctrl"),
    ];

    for (name, f) in &metrics {
        let vals: Vec<f64> = all_emax.iter().map(|(s, _)| f(s)).collect();
        let is_pct = name.contains("ratio") || name.contains("rate");
        if is_pct {
            report.push_str(&format!(
                "| {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
                name, vals[0] * 100.0, vals[1] * 100.0, vals[2] * 100.0, vals[3] * 100.0,
            ));
        } else {
            report.push_str(&format!(
                "| {} | {:.0} | {:.0} | {:.0} | {:.0} |\n",
                name, vals[0], vals[1], vals[2], vals[3],
            ));
        }
    }

    report.push_str("\n---\n\n");
    report.push_str("## Detailed Results (Seed 42, E_MAX=1000)\n\n");
    report.push_str(&single_report);

    fs::write("D:/project/d0-vm/RESULTS.md", &report).expect("Failed to write RESULTS.md");
    eprintln!("\nResults written to RESULTS.md");

    println!("{}", report);
}

// ============================================================================
// Cell VM v3 experiments
// ============================================================================

fn run_cell_experiments() {
    eprintln!("D0 Virtual Machine — Cell-based v3 Experiments");
    eprintln!("===============================================\n");

    let seeds: Vec<u64> = vec![42, 137, 256];
    let mut report = String::new();
    report.push_str("# D0 VM Cell-based v3 Experiment Results\n\n");
    report.push_str("## Architecture\n\n");
    report.push_str("- **Unified Cell system**: Code/Energy/Stomach cells with per-cell freshness\n");
    report.push_str("- **Two-step digestion**: EAT (food pool -> Stomach) + DIGEST (Stomach -> Energy)\n");
    report.push_str("- **Local REFRESH**: refreshes cells within radius R of current IP position\n");
    report.push_str("- **Gradual degradation**: individual cells die (freshness=0), not instant organism death\n\n");

    // =========================================================================
    // Experiment A: 3-seed exp vs ctrl (500k ticks)
    // =========================================================================
    report.push_str("## Experiment A: Cell v3 Exp vs Ctrl (3 seeds, 500k ticks)\n\n");
    report.push_str("| Seed | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |\n");
    report.push_str("|------|-------|----------|---------|-----------|------|---------|----------|--------|\n");

    for &seed in &seeds {
        eprintln!(">>> Cell Experiment A, seed {}", seed);

        let exp = CellConfig::experimental();
        let exp_snap = run_cell_experiment(&format!("cell_exp_{}", seed), exp, seed);
        let exp_ss = cell_compute_steady_state(&exp_snap);

        let ctrl = CellConfig::control();
        let ctrl_snap = run_cell_experiment(&format!("cell_ctrl_{}", seed), ctrl, seed);
        let ctrl_ss = cell_compute_steady_state(&ctrl_snap);

        for (group, ss) in [("Exp", &exp_ss), ("Ctrl", &ctrl_ss)] {
            report.push_str(&format!(
                "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
                seed, group, if ss.survived { "YES" } else { "NO" },
                ss.avg_population, ss.avg_energy,
                ss.eat_ratio * 100.0, ss.digest_ratio * 100.0,
                ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
            ));
        }
    }

    // =========================================================================
    // Experiment B: CELL_ENERGY_MAX gradient (seed=42)
    // =========================================================================
    report.push_str("\n## Experiment B: CELL_ENERGY_MAX Gradient (seed=42, 500k ticks)\n\n");
    report.push_str("| CEM | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-----|----------|---------|-----------|------|---------|----------|--------|\n");

    for cem in [5u8, 10, 20, 50] {
        eprintln!(">>> Cell Experiment B, cell_energy_max={}", cem);
        let mut config = CellConfig::experimental();
        config.cell_energy_max = cem;
        let snap = run_cell_experiment(&format!("cell_cem_{}", cem), config, 42);
        let ss = cell_compute_steady_state(&snap);
        report.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            cem, if ss.survived { "YES" } else { "NO" },
            ss.avg_population, ss.avg_energy,
            ss.eat_ratio * 100.0, ss.digest_ratio * 100.0,
            ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
        ));
    }

    report.push_str("\n---\n\n*Generated by D0 VM v3 Cell-based experiments*\n");

    fs::write("D:/project/d0-vm/CELL_RESULTS.md", &report).expect("Failed to write CELL_RESULTS.md");
    eprintln!("\nCell results written to CELL_RESULTS.md");
    println!("{}", report);
}
