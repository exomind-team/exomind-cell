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
// 30-seed statistical analysis (Paper I requirement)
// ============================================================================

fn run_statistical_analysis() {
    eprintln!("D0 VM — 30-Seed Statistical Analysis (Paper I)");
    eprintln!("===============================================");
    eprintln!("  Cell v3, CEM=50, R=5, 500k ticks, 30 seeds\n");

    let seeds: Vec<u64> = (100..130).collect(); // 30 seeds: 100..129
    let n = seeds.len();

    let mut exp_refresh: Vec<f64> = Vec::new();
    let mut exp_eat: Vec<f64> = Vec::new();
    let mut exp_divide: Vec<f64> = Vec::new();
    let mut exp_pop: Vec<f64> = Vec::new();
    let mut exp_energy: Vec<f64> = Vec::new();

    let mut ctrl_refresh: Vec<f64> = Vec::new();
    let mut ctrl_eat: Vec<f64> = Vec::new();
    let mut ctrl_divide: Vec<f64> = Vec::new();
    let mut ctrl_pop: Vec<f64> = Vec::new();
    let mut ctrl_energy: Vec<f64> = Vec::new();

    // Time series: REFRESH% at each snapshot interval for seed 100 (first seed)
    let mut ts_exp_refresh: Vec<(u64, f64)> = Vec::new();
    let mut ts_ctrl_refresh: Vec<(u64, f64)> = Vec::new();

    for (i, &seed) in seeds.iter().enumerate() {
        eprintln!(">>> Seed {}/{}: {}", i + 1, n, seed);

        // Experimental
        let mut exp_config = CellConfig::experimental();
        exp_config.cell_energy_max = 50;
        let exp_snap = run_cell_experiment(&format!("stat_exp_{}", seed), exp_config, seed);
        let exp_ss = cell_compute_steady_state(&exp_snap);

        exp_refresh.push(exp_ss.refresh_ratio);
        exp_eat.push(exp_ss.eat_ratio);
        exp_divide.push(exp_ss.divide_ratio);
        exp_pop.push(exp_ss.avg_population);
        exp_energy.push(exp_ss.avg_energy);

        // Collect time series for first seed
        if i == 0 {
            for s in &exp_snap {
                if s.population > 0 {
                    ts_exp_refresh.push((s.tick, s.refresh_ratio));
                }
            }
        }

        // Control
        let mut ctrl_config = CellConfig::control();
        ctrl_config.cell_energy_max = 50;
        let ctrl_snap = run_cell_experiment(&format!("stat_ctrl_{}", seed), ctrl_config, seed);
        let ctrl_ss = cell_compute_steady_state(&ctrl_snap);

        ctrl_refresh.push(ctrl_ss.refresh_ratio);
        ctrl_eat.push(ctrl_ss.eat_ratio);
        ctrl_divide.push(ctrl_ss.divide_ratio);
        ctrl_pop.push(ctrl_ss.avg_population);
        ctrl_energy.push(ctrl_ss.avg_energy);

        if i == 0 {
            for s in &ctrl_snap {
                if s.population > 0 {
                    ts_ctrl_refresh.push((s.tick, s.refresh_ratio));
                }
            }
        }
    }

    // Statistical comparisons
    let comparisons = vec![
        stats::compare_groups("REFRESH ratio", &exp_refresh, &ctrl_refresh),
        stats::compare_groups("EAT ratio", &exp_eat, &ctrl_eat),
        stats::compare_groups("DIVIDE ratio", &exp_divide, &ctrl_divide),
        stats::compare_groups("Population", &exp_pop, &ctrl_pop),
        stats::compare_groups("Avg Energy", &exp_energy, &ctrl_energy),
    ];

    // Generate report
    let mut report = String::new();
    report.push_str("# D0 VM Statistical Analysis — 30-Seed Cell v3 Experiment\n\n");
    report.push_str("## Parameters\n\n");
    report.push_str("- VM: Cell v3 (per-cell freshness)\n");
    report.push_str("- CEM: 50, R: 5, freshness_max: 255\n");
    report.push_str("- Ticks: 500,000 per run\n");
    report.push_str("- Seeds: 100-129 (30 seeds)\n");
    report.push_str("- Steady-state window: tick 250k-500k\n");
    report.push_str("- Bootstrap: 10,000 resamples, seed 12345\n\n");

    // Per-seed table
    report.push_str("## Per-Seed Results\n\n");
    report.push_str("| Seed | Exp REFRESH% | Ctrl REFRESH% | Exp DIVIDE% | Ctrl DIVIDE% | Exp Pop | Ctrl Pop |\n");
    report.push_str("|------|-------------|---------------|------------|-------------|---------|----------|\n");
    for (i, &seed) in seeds.iter().enumerate() {
        report.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            seed,
            exp_refresh[i] * 100.0, ctrl_refresh[i] * 100.0,
            exp_divide[i] * 100.0, ctrl_divide[i] * 100.0,
            exp_pop[i], ctrl_pop[i],
        ));
    }

    // Statistical tests
    report.push_str("\n## Statistical Tests\n\n");
    report.push_str("| Metric | Exp (mean+/-sd) | Ctrl (mean+/-sd) | Diff | 95% CI | U | p-value | r_rb | Cohen's d |\n");
    report.push_str("|--------|----------------|-----------------|------|--------|---|---------|------|----------|\n");
    for c in &comparisons {
        report.push_str(&c.to_markdown_row());
        report.push_str("\n");
    }

    // Interpretation
    report.push_str("\n## Interpretation\n\n");
    let refresh_comp = &comparisons[0];
    if refresh_comp.p_value < 0.05 {
        report.push_str(&format!(
            "REFRESH ratio: **statistically significant** (p={:.4}). ",
            refresh_comp.p_value
        ));
        report.push_str(&format!(
            "95% CI of difference: [{:.4}, {:.4}]. ",
            refresh_comp.ci_lower, refresh_comp.ci_upper
        ));
        if refresh_comp.ci_lower > 0.0 {
            report.push_str("CI does not contain 0 — the effect is robust.\n\n");
        } else {
            report.push_str("CI contains 0 — despite significant p, effect direction uncertain.\n\n");
        }
    } else {
        report.push_str(&format!(
            "REFRESH ratio: **not statistically significant** (p={:.4}). ",
            refresh_comp.p_value
        ));
        report.push_str("The difference between experimental and control groups may be due to chance.\n\n");
    }

    let d = refresh_comp.cohens_d.abs();
    report.push_str(&format!("Effect size (Cohen's d): {:.3} — ", d));
    if d < 0.2 { report.push_str("negligible\n"); }
    else if d < 0.5 { report.push_str("small\n"); }
    else if d < 0.8 { report.push_str("medium\n"); }
    else { report.push_str("large\n"); }

    report.push_str("\n---\n\n*Generated by D0 VM statistical analysis module*\n");

    // Write report
    fs::write("D:/project/d0-vm/data/statistical_analysis.md", &report)
        .expect("Failed to write statistical_analysis.md");
    eprintln!("\nStatistical analysis written to data/statistical_analysis.md");

    // Write time series CSV
    let mut ts_file = fs::File::create("D:/project/d0-vm/data/time_series_refresh.csv")
        .expect("Failed to create time series CSV");
    writeln!(ts_file, "tick,exp_refresh_ratio,ctrl_refresh_ratio").unwrap();
    let max_len = ts_exp_refresh.len().min(ts_ctrl_refresh.len());
    for i in 0..max_len {
        writeln!(ts_file, "{},{:.6},{:.6}",
            ts_exp_refresh[i].0, ts_exp_refresh[i].1, ts_ctrl_refresh[i].1
        ).unwrap();
    }
    eprintln!("Time series written to data/time_series_refresh.csv");

    // Write per-seed CSV for external analysis
    let mut seed_file = fs::File::create("D:/project/d0-vm/data/per_seed_summary.csv")
        .expect("Failed to create per-seed CSV");
    writeln!(seed_file, "seed,group,refresh_ratio,eat_ratio,divide_ratio,population,avg_energy").unwrap();
    for (i, &seed) in seeds.iter().enumerate() {
        writeln!(seed_file, "{},exp,{:.6},{:.6},{:.6},{:.2},{:.2}",
            seed, exp_refresh[i], exp_eat[i], exp_divide[i], exp_pop[i], exp_energy[i]).unwrap();
        writeln!(seed_file, "{},ctrl,{:.6},{:.6},{:.6},{:.2},{:.2}",
            seed, ctrl_refresh[i], ctrl_eat[i], ctrl_divide[i], ctrl_pop[i], ctrl_energy[i]).unwrap();
    }
    eprintln!("Per-seed summary written to data/per_seed_summary.csv");

    println!("{}", report);
}
