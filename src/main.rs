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
mod stats;

use std::fs;
use std::io::Write as IoWrite;
use organism::Config;
use experiment::{run_experiment, analyze_and_report, compute_steady_state, SteadyState};
use cell_vm::{CellConfig, run_cell_experiment, run_cell_data_experiment, cell_compute_steady_state};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // TUI mode
    if args.iter().any(|a| a == "--tui") {
        if args.iter().any(|a| a == "--cell") {
            // Cell v3 TUI
            let mut config = CellConfig::experimental();
            config.cell_energy_max = 50;
            config.total_ticks = 500_000;
            config.snapshot_interval = 100;
            config.genome_dump_interval = 0;
            if args.iter().any(|a| a == "--no-decay") {
                config.freshness_decay = false;
            }
            eprintln!("Starting Cell v3 TUI mode...");
            if let Err(e) = tui::run_cell_tui(config) {
                eprintln!("TUI error: {}", e);
            }
            return;
        }
        // v2 TUI
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

        eprintln!("Starting v2 TUI mode...");
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

    // Statistical analysis mode: cargo run -- --stats
    if args.iter().any(|a| a == "--stats") {
        run_statistical_analysis();
        return;
    }

    eprintln!("D0 Virtual Machine — Operational Closure Experiment v3");
    eprintln!("======================================================");
    eprintln!("  500k ticks, E_MAX=1000, 5 seeds");
    eprintln!("  Use --tui for real-time visualization");
    eprintln!("  Use --cell for cell-based v3 experiments");
    eprintln!("  Use --stats for 30-seed statistical analysis\n");

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
    eprintln!("D0 Virtual Machine — Cell-based v3 Experiments (Paper Edition)");
    eprintln!("==============================================================\n");

    let mut report = String::new();
    report.push_str("# D0 VM Cell-based v3 Experiment Results (Paper Edition)\n\n");
    report.push_str("## Architecture\n\n");
    report.push_str("- **Unified Cell system**: Code/Energy/Stomach cells with per-cell freshness\n");
    report.push_str("- **Two-step digestion**: EAT (food pool -> Stomach) + DIGEST (Stomach -> Energy)\n");
    report.push_str("- **Local REFRESH**: refreshes cells within radius R of current IP position\n");
    report.push_str("- **Gradual degradation**: individual cells die (freshness=0), not instant organism death\n");
    report.push_str("- **CEM=50** (cell energy max): required for DIVIDE to emerge\n\n");

    // =========================================================================
    // Experiment 1: Multi-seed CEM=50 (5 seeds, exp+ctrl, 500k ticks)
    // =========================================================================
    let seeds: Vec<u64> = vec![42, 137, 256, 999, 2026];

    report.push_str("## Experiment 1: Multi-Seed Validation (CEM=50, R=5, 5 seeds, 500k ticks)\n\n");
    report.push_str("| Seed | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |\n");
    report.push_str("|------|-------|----------|---------|-----------|------|---------|----------|--------|\n");

    for &seed in &seeds {
        eprintln!(">>> Experiment 1: seed {}", seed);

        let mut exp = CellConfig::experimental();
        exp.cell_energy_max = 50;
        let exp_snap = run_cell_experiment(&format!("cell50_exp_{}", seed), exp, seed);
        let exp_ss = cell_compute_steady_state(&exp_snap);

        let mut ctrl = CellConfig::control();
        ctrl.cell_energy_max = 50;
        let ctrl_snap = run_cell_experiment(&format!("cell50_ctrl_{}", seed), ctrl, seed);
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
    // Experiment 2: REFRESH radius gradient (CEM=50, seed=42)
    // =========================================================================
    report.push_str("\n## Experiment 2: REFRESH Radius Gradient (CEM=50, seed=42, 500k ticks)\n\n");
    report.push_str("| R | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |\n");
    report.push_str("|---|-------|----------|---------|-----------|------|---------|----------|--------|\n");

    for r in [1usize, 2, 3, 5, 8] {
        eprintln!(">>> Experiment 2: R={}", r);

        let mut exp = CellConfig::experimental();
        exp.cell_energy_max = 50;
        exp.refresh_radius = r;
        let exp_snap = run_cell_experiment(&format!("cellR{}_exp", r), exp, 42);
        let exp_ss = cell_compute_steady_state(&exp_snap);

        let mut ctrl = CellConfig::control();
        ctrl.cell_energy_max = 50;
        ctrl.refresh_radius = r;
        let ctrl_snap = run_cell_experiment(&format!("cellR{}_ctrl", r), ctrl, 42);
        let ctrl_ss = cell_compute_steady_state(&ctrl_snap);

        for (group, ss) in [("Exp", &exp_ss), ("Ctrl", &ctrl_ss)] {
            report.push_str(&format!(
                "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
                r, group, if ss.survived { "YES" } else { "NO" },
                ss.avg_population, ss.avg_energy,
                ss.eat_ratio * 100.0, ss.digest_ratio * 100.0,
                ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
            ));
        }
    }

    // =========================================================================
    // Experiment 3: Data cell exploration (D2 prep)
    // =========================================================================
    report.push_str("\n## Experiment 3: Data Cell Exploration (CEM=50, seed=42, 500k ticks)\n\n");
    report.push_str("Seed D has a Data cell that stores energy readings for experience-based decisions.\n");
    report.push_str("Comparing: with Data cell seeds (5A+5B+10D) vs without (10A+10B).\n\n");
    report.push_str("| Setup | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|-------|----------|---------|-----------|------|---------|----------|--------|\n");

    eprintln!(">>> Experiment 3: Data cell exploration");

    // With Data cells
    let mut data_exp = CellConfig::experimental();
    data_exp.cell_energy_max = 50;
    let data_exp_snap = run_cell_data_experiment("cell_data_exp", data_exp, 42);
    let data_exp_ss = cell_compute_steady_state(&data_exp_snap);

    let mut data_ctrl = CellConfig::control();
    data_ctrl.cell_energy_max = 50;
    let data_ctrl_snap = run_cell_data_experiment("cell_data_ctrl", data_ctrl, 42);
    let data_ctrl_ss = cell_compute_steady_state(&data_ctrl_snap);

    // Without Data cells (baseline — reuse existing CEM=50 data if available, otherwise run)
    let mut base_exp = CellConfig::experimental();
    base_exp.cell_energy_max = 50;
    let base_exp_snap = run_cell_experiment("cell_nodata_exp", base_exp, 42);
    let base_exp_ss = cell_compute_steady_state(&base_exp_snap);

    let mut base_ctrl = CellConfig::control();
    base_ctrl.cell_energy_max = 50;
    let base_ctrl_snap = run_cell_experiment("cell_nodata_ctrl", base_ctrl, 42);
    let base_ctrl_ss = cell_compute_steady_state(&base_ctrl_snap);

    for (setup, group, ss) in [
        ("With Data", "Exp", &data_exp_ss), ("With Data", "Ctrl", &data_ctrl_ss),
        ("No Data", "Exp", &base_exp_ss), ("No Data", "Ctrl", &base_ctrl_ss),
    ] {
        report.push_str(&format!(
            "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            setup, group, if ss.survived { "YES" } else { "NO" },
            ss.avg_population, ss.avg_energy,
            ss.eat_ratio * 100.0, ss.digest_ratio * 100.0,
            ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
        ));
    }

    report.push_str("\n---\n\n*Generated by D0 VM v3 Cell-based experiments (paper edition)*\n");

    fs::write("D:/project/d0-vm/CELL_RESULTS.md", &report).expect("Failed to write CELL_RESULTS.md");
    eprintln!("\nCell results written to CELL_RESULTS.md");
    println!("{}", report);
}

// ============================================================================
// Large-scale statistical analysis with rayon parallelism
// ============================================================================

/// Steady-state summary for one run (thread-safe, no references).
struct RunResult {
    seed: u64,
    group: String, // "exp" or "ctrl"
    refresh: f64,
    eat: f64,
    divide: f64,
    pop: f64,
    energy: f64,
}

fn run_one_seed(seed: u64, freshness_decay: bool) -> RunResult {
    use cell_vm::{CellConfig, CellWorld, cell_seed_a, cell_seed_b};

    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.max_organisms = 1000;
    config.food_per_tick = 500;
    config.total_ticks = 2_000_000;
    config.snapshot_interval = 10_000;
    config.genome_dump_interval = 0; // no file I/O in parallel mode
    config.freshness_decay = freshness_decay;

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..50 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..50 { world.add_organism(cell_seed_b(&config)); }
    world.run();

    let ss = cell_compute_steady_state(&world.snapshots);
    RunResult {
        seed,
        group: if freshness_decay { "exp".into() } else { "ctrl".into() },
        refresh: ss.refresh_ratio,
        eat: ss.eat_ratio,
        divide: ss.divide_ratio,
        pop: ss.avg_population,
        energy: ss.avg_energy,
    }
}

fn run_statistical_analysis() {
    use rayon::prelude::*;

    eprintln!("D0 VM — Large-Scale Statistical Analysis (Paper I)");
    eprintln!("===================================================");
    eprintln!("  Cell v3, CEM=50, R=5, 2M ticks, 100 seeds, max_org=1000");
    eprintln!("  food_per_tick=500, rayon parallel\n");

    let seeds: Vec<u64> = (1000..1100).collect(); // 100 seeds
    let n = seeds.len();

    // Run all experimental seeds in parallel
    eprintln!("Running {} experimental seeds in parallel...", n);
    let exp_results: Vec<RunResult> = seeds.par_iter()
        .map(|&seed| {
            eprintln!("  [exp] seed {}", seed);
            run_one_seed(seed, true)
        })
        .collect();

    eprintln!("\nRunning {} control seeds in parallel...", n);
    let ctrl_results: Vec<RunResult> = seeds.par_iter()
        .map(|&seed| {
            eprintln!("  [ctrl] seed {}", seed);
            run_one_seed(seed, false)
        })
        .collect();

    // Extract vectors
    let exp_refresh: Vec<f64> = exp_results.iter().map(|r| r.refresh).collect();
    let exp_eat: Vec<f64> = exp_results.iter().map(|r| r.eat).collect();
    let exp_divide: Vec<f64> = exp_results.iter().map(|r| r.divide).collect();
    let exp_pop: Vec<f64> = exp_results.iter().map(|r| r.pop).collect();
    let exp_energy: Vec<f64> = exp_results.iter().map(|r| r.energy).collect();

    let ctrl_refresh: Vec<f64> = ctrl_results.iter().map(|r| r.refresh).collect();
    let ctrl_eat: Vec<f64> = ctrl_results.iter().map(|r| r.eat).collect();
    let ctrl_divide: Vec<f64> = ctrl_results.iter().map(|r| r.divide).collect();
    let ctrl_pop: Vec<f64> = ctrl_results.iter().map(|r| r.pop).collect();
    let ctrl_energy: Vec<f64> = ctrl_results.iter().map(|r| r.energy).collect();

    eprintln!("\nComputing statistics...");

    let comparisons = vec![
        stats::compare_groups("REFRESH ratio", &exp_refresh, &ctrl_refresh),
        stats::compare_groups("EAT ratio", &exp_eat, &ctrl_eat),
        stats::compare_groups("DIVIDE ratio", &exp_divide, &ctrl_divide),
        stats::compare_groups("Population", &exp_pop, &ctrl_pop),
        stats::compare_groups("Avg Energy", &exp_energy, &ctrl_energy),
    ];

    // Generate report
    let mut report = String::new();
    report.push_str("# D0 VM Large-Scale Statistical Analysis\n\n");
    report.push_str("## Parameters\n\n");
    report.push_str("- VM: Cell v3 (per-cell freshness)\n");
    report.push_str("- CEM: 50, R: 5, freshness_max: 255\n");
    report.push_str(&format!("- Ticks: 2,000,000 per run\n"));
    report.push_str(&format!("- Seeds: 1000-1099 ({} seeds)\n", n));
    report.push_str("- max_organisms: 1000, food_per_tick: 500\n");
    report.push_str("- Initial: 50 Seed A + 50 Seed B\n");
    report.push_str("- Steady-state window: tick 1M-2M\n");
    report.push_str("- Bootstrap: 10,000 resamples\n");
    report.push_str("- Parallel: rayon (all available cores)\n\n");

    // Per-seed table (first 20 + last 5)
    report.push_str("## Per-Seed Results (first 20 + last 5)\n\n");
    report.push_str("| Seed | Exp REFRESH% | Ctrl REFRESH% | Exp EAT% | Ctrl EAT% | Exp Pop | Ctrl Pop |\n");
    report.push_str("|------|-------------|---------------|---------|----------|---------|----------|\n");
    let show_indices: Vec<usize> = (0..20).chain(n-5..n).collect();
    for &i in &show_indices {
        report.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            seeds[i],
            exp_refresh[i] * 100.0, ctrl_refresh[i] * 100.0,
            exp_eat[i] * 100.0, ctrl_eat[i] * 100.0,
            exp_pop[i], ctrl_pop[i],
        ));
    }

    // Statistical tests table
    report.push_str("\n## Statistical Tests\n\n");
    report.push_str("| Metric | Exp (mean+/-sd) | Ctrl (mean+/-sd) | Diff | 95% CI | MW p | r_rb | d | KS D | KS p |\n");
    report.push_str("|--------|----------------|-----------------|------|--------|------|------|---|------|------|\n");
    for c in &comparisons {
        report.push_str(&c.to_markdown_row());
        report.push_str("\n");
    }

    // Interpretation
    report.push_str("\n## Interpretation\n\n");
    for c in &comparisons {
        let sig = if c.p_value < 0.001 { "***" }
            else if c.p_value < 0.01 { "**" }
            else if c.p_value < 0.05 { "*" }
            else { "n.s." };
        let d_label = if c.cohens_d.abs() >= 0.8 { "large" }
            else if c.cohens_d.abs() >= 0.5 { "medium" }
            else if c.cohens_d.abs() >= 0.2 { "small" }
            else { "negligible" };
        report.push_str(&format!(
            "- **{}**: MW p={:.4} {}, d={:.3} ({}), KS D={:.3} p={:.4}\n",
            c.metric_name, c.p_value, sig, c.cohens_d, d_label, c.ks_d, c.ks_p
        ));
    }

    report.push_str("\n---\n\n*Generated by D0 VM large-scale statistical analysis (rayon parallel)*\n");

    // Ensure output directory exists
    let _ = fs::create_dir_all("D:/project/d0-vm/data/large_scale");

    fs::write("D:/project/d0-vm/data/large_scale/statistical_analysis.md", &report)
        .expect("Failed to write statistical_analysis.md");

    // Per-seed CSV
    let mut seed_file = fs::File::create("D:/project/d0-vm/data/large_scale/per_seed_summary.csv")
        .expect("Failed to create per-seed CSV");
    writeln!(seed_file, "seed,group,refresh_ratio,eat_ratio,divide_ratio,population,avg_energy").unwrap();
    for r in exp_results.iter().chain(ctrl_results.iter()) {
        writeln!(seed_file, "{},{},{:.6},{:.6},{:.6},{:.2},{:.2}",
            r.seed, r.group, r.refresh, r.eat, r.divide, r.pop, r.energy).unwrap();
    }

    eprintln!("\nResults written to data/large_scale/");
    println!("{}", report);
}
