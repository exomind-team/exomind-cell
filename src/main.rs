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
mod signal;

use std::fs;
use std::io::Write as IoWrite;
use organism::Config;
use experiment::{run_experiment, analyze_and_report, compute_steady_state, SteadyState};
use cell_vm::{CellConfig, CellWorld, cell_seed_a, cell_seed_b, cell_seed_f, cell_seed_g, cell_compute_steady_state, run_cell_experiment, run_cell_data_experiment, run_cell_growth_experiment};

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

    // EXP-011 sense-making: cargo run -- --exp011
    if args.iter().any(|a| a == "--exp011") {
        run_exp011();
        return;
    }

    // EXP-012 history impact: cargo run -- --exp012
    if args.iter().any(|a| a == "--exp012") {
        run_exp012();
        return;
    }

    // EXP-014 GATE learning: cargo run -- --exp014
    if args.iter().any(|a| a == "--exp014") {
        run_exp014();
        return;
    }

    // Real CPU mode: cargo run -- --real-cpu
    if args.iter().any(|a| a == "--real-cpu") {
        run_realcpu_experiment();
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

    // No recognized flag or --help — show help and exit
    if !args.iter().any(|a| a == "--run-v2") {
        println!(r#"
  ╔═══════════════════════════════════════════╗
  ║         exomind-cell v0.1.0               ║
  ║    Cognitive Life Science VM               ║
  ╠═══════════════════════════════════════════╣
  ║  Modes:                                    ║
  ║    --tui            v2 Classic TUI         ║
  ║    --tui --cell     v3 Cell-based TUI      ║
  ║    --tui --cell --no-decay  v3 Control     ║
  ║    --cell           v3 Cell experiments     ║
  ║    --stats          100-seed parallel       ║
  ║    (no flag)        v2 experiments          ║
  ║                                            ║
  ║  TUI Controls:                             ║
  ║    p   Pause/Resume                        ║
  ║    q   Quit                                ║
  ║    h   Help overlay                        ║
  ║    s   Step (when paused)                  ║
  ║    i   Inspect organism                    ║
  ║    +/- Speed up/down                       ║
  ╚═══════════════════════════════════════════╝
"#);
        return;
    }

    eprintln!("ExoMind Cell — Operational Closure Experiment v3");
    eprintln!("================================================");
    eprintln!("  500k ticks, E_MAX=1000, 5 seeds\n");

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

    // =========================================================================
    // Experiment 4: Growth (ALLOC) exploration
    // =========================================================================
    report.push_str("\n## Experiment 4: Growth (ALLOC) Exploration (CEM=50, seed=42, 500k ticks)\n\n");
    report.push_str("Seed E can ALLOC new Energy cells when energy is sufficient (body grows).\n\n");
    report.push_str("| Setup | Group | Survived | Avg Pop | Avg Cells | Avg Energy | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|-------|----------|---------|----------|-----------|------|----------|--------|\n");

    eprintln!(">>> Experiment 4: Growth (ALLOC)");

    let mut growth_exp = CellConfig::experimental();
    growth_exp.cell_energy_max = 50;
    let growth_exp_snap = run_cell_growth_experiment("cell_growth_exp", growth_exp, 42);
    let growth_exp_ss = cell_compute_steady_state(&growth_exp_snap);

    let mut growth_ctrl = CellConfig::control();
    growth_ctrl.cell_energy_max = 50;
    let growth_ctrl_snap = run_cell_growth_experiment("cell_growth_ctrl", growth_ctrl, 42);
    let growth_ctrl_ss = cell_compute_steady_state(&growth_ctrl_snap);

    // Baseline without growth seeds
    let mut nogrow_exp = CellConfig::experimental();
    nogrow_exp.cell_energy_max = 50;
    let nogrow_exp_snap = run_cell_experiment("cell_nogrow_exp", nogrow_exp, 42);
    let nogrow_exp_ss = cell_compute_steady_state(&nogrow_exp_snap);

    for (setup, group, ss) in [
        ("With ALLOC", "Exp", &growth_exp_ss), ("With ALLOC", "Ctrl", &growth_ctrl_ss),
        ("No ALLOC", "Exp", &nogrow_exp_ss),
    ] {
        report.push_str(&format!(
            "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            setup, group, if ss.survived { "YES" } else { "NO" },
            ss.avg_population, ss.avg_cell_count, ss.avg_energy,
            ss.eat_ratio * 100.0, ss.refresh_ratio * 100.0, ss.divide_ratio * 100.0,
        ));
    }

    // =========================================================================
    // Experiment 5: Multi-food type (value evaluation, D1)
    // =========================================================================
    report.push_str("\n## Experiment 5: Multi-Food Type (CEM=50, seed=42, 500k ticks)\n\n");
    report.push_str("Simple food (energy=5, instant) + Complex food (energy=20, needs extra DIGEST).\n\n");
    report.push_str("| Setup | Group | Survived | Pop | Energy | EAT% | Simple% | Complex% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|-------|----------|-----|--------|------|---------|----------|----------|--------|\n");

    eprintln!(">>> Experiment 5: Multi-food type");

    // Exp: multi-food + freshness decay
    let mut mf_exp = CellConfig::experimental();
    mf_exp.cell_energy_max = 50;
    mf_exp.multi_food = true;
    mf_exp.food_per_tick = 0;
    let mf_exp_snap = run_cell_experiment("cell_multifood_exp", mf_exp, 42);
    let mf_exp_ss = cell_compute_steady_state(&mf_exp_snap);

    // Ctrl A: multi-food + no decay
    let mut mf_ctrl = CellConfig::control();
    mf_ctrl.cell_energy_max = 50;
    mf_ctrl.multi_food = true;
    mf_ctrl.food_per_tick = 0;
    let mf_ctrl_snap = run_cell_experiment("cell_multifood_ctrl", mf_ctrl, 42);
    let mf_ctrl_ss = cell_compute_steady_state(&mf_ctrl_snap);

    // Ctrl B: simple-only food + decay
    let mut sf_exp = CellConfig::experimental();
    sf_exp.cell_energy_max = 50;
    let sf_exp_snap = run_cell_experiment("cell_simplefood_exp", sf_exp, 42);
    let sf_exp_ss = cell_compute_steady_state(&sf_exp_snap);

    for (setup, group, ss) in [
        ("Multi-food", "Exp", &mf_exp_ss),
        ("Multi-food", "Ctrl", &mf_ctrl_ss),
        ("Simple-only", "Exp", &sf_exp_ss),
    ] {
        report.push_str(&format!(
            "| {} | {} | {} | {:.1} | {:.1} | {:.1} | — | — | {:.1} | {:.1} |\n",
            setup, group, if ss.survived { "YES" } else { "NO" },
            ss.avg_population, ss.avg_energy,
            ss.eat_ratio * 100.0,
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

// ============================================================================
// Real CPU data-driven experiment
// ============================================================================

fn run_realcpu_experiment() {
    use sysinfo::System;
    use cell_vm::{CellConfig, CellWorld, cell_seed_a, cell_seed_b};

    eprintln!("ExoMind Cell — Real CPU Experiment");
    eprintln!("==================================");
    eprintln!("  Food = f(CPU availability). Base food = 500.\n");

    let mut sys = System::new_all();
    let base_food: i32 = 500;
    let total_ticks: u64 = 500_000;
    let cpu_sample_interval: u64 = 100;

    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.food_per_tick = 50; // Baseline food (CPU modulation adds on top)
    config.total_ticks = total_ticks;
    config.max_organisms = 200;

    let mut world = CellWorld::new(config.clone(), 42);
    for _ in 0..20 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_b(&config)); }

    // Control: constant food at same average as base_food injection
    let mut ctrl_config = config.clone();
    ctrl_config.food_per_tick = 50 + base_food / (cpu_sample_interval as i32); // base + avg injection
    let mut ctrl_world = CellWorld::new(ctrl_config.clone(), 42);
    for _ in 0..20 { ctrl_world.add_organism(cell_seed_a(&ctrl_config)); }
    for _ in 0..20 { ctrl_world.add_organism(cell_seed_b(&ctrl_config)); }

    let mut cpu_log: Vec<(u64, f32, i32)> = Vec::new(); // (tick, cpu_usage, food_injected)

    for t in 0..total_ticks {
        // Sample CPU every cpu_sample_interval ticks
        if t % cpu_sample_interval == 0 {
            sys.refresh_cpu_usage();
            let cpu_usage = sys.global_cpu_usage() / 100.0; // 0.0-1.0
            // Floor: always provide at least 30% of base food
            // Scale remaining 70% by CPU availability
            // This prevents self-starvation (experiment itself uses CPU)
            let available = 0.3 + 0.7 * (1.0 - cpu_usage).max(0.0);
            let food = (base_food as f32 * available) as i32;
            world.food_pool += food;
            cpu_log.push((t, cpu_usage, food));

            if t % (total_ticks / 10) == 0 && t > 0 {
                let alive = world.organisms.iter().filter(|o| o.alive).count();
                eprintln!("  tick {}/{} — CPU {:.0}% → food {} — {} alive",
                    t, total_ticks, cpu_usage * 100.0, food, alive);
            }
        }

        world.tick();
        ctrl_world.tick();
    }

    // Compute steady states
    world.take_snapshot();
    ctrl_world.take_snapshot();
    let exp_ss = cell_compute_steady_state(&world.snapshots);
    let ctrl_ss = cell_compute_steady_state(&ctrl_world.snapshots);

    // Write CPU log
    let _ = fs::create_dir_all("D:/project/d0-vm/data");
    let mut log_file = fs::File::create("D:/project/d0-vm/data/cpu_log.csv")
        .expect("Failed to create cpu_log.csv");
    writeln!(log_file, "tick,cpu_usage,food_injected").unwrap();
    for (t, cpu, food) in &cpu_log {
        writeln!(log_file, "{},{:.4},{}", t, cpu, food).unwrap();
    }

    // Report
    let mut report = String::new();
    report.push_str("# Real CPU Experiment Results\n\n");
    report.push_str(&format!("- Base food: {}\n", base_food));
    report.push_str(&format!("- CPU sample interval: {} ticks\n", cpu_sample_interval));
    report.push_str(&format!("- Total ticks: {}\n", total_ticks));
    report.push_str(&format!("- CPU samples: {}\n\n", cpu_log.len()));

    let avg_cpu: f32 = cpu_log.iter().map(|(_, c, _)| c).sum::<f32>() / cpu_log.len() as f32;
    let avg_food: f32 = cpu_log.iter().map(|(_, _, f)| *f as f32).sum::<f32>() / cpu_log.len() as f32;
    report.push_str(&format!("- Avg CPU usage: {:.1}%\n", avg_cpu * 100.0));
    report.push_str(&format!("- Avg food injected: {:.0}\n\n", avg_food));

    report.push_str("| Metric | Real CPU | Constant Food |\n");
    report.push_str("|--------|---------|---------------|\n");
    report.push_str(&format!("| Survived | {} | {} |\n",
        if exp_ss.survived { "YES" } else { "NO" },
        if ctrl_ss.survived { "YES" } else { "NO" }));
    report.push_str(&format!("| Avg Pop | {:.1} | {:.1} |\n", exp_ss.avg_population, ctrl_ss.avg_population));
    report.push_str(&format!("| Avg Energy | {:.1} | {:.1} |\n", exp_ss.avg_energy, ctrl_ss.avg_energy));
    report.push_str(&format!("| EAT% | {:.1} | {:.1} |\n", exp_ss.eat_ratio * 100.0, ctrl_ss.eat_ratio * 100.0));
    report.push_str(&format!("| REFRESH% | {:.1} | {:.1} |\n", exp_ss.refresh_ratio * 100.0, ctrl_ss.refresh_ratio * 100.0));
    report.push_str(&format!("| DIVIDE% | {:.1} | {:.1} |\n", exp_ss.divide_ratio * 100.0, ctrl_ss.divide_ratio * 100.0));

    report.push_str("\n---\n*Real CPU experiment: food scales with system CPU availability*\n");

    fs::write("D:/project/d0-vm/data/realcpu_results.md", &report).expect("Failed to write");
    eprintln!("\nResults written to data/realcpu_results.md");
    println!("{}", report);
}

// ============================================================================
// EXP-011: Sense-making (signal prediction) experiment
// ============================================================================

/// Run one sense-making trial: signal → medium → delayed food modulation.
fn run_sensemaking_trial(
    group: &signal::SenseMakingGroup,
    seed: u64,
) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.food_per_tick = 30; // Baseline food (signal modulation adds more)
    config.total_ticks = 500_000;
    config.max_organisms = 200;
    config.medium_size = 256;
    config.snapshot_interval = 1000;
    config.genome_dump_interval = 0;

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_f(&config)); }

    let base_food: i32 = 300;
    let mut signal_history: Vec<f32> = Vec::new(); // ring buffer for delta delay

    for t in 0..config.total_ticks {
        // Generate signal value
        let signal_val = if group.use_real_cpu {
            // Simplified: use hash-based pseudo to avoid sysinfo in parallel
            let block = t / 100;
            let hash = block.wrapping_mul(6364136223846793005).wrapping_add(seed) >> 33;
            (hash % 100) as f32 / 100.0
        } else {
            group.signal.value_at(t)
        };

        // Write signal to medium channel 0 (organisms can SAMPLE it)
        if !world.medium.is_empty() {
            world.medium[0] = (signal_val * 200.0) as u8;
        }

        // Store signal for delayed food modulation
        signal_history.push(signal_val);

        // Food tracks signal with delta-tick delay
        let delayed_signal = if group.delta > 0 && signal_history.len() > group.delta as usize {
            signal_history[signal_history.len() - 1 - group.delta as usize]
        } else if group.delta == 0 {
            signal_val // synchronous
        } else {
            0.5 // not enough history yet, use neutral
        };

        // Inject food proportional to delayed signal
        if t % 10 == 0 { // every 10 ticks
            let food = (base_food as f32 * (0.2 + 0.8 * delayed_signal)) as i32;
            world.food_pool += food;
        }

        world.tick();

        // Keep signal history bounded
        if signal_history.len() > 10000 {
            signal_history.drain(0..5000);
        }
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_exp011() {
    use rayon::prelude::*;

    eprintln!("EXP-011: Sense-Making (Signal Prediction) Experiment");
    eprintln!("=====================================================");
    eprintln!("  6 groups x 30 seeds = 180 runs, 500k ticks each\n");

    let groups = signal::SenseMakingGroup::all_groups();
    let seeds: Vec<u64> = (200..230).collect(); // 30 seeds

    let _ = fs::create_dir_all("D:/project/d0-vm/data/experiments/EXP-011/raw");

    // Run all groups in parallel
    let mut all_results: Vec<(String, Vec<cell_vm::CellSteadyState>)> = Vec::new();

    for group in &groups {
        eprintln!(">>> Group {}: {} seeds parallel...", group.name, seeds.len());
        let group_clone = group.clone();
        let results: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
            .map(|&seed| {
                run_sensemaking_trial(&group_clone, seed)
            })
            .collect();
        all_results.push((group.name.clone(), results));
    }

    // Generate report
    let mut report = String::new();
    report.push_str("# EXP-011: Sense-Making (Signal Prediction) Results\n\n");
    report.push_str("## Design\n\n");
    report.push_str("- Signal written to medium[0], food tracks signal with delta-tick delay\n");
    report.push_str("- Organisms with SAMPLE can read signal, organisms without cannot\n");
    report.push_str("- Seed F: SAMPLE → STORE → EAT → DIGEST → LOAD → CMP → conditional extra EAT\n\n");

    report.push_str("## Summary (30 seeds per group)\n\n");
    report.push_str("| Group | Signal | Delta | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|--------|-------|----------|---------|-----------|------|----------|--------|\n");

    let mut group_eat_vecs: Vec<(String, Vec<f64>)> = Vec::new();

    for (name, results) in &all_results {
        let survived = results.iter().filter(|r| r.survived).count();
        let n = results.len() as f64;
        let avg_pop: f64 = results.iter().map(|r| r.avg_population).sum::<f64>() / n;
        let avg_energy: f64 = results.iter().map(|r| r.avg_energy).sum::<f64>() / n;
        let avg_eat: f64 = results.iter().map(|r| r.eat_ratio).sum::<f64>() / n;
        let avg_refresh: f64 = results.iter().map(|r| r.refresh_ratio).sum::<f64>() / n;
        let avg_divide: f64 = results.iter().map(|r| r.divide_ratio).sum::<f64>() / n;

        let group_info = groups.iter().find(|g| g.name == *name).unwrap();
        let signal_desc = if group_info.use_real_cpu { "Real CPU" }
            else { match &group_info.signal {
                signal::SignalType::SquareWave { .. } => "Square",
                signal::SignalType::SineWave { .. } => "Sine",
                signal::SignalType::Random { .. } => "Random",
                signal::SignalType::None => "None",
            }};

        report.push_str(&format!(
            "| {} | {} | {} | {}/{} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            name, signal_desc, group_info.delta,
            survived, results.len(),
            avg_pop, avg_energy,
            avg_eat * 100.0, avg_refresh * 100.0, avg_divide * 100.0,
        ));

        group_eat_vecs.push((name.clone(), results.iter().map(|r| r.eat_ratio).collect()));
    }

    // Statistical comparisons: Group A vs Group D (predictable vs synchronous)
    // and Group A vs Group E (predictable vs random)
    report.push_str("\n## Statistical Comparisons\n\n");
    report.push_str("| Comparison | Metric | Diff | 95% CI | MW p | d | KS D | KS p |\n");
    report.push_str("|-----------|--------|------|--------|------|---|------|------|\n");

    let find_group = |name: &str| -> &Vec<f64> {
        &group_eat_vecs.iter().find(|(n, _)| n == name).unwrap().1
    };

    // A vs D (predictable with delay vs synchronous)
    let comp_ad = stats::compare_groups("A vs D EAT", find_group("A_square_d200"), find_group("D_square_d0"));
    report.push_str(&format!(
        "| A(d200) vs D(d0) | EAT | {:.4} | [{:.4}, {:.4}] | {:.4} | {:.3} | {:.3} | {:.4} |\n",
        comp_ad.mean_diff, comp_ad.ci_lower, comp_ad.ci_upper,
        comp_ad.p_value, comp_ad.cohens_d, comp_ad.ks_d, comp_ad.ks_p,
    ));

    // A vs E (predictable vs random)
    let comp_ae = stats::compare_groups("A vs E EAT", find_group("A_square_d200"), find_group("E_random_d200"));
    report.push_str(&format!(
        "| A(predict) vs E(random) | EAT | {:.4} | [{:.4}, {:.4}] | {:.4} | {:.3} | {:.3} | {:.4} |\n",
        comp_ae.mean_diff, comp_ae.ci_lower, comp_ae.ci_upper,
        comp_ae.p_value, comp_ae.cohens_d, comp_ae.ks_d, comp_ae.ks_p,
    ));

    // A vs F (signal vs no signal)
    let comp_af = stats::compare_groups("A vs F EAT", find_group("A_square_d200"), find_group("F_nosignal"));
    report.push_str(&format!(
        "| A(signal) vs F(none) | EAT | {:.4} | [{:.4}, {:.4}] | {:.4} | {:.3} | {:.3} | {:.4} |\n",
        comp_af.mean_diff, comp_af.ci_lower, comp_af.ci_upper,
        comp_af.p_value, comp_af.cohens_d, comp_af.ks_d, comp_af.ks_p,
    ));

    report.push_str("\n---\n*EXP-011: Sense-making signal prediction experiment*\n");

    // Write outputs
    let exp_dir = "D:/project/d0-vm/data/experiments/EXP-011";
    fs::write(format!("{}/results.md", exp_dir), &report).expect("Failed to write results");

    // Per-seed CSV
    let mut csv = fs::File::create(format!("{}/raw/per_seed.csv", exp_dir)).expect("CSV");
    writeln!(csv, "group,seed,survived,avg_pop,avg_energy,eat_ratio,refresh_ratio,divide_ratio").unwrap();
    for (name, results) in &all_results {
        for (i, r) in results.iter().enumerate() {
            writeln!(csv, "{},{},{},{:.2},{:.2},{:.6},{:.6},{:.6}",
                name, seeds[i], r.survived, r.avg_population, r.avg_energy,
                r.eat_ratio, r.refresh_ratio, r.divide_ratio).unwrap();
        }
    }

    eprintln!("\nResults: data/experiments/EXP-011/results.md");
    println!("{}", report);
}

// ============================================================================
// EXP-012: History Impact (D2 experience learning)
// ============================================================================

fn run_history_trial(seed: u64, abundant_first: bool) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.total_ticks = 500_000;
    config.max_organisms = 200;
    config.medium_size = 256;
    config.snapshot_interval = 1000;
    config.genome_dump_interval = 0;

    // Start with appropriate food level
    if abundant_first {
        config.food_per_tick = 500; // abundant phase
    } else {
        config.food_per_tick = 50; // scarce from start
    }

    let mut world = CellWorld::new(config.clone(), seed);
    // Use Seed F (has Data cell for experience storage)
    for _ in 0..20 { world.add_organism(cell_seed_f(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    let switch_tick: u64 = 10_000; // switch at tick 10k

    for t in 0..config.total_ticks {
        // Switch food at tick 10k for abundant_first group
        if abundant_first && t == switch_tick {
            world.config.food_per_tick = 50; // switch to scarce
        }
        world.tick();
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_exp012() {
    use rayon::prelude::*;

    eprintln!("EXP-012: History Impact (D2 Experience Learning)");
    eprintln!("================================================");
    eprintln!("  Group 1: abundant→scarce (500→50 at tick 10k)");
    eprintln!("  Group 2: always scarce (50)\n");

    let seeds: Vec<u64> = (300..310).collect(); // 10 seeds

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-012-history-impact/data");

    eprintln!("Running Group 1 (abundant→scarce)...");
    let g1_results: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&seed| run_history_trial(seed, true))
        .collect();

    eprintln!("Running Group 2 (always scarce)...");
    let g2_results: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&seed| run_history_trial(seed, false))
        .collect();

    // Extract metrics
    let g1_eat: Vec<f64> = g1_results.iter().map(|r| r.eat_ratio).collect();
    let g2_eat: Vec<f64> = g2_results.iter().map(|r| r.eat_ratio).collect();
    let g1_refresh: Vec<f64> = g1_results.iter().map(|r| r.refresh_ratio).collect();
    let g2_refresh: Vec<f64> = g2_results.iter().map(|r| r.refresh_ratio).collect();
    let g1_divide: Vec<f64> = g1_results.iter().map(|r| r.divide_ratio).collect();
    let g2_divide: Vec<f64> = g2_results.iter().map(|r| r.divide_ratio).collect();
    let g1_pop: Vec<f64> = g1_results.iter().map(|r| r.avg_population).collect();
    let g2_pop: Vec<f64> = g2_results.iter().map(|r| r.avg_population).collect();

    let comp_eat = stats::compare_groups("EAT", &g1_eat, &g2_eat);
    let comp_refresh = stats::compare_groups("REFRESH", &g1_refresh, &g2_refresh);
    let comp_divide = stats::compare_groups("DIVIDE", &g1_divide, &g2_divide);
    let comp_pop = stats::compare_groups("Population", &g1_pop, &g2_pop);

    let mut report = String::new();
    report.push_str("# EXP-012: History Impact (D2 Experience Learning)\n\n");
    report.push_str("## Design\n\n");
    report.push_str("- Group 1: abundant food (500/tick) for first 10k ticks, then scarce (50/tick)\n");
    report.push_str("- Group 2: scarce food (50/tick) from start\n");
    report.push_str("- Same genomes (Seed F with Data cell), same seeds\n");
    report.push_str("- If history matters: Group 1 behavior differs from Group 2 after switch\n\n");

    report.push_str("## Results (10 seeds per group)\n\n");
    report.push_str("| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|----------|---------|-----------|------|----------|--------|\n");

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let surv = |r: &[cell_vm::CellSteadyState]| r.iter().filter(|x| x.survived).count();

    report.push_str(&format!(
        "| Abundant→Scarce | {}/10 | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
        surv(&g1_results), avg(&g1_pop), avg(&g1_results.iter().map(|r| r.avg_energy).collect::<Vec<_>>()),
        avg(&g1_eat) * 100.0, avg(&g1_refresh) * 100.0, avg(&g1_divide) * 100.0,
    ));
    report.push_str(&format!(
        "| Always Scarce | {}/10 | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
        surv(&g2_results), avg(&g2_pop), avg(&g2_results.iter().map(|r| r.avg_energy).collect::<Vec<_>>()),
        avg(&g2_eat) * 100.0, avg(&g2_refresh) * 100.0, avg(&g2_divide) * 100.0,
    ));

    report.push_str("\n## Statistical Tests\n\n");
    report.push_str("| Metric | Diff | 95% CI | MW p | d | KS D | KS p |\n");
    report.push_str("|--------|------|--------|------|---|------|------|\n");
    for c in [&comp_eat, &comp_refresh, &comp_divide, &comp_pop] {
        report.push_str(&format!(
            "| {} | {:.4} | [{:.4}, {:.4}] | {:.4} | {:.3} | {:.3} | {:.4} |\n",
            c.metric_name, c.mean_diff, c.ci_lower, c.ci_upper,
            c.p_value, c.cohens_d, c.ks_d, c.ks_p,
        ));
    }

    report.push_str("\n## Conclusion\n\n");
    if comp_eat.p_value < 0.05 || comp_refresh.p_value < 0.05 {
        report.push_str("History DOES affect behavior — organisms with abundant-then-scarce experience behave differently from always-scarce organisms.\n");
    } else {
        report.push_str("History does NOT measurably affect behavior in current design — Data cell contents do not significantly influence evolved strategies at this scale.\n");
    }

    report.push_str("\n---\n*EXP-012: History impact experiment*\n");

    let exp_dir = "D:/project/d0-vm/docs/experiments/EXP-012-history-impact";
    fs::write(format!("{}/experiment.md", exp_dir), &report).expect("Failed to write");

    // Per-seed CSV
    let mut csv = fs::File::create(format!("{}/data/per_seed.csv", exp_dir)).expect("CSV");
    writeln!(csv, "group,seed,survived,avg_pop,avg_energy,eat,refresh,divide").unwrap();
    for (i, &seed) in seeds.iter().enumerate() {
        let r = &g1_results[i];
        writeln!(csv, "abundant_scarce,{},{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.eat_ratio, r.refresh_ratio, r.divide_ratio).unwrap();
        let r = &g2_results[i];
        writeln!(csv, "always_scarce,{},{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.eat_ratio, r.refresh_ratio, r.divide_ratio).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-012-history-impact/experiment.md");
    println!("{}", report);
}

// ============================================================================
// EXP-014: GATE learning (Data cell as gene regulation switch)
// ============================================================================

fn run_gate_trial(seed: u64, abundant_first: bool) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.total_ticks = 1_000_000; // 1M ticks for longer evolution
    config.max_organisms = 200;
    config.data_cell_gating = true; // Enable GATE instruction
    config.snapshot_interval = 1000;
    config.genome_dump_interval = 0;

    if abundant_first {
        config.food_per_tick = 500;
    } else {
        config.food_per_tick = 50;
    }

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_g(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    let switch_tick: u64 = 10_000;

    for t in 0..config.total_ticks {
        if abundant_first && t == switch_tick {
            world.config.food_per_tick = 50;
        }
        world.tick();
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_exp014() {
    use rayon::prelude::*;

    eprintln!("EXP-014: GATE Learning (Data Cell Gene Regulation)");
    eprintln!("===================================================");
    eprintln!("  GATE instruction + Seed G (evaluation + gated DIVIDE)");
    eprintln!("  100 seeds x 2 groups, 1M ticks, rayon parallel\n");

    let seeds: Vec<u64> = (400..500).collect(); // 100 seeds
    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-014-gate-learning/data");

    eprintln!("Running Group 1 (abundant→scarce)...");
    let g1: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_gate_trial(s, true)).collect();

    eprintln!("Running Group 2 (always scarce)...");
    let g2: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_gate_trial(s, false)).collect();

    let g1_eat: Vec<f64> = g1.iter().map(|r| r.eat_ratio).collect();
    let g2_eat: Vec<f64> = g2.iter().map(|r| r.eat_ratio).collect();
    let g1_ref: Vec<f64> = g1.iter().map(|r| r.refresh_ratio).collect();
    let g2_ref: Vec<f64> = g2.iter().map(|r| r.refresh_ratio).collect();
    let g1_div: Vec<f64> = g1.iter().map(|r| r.divide_ratio).collect();
    let g2_div: Vec<f64> = g2.iter().map(|r| r.divide_ratio).collect();

    let comp_eat = stats::compare_groups("EAT", &g1_eat, &g2_eat);
    let comp_ref = stats::compare_groups("REFRESH", &g1_ref, &g2_ref);
    let comp_div = stats::compare_groups("DIVIDE", &g1_div, &g2_div);
    let g1_pop: Vec<f64> = g1.iter().map(|r| r.avg_population).collect();
    let g2_pop: Vec<f64> = g2.iter().map(|r| r.avg_population).collect();
    let comp_pop = stats::compare_groups("Population", &g1_pop, &g2_pop);

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let surv = |r: &[cell_vm::CellSteadyState]| r.iter().filter(|x| x.survived).count();

    let mut report = String::new();
    report.push_str("# EXP-014: GATE Learning (Data Cell Gene Regulation)\n\n");
    report.push_str("## Design\n\n");
    report.push_str("- GATE instruction: reads adjacent Data cell, if value==0 skips next Code cell\n");
    report.push_str("- Seed G: evaluation module (SENSE→EAT→SENSE→CMP→STORE) + GATE→DIVIDE\n");
    report.push_str("- Only divides when Data cell > 0 (= energy improved after eating)\n");
    report.push_str("- Group 1: abundant (500) first 10k ticks, then scarce (50)\n");
    report.push_str("- Group 2: always scarce (50)\n");
    report.push_str(&format!("- 1M ticks, {} seeds, CEM=50, data_cell_gating=true\n\n", seeds.len()));

    report.push_str("## Results\n\n");
    report.push_str("| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |\n");
    report.push_str("|-------|----------|---------|-----------|------|----------|--------|\n");

    let g1_energy: Vec<f64> = g1.iter().map(|r| r.avg_energy).collect();
    let g2_energy: Vec<f64> = g2.iter().map(|r| r.avg_energy).collect();
    // g1_pop, g2_pop already defined above for comp_pop

    let n = seeds.len();
    report.push_str(&format!(
        "| Abundant→Scarce | {}/{} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
        surv(&g1), n, avg(&g1_pop), avg(&g1_energy),
        avg(&g1_eat)*100.0, avg(&g1_ref)*100.0, avg(&g1_div)*100.0,
    ));
    report.push_str(&format!(
        "| Always Scarce | {}/{} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
        surv(&g2), n, avg(&g2_pop), avg(&g2_energy),
        avg(&g2_eat)*100.0, avg(&g2_ref)*100.0, avg(&g2_div)*100.0,
    ));

    report.push_str("\n## Statistical Tests\n\n");
    report.push_str("| Metric | Diff | 95% CI | MW p | d |\n");
    report.push_str("|--------|------|--------|------|---|\n");
    for c in [&comp_eat, &comp_ref, &comp_div, &comp_pop] {
        report.push_str(&format!(
            "| {} | {:.4} | [{:.4}, {:.4}] | {:.4} | {:.3} |\n",
            c.metric_name, c.mean_diff, c.ci_lower, c.ci_upper, c.p_value, c.cohens_d,
        ));
    }

    if comp_eat.p_value < 0.05 || comp_div.p_value < 0.05 {
        report.push_str("\n## Conclusion\n\nGATE mechanism produces measurable behavioral difference between history groups.\n");
    } else {
        report.push_str("\n## Conclusion\n\nGATE mechanism does not yet produce significant history-dependent behavior at this scale.\n");
    }

    report.push_str("\n---\n*EXP-014: GATE learning experiment*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-014-gate-learning";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/per_seed.csv", dir)).expect("csv");
    writeln!(csv, "group,seed,survived,pop,energy,eat,refresh,divide").unwrap();
    for (i, &seed) in seeds.iter().enumerate() {
        let r = &g1[i];
        writeln!(csv, "abundant_scarce,{},{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.eat_ratio, r.refresh_ratio, r.divide_ratio).unwrap();
        let r = &g2[i];
        writeln!(csv, "always_scarce,{},{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.eat_ratio, r.refresh_ratio, r.divide_ratio).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-014-gate-learning/experiment.md");
    println!("{}", report);
}
