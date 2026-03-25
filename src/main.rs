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

    // EXP-001~003 100-round replication: cargo run --release -- --replicate001
    if args.iter().any(|a| a == "--replicate001") {
        run_replication_exp001_003();
        return;
    }

    // EXP-009 100-round replication: cargo run --release -- --replicate009
    if args.iter().any(|a| a == "--replicate009") {
        run_replication_exp009();
        return;
    }

    // EXP-011 100-round replication: cargo run --release -- --replicate011
    if args.iter().any(|a| a == "--replicate011") {
        run_replication_exp011();
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
  ║    --replicate001   EXP-001~003 100 rounds  ║
  ║    --replicate009   EXP-009 100 rounds      ║
  ║    --replicate011   EXP-011 100 rounds      ║
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

    let num_rounds = 100;
    let seeds_per_round = 10;

    eprintln!("EXP-014: GATE Learning — 100 Independent Rounds");
    eprintln!("================================================");
    eprintln!("  {} rounds x {} seeds/round x 2 groups = {} total runs",
        num_rounds, seeds_per_round, num_rounds * seeds_per_round * 2);
    eprintln!("  Each round: independent seeds, 1M ticks, per-round p-value\n");

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-014-gate-learning/data");

    // Run all rounds in parallel (each round runs 10 seeds sequentially for both groups)
    struct RoundResult {
        round: usize,
        refresh_diff: f64,  // mean(g1_refresh) - mean(g2_refresh)
        eat_diff: f64,
        pop_diff: f64,
        refresh_p: f64,
        refresh_d: f64,
        g1_refresh_mean: f64,
        g2_refresh_mean: f64,
    }

    let rounds: Vec<usize> = (0..num_rounds).collect();
    let results: Vec<RoundResult> = rounds.par_iter().map(|&round| {
        let base_seed = (round as u64) * 1000 + 5000;
        let round_seeds: Vec<u64> = (base_seed..base_seed + seeds_per_round as u64).collect();

        let g1: Vec<cell_vm::CellSteadyState> = round_seeds.iter()
            .map(|&s| run_gate_trial(s, true)).collect();
        let g2: Vec<cell_vm::CellSteadyState> = round_seeds.iter()
            .map(|&s| run_gate_trial(s, false)).collect();

        let g1_ref: Vec<f64> = g1.iter().map(|r| r.refresh_ratio).collect();
        let g2_ref: Vec<f64> = g2.iter().map(|r| r.refresh_ratio).collect();
        let g1_eat: Vec<f64> = g1.iter().map(|r| r.eat_ratio).collect();
        let g2_eat: Vec<f64> = g2.iter().map(|r| r.eat_ratio).collect();
        let g1_pop: Vec<f64> = g1.iter().map(|r| r.avg_population).collect();
        let g2_pop: Vec<f64> = g2.iter().map(|r| r.avg_population).collect();

        let comp = stats::compare_groups("REFRESH", &g1_ref, &g2_ref);
        let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;

        if round % 10 == 0 {
            eprintln!("  Round {}/{} done", round + 1, num_rounds);
        }

        RoundResult {
            round,
            refresh_diff: avg(&g1_ref) - avg(&g2_ref),
            eat_diff: avg(&g1_eat) - avg(&g2_eat),
            pop_diff: avg(&g1_pop) - avg(&g2_pop),
            refresh_p: comp.p_value,
            refresh_d: comp.cohens_d,
            g1_refresh_mean: avg(&g1_ref),
            g2_refresh_mean: avg(&g2_ref),
        }
    }).collect();

    // Meta-analysis
    let refresh_positive = results.iter().filter(|r| r.refresh_diff > 0.0).count();
    let refresh_sig = results.iter().filter(|r| r.refresh_p < 0.05 && r.refresh_diff > 0.0).count();
    let pop_negative = results.iter().filter(|r| r.pop_diff < 0.0).count();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = avg(v);
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };

    let all_refresh_diffs: Vec<f64> = results.iter().map(|r| r.refresh_diff).collect();
    let all_refresh_d: Vec<f64> = results.iter().map(|r| r.refresh_d).collect();
    let all_eat_diffs: Vec<f64> = results.iter().map(|r| r.eat_diff).collect();
    let all_pop_diffs: Vec<f64> = results.iter().map(|r| r.pop_diff).collect();

    // Generate report
    let mut report = String::new();
    report.push_str("# EXP-014: GATE Learning — 100 Independent Rounds\n\n");
    report.push_str("## Design\n\n");
    report.push_str("- GATE instruction + Seed G (evaluation + gated DIVIDE)\n");
    report.push_str(&format!("- {} independent rounds, {} seeds per round, 1M ticks each\n", num_rounds, seeds_per_round));
    report.push_str("- Each round: independent seed set, independent population, per-round p-value\n\n");

    // Meta-analysis table
    let eat_positive = results.iter().filter(|r| r.eat_diff > 0.0).count();
    report.push_str("## Meta-Analysis\n\n");
    report.push_str("| Metric | Direction win | p<0.05 wins | Mean diff | SD | Win rate |\n");
    report.push_str("|--------|-------------|------------|-----------|-----|----------|\n");
    report.push_str(&format!(
        "| REFRESH | {}/{} | {}/{} | {:.4} | {:.4} | {:.0}% |\n",
        refresh_positive, num_rounds, refresh_sig, num_rounds,
        avg(&all_refresh_diffs), sd(&all_refresh_diffs),
        refresh_positive as f64 / num_rounds as f64 * 100.0,
    ));
    report.push_str(&format!(
        "| EAT | {}/{} | — | {:.4} | {:.4} | {:.0}% |\n",
        eat_positive, num_rounds,
        avg(&all_eat_diffs), sd(&all_eat_diffs),
        eat_positive as f64 / num_rounds as f64 * 100.0,
    ));
    report.push_str(&format!(
        "| Population | {}/{} | — | {:.1} | {:.1} | {:.0}% |\n",
        pop_negative, num_rounds,
        avg(&all_pop_diffs), sd(&all_pop_diffs),
        pop_negative as f64 / num_rounds as f64 * 100.0,
    ));

    // Effect size distribution
    report.push_str(&format!("\n## Effect Size Distribution (REFRESH Cohen's d across rounds)\n\n"));
    report.push_str(&format!("- Mean d: {:.3}\n", avg(&all_refresh_d)));
    report.push_str(&format!("- SD d: {:.3}\n", sd(&all_refresh_d)));
    let positive_d = all_refresh_d.iter().filter(|&&d| d > 0.0).count();
    report.push_str(&format!("- Positive d: {}/{} ({:.0}%)\n", positive_d, num_rounds, positive_d as f64 / num_rounds as f64 * 100.0));

    report.push_str("\n## Conclusion\n\n");
    report.push_str(&format!(
        "Across {} truly independent experiments (each with {} seeds, independent initialization, 1M ticks):\n",
        num_rounds, seeds_per_round
    ));
    report.push_str(&format!(
        "- REFRESH effect in predicted direction: **{}%** of rounds\n", refresh_positive * 100 / num_rounds
    ));
    report.push_str(&format!(
        "- REFRESH p<0.05 in predicted direction: **{}%** of rounds\n", refresh_sig * 100 / num_rounds
    ));
    report.push_str(&format!(
        "- Population effect: **{}%** of rounds\n", pop_negative * 100 / num_rounds
    ));

    report.push_str("\n---\n*EXP-014: 100 independent rounds meta-analysis*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-014-gate-learning";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    // Per-round CSV
    let mut csv = fs::File::create(format!("{}/data/per_round.csv", dir)).expect("csv");
    writeln!(csv, "round,refresh_diff,eat_diff,pop_diff,refresh_p,refresh_d,g1_refresh,g2_refresh").unwrap();
    for r in &results {
        writeln!(csv, "{},{:.6},{:.6},{:.2},{:.6},{:.4},{:.6},{:.6}",
            r.round, r.refresh_diff, r.eat_diff, r.pop_diff,
            r.refresh_p, r.refresh_d, r.g1_refresh_mean, r.g2_refresh_mean).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-014-gate-learning/experiment.md");
    println!("{}", report);
}

// ============================================================================
// EXP-001~003: 100-Round Independent Replication
// D0 Operational Closure — freshness_decay (true vs false)
// CEM=50, R=5, 500k ticks
// ============================================================================

fn run_replication_exp001_003() {
    use rayon::prelude::*;

    let num_rounds = 100usize;
    let seeds_per_round = 5usize; // 5 seeds per round, both groups

    eprintln!("EXP-001~003: 100-Round Independent Replication (Operational Closure)");
    eprintln!("======================================================================");
    eprintln!("  {} rounds x {} seeds/round x 2 groups = {} total runs",
        num_rounds, seeds_per_round, num_rounds * seeds_per_round * 2);
    eprintln!("  Config: Cell v3, CEM=50, R=5, 500k ticks\n");

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-001-replication/data");

    struct RoundResult {
        round: usize,
        exp_refresh: f64,
        ctrl_refresh: f64,
        refresh_diff: f64,
        eat_diff: f64,
        pop_diff: f64,
        refresh_p: f64,
        refresh_d: f64,
        exp_survived: usize,
        ctrl_survived: usize,
    }

    let rounds: Vec<usize> = (0..num_rounds).collect();

    let results: Vec<RoundResult> = rounds.par_iter().map(|&round| {
        let base_seed = (round as u64) * 100 + 10_000;
        let round_seeds: Vec<u64> = (base_seed..base_seed + seeds_per_round as u64).collect();

        let mut exp_refreshes = Vec::new();
        let mut ctrl_refreshes = Vec::new();
        let mut exp_eats = Vec::new();
        let mut ctrl_eats = Vec::new();
        let mut exp_pops = Vec::new();
        let mut ctrl_pops = Vec::new();
        let mut exp_survived = 0usize;
        let mut ctrl_survived = 0usize;

        for &seed in &round_seeds {
            // Experimental: freshness_decay = true
            let mut exp_config = CellConfig::experimental();
            exp_config.cell_energy_max = 50;
            exp_config.refresh_radius = 5;
            exp_config.total_ticks = 500_000;
            exp_config.snapshot_interval = 5_000;
            exp_config.genome_dump_interval = 0;

            let mut exp_world = CellWorld::new(exp_config.clone(), seed);
            for _ in 0..10 { exp_world.add_organism(cell_seed_a(&exp_config)); }
            for _ in 0..10 { exp_world.add_organism(cell_seed_b(&exp_config)); }
            exp_world.run();
            let exp_ss = cell_compute_steady_state(&exp_world.snapshots);

            if exp_ss.survived { exp_survived += 1; }
            exp_refreshes.push(exp_ss.refresh_ratio);
            exp_eats.push(exp_ss.eat_ratio);
            exp_pops.push(exp_ss.avg_population);

            // Control: freshness_decay = false
            let mut ctrl_config = CellConfig::control();
            ctrl_config.cell_energy_max = 50;
            ctrl_config.refresh_radius = 5;
            ctrl_config.total_ticks = 500_000;
            ctrl_config.snapshot_interval = 5_000;
            ctrl_config.genome_dump_interval = 0;

            let mut ctrl_world = CellWorld::new(ctrl_config.clone(), seed);
            for _ in 0..10 { ctrl_world.add_organism(cell_seed_a(&ctrl_config)); }
            for _ in 0..10 { ctrl_world.add_organism(cell_seed_b(&ctrl_config)); }
            ctrl_world.run();
            let ctrl_ss = cell_compute_steady_state(&ctrl_world.snapshots);

            if ctrl_ss.survived { ctrl_survived += 1; }
            ctrl_refreshes.push(ctrl_ss.refresh_ratio);
            ctrl_eats.push(ctrl_ss.eat_ratio);
            ctrl_pops.push(ctrl_ss.avg_population);
        }

        let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let exp_refresh_mean = avg(&exp_refreshes);
        let ctrl_refresh_mean = avg(&ctrl_refreshes);

        let comp = stats::compare_groups("REFRESH", &exp_refreshes, &ctrl_refreshes);

        if round % 10 == 0 {
            eprintln!("  Round {}/{}: exp_refresh={:.4} ctrl_refresh={:.4} diff={:.4} p={:.4}",
                round + 1, num_rounds,
                exp_refresh_mean, ctrl_refresh_mean,
                exp_refresh_mean - ctrl_refresh_mean,
                comp.p_value);
        }

        RoundResult {
            round,
            exp_refresh: exp_refresh_mean,
            ctrl_refresh: ctrl_refresh_mean,
            refresh_diff: exp_refresh_mean - ctrl_refresh_mean,
            eat_diff: avg(&exp_eats) - avg(&ctrl_eats),
            pop_diff: avg(&exp_pops) - avg(&ctrl_pops),
            refresh_p: comp.p_value,
            refresh_d: comp.cohens_d,
            exp_survived,
            ctrl_survived,
        }
    }).collect();

    // Meta-analysis
    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = avg(v);
        ((v.iter().map(|x| (x - m).powi(2)).sum::<f64>()) / (v.len() - 1) as f64).sqrt()
    };
    // 95% CI on mean_diff using t-interval (n=100)
    let ci95 = |v: &[f64]| -> (f64, f64) {
        let m = avg(v);
        let s = sd(v);
        let se = s / (v.len() as f64).sqrt();
        (m - 1.984 * se, m + 1.984 * se) // t_{99, 0.025} ≈ 1.984
    };

    let all_refresh_diffs: Vec<f64> = results.iter().map(|r| r.refresh_diff).collect();
    let all_refresh_d: Vec<f64> = results.iter().map(|r| r.refresh_d).collect();
    let all_eat_diffs: Vec<f64> = results.iter().map(|r| r.eat_diff).collect();
    let all_pop_diffs: Vec<f64> = results.iter().map(|r| r.pop_diff).collect();

    // Direction consistency: how many rounds have exp_refresh > ctrl_refresh?
    let refresh_positive = results.iter().filter(|r| r.refresh_diff > 0.0).count();
    // Significant positive
    let refresh_sig_pos = results.iter().filter(|r| r.refresh_diff > 0.0 && r.refresh_p < 0.05).count();
    // Significant negative (falsification attempts)
    let refresh_sig_neg = results.iter().filter(|r| r.refresh_diff < 0.0 && r.refresh_p < 0.05).count();
    let pop_negative = results.iter().filter(|r| r.pop_diff < 0.0).count();

    // Sign test p-value (binomial approximation): k successes in n=100 trials, H0: p=0.5
    // Normal approximation: z = (k - 50) / sqrt(25)
    let sign_z = (refresh_positive as f64 - 50.0) / 5.0;
    // P(Z > |z|) * 2 using erfc approximation
    let sign_p = {
        let z = sign_z.abs();
        // erfc approximation for p-value
        let t = 1.0 / (1.0 + 0.3275911 * z);
        let poly = t * (0.254829592
            + t * (-0.284496736
            + t * (1.421413741
            + t * (-1.453152027
            + t * 1.061405429))));
        let p_one = 0.3989422804 * (-z * z / 2.0).exp() * poly;
        2.0 * p_one
    };

    let (diff_ci_lo, diff_ci_hi) = ci95(&all_refresh_diffs);

    // Stouffer's Z for combined p-values (Fisher's method alternative)
    // Use sign test as primary meta-p
    let effect_mean = avg(&all_refresh_diffs);
    let effect_sd = sd(&all_refresh_diffs);

    let mut report = String::new();
    report.push_str("# EXP-001~003: 100-Round Independent Replication\n\n");
    report.push_str("## Hypothesis\n\n");
    report.push_str("Freshness decay (operational closure constraint) drives organisms to execute REFRESH more frequently.\n");
    report.push_str("Prediction: exp (freshness_decay=true) REFRESH ratio > ctrl (freshness_decay=false) in majority of independent replication rounds.\n\n");

    report.push_str("## Parameters\n\n");
    report.push_str("| Parameter | Value |\n");
    report.push_str("|-----------|-------|\n");
    report.push_str(&format!("| VM | Cell v3 (per-cell freshness) |\n"));
    report.push_str(&format!("| CEM | 50 |\n"));
    report.push_str(&format!("| R (refresh radius) | 5 |\n"));
    report.push_str(&format!("| Ticks | 500,000 |\n"));
    report.push_str(&format!("| Seeds per round | {} |\n", seeds_per_round));
    report.push_str(&format!("| Rounds | {} |\n", num_rounds));
    report.push_str(&format!("| Total runs | {} |\n", num_rounds * seeds_per_round * 2));
    report.push_str(&format!("| Initial organisms | 10 Seed A + 10 Seed B |\n"));
    report.push_str(&format!("| Seed range | 10000–{} (round*100+10000) |\n", (num_rounds - 1) * 100 + 10000 + seeds_per_round as usize - 1));
    report.push_str("\n");

    report.push_str("## Meta-Analysis Summary\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!("| Direction win (exp_REFRESH > ctrl_REFRESH) | **{}/{}** ({:.0}%) |\n",
        refresh_positive, num_rounds, refresh_positive as f64 / num_rounds as f64 * 100.0));
    report.push_str(&format!("| Sign test p-value | **{:.6}** (H0: p=0.5) |\n", sign_p));
    report.push_str(&format!("| Sig. wins (p<0.05 AND diff>0) | {}/{} ({:.0}%) |\n",
        refresh_sig_pos, num_rounds, refresh_sig_pos as f64 / num_rounds as f64 * 100.0));
    report.push_str(&format!("| Sig. losses (p<0.05 AND diff<0) | {}/{} ({:.0}%) |\n",
        refresh_sig_neg, num_rounds, refresh_sig_neg as f64 / num_rounds as f64 * 100.0));
    report.push_str(&format!("| Mean REFRESH diff (exp-ctrl) | **{:.4}** |\n", effect_mean));
    report.push_str(&format!("| SD REFRESH diff | {:.4} |\n", effect_sd));
    report.push_str(&format!("| 95% CI on mean diff | [{:.4}, {:.4}] |\n", diff_ci_lo, diff_ci_hi));
    report.push_str(&format!("| Mean Cohen's d | {:.3} |\n", avg(&all_refresh_d)));
    report.push_str(&format!("| SD Cohen's d | {:.3} |\n", sd(&all_refresh_d)));
    report.push_str(&format!("| Population effect (exp_pop < ctrl_pop) | {}/{} ({:.0}%) |\n",
        pop_negative, num_rounds, pop_negative as f64 / num_rounds as f64 * 100.0));
    report.push_str("\n");

    // Per-round table (all 100)
    report.push_str("## Per-Round Results (100 rounds)\n\n");
    report.push_str("| Round | Exp REFRESH | Ctrl REFRESH | Diff | Direction | p | d |\n");
    report.push_str("|-------|------------|-------------|------|-----------|---|---|\n");
    for r in &results {
        let dir = if r.refresh_diff > 0.0 { "+" } else { "-" };
        let sig = if r.refresh_p < 0.001 { "***" } else if r.refresh_p < 0.01 { "**" } else if r.refresh_p < 0.05 { "*" } else { "n.s." };
        report.push_str(&format!("| {} | {:.4} | {:.4} | {:+.4} | {} | {:.4} {} | {:.3} |\n",
            r.round + 1, r.exp_refresh, r.ctrl_refresh, r.refresh_diff,
            dir, r.refresh_p, sig, r.refresh_d));
    }
    report.push_str("\n");

    // Conclusion
    report.push_str("## Conclusion\n\n");
    let replication_rate = refresh_positive as f64 / num_rounds as f64;
    report.push_str(&format!(
        "Across {} truly independent rounds (each with {} fresh seeds, independent initialization):\n\n",
        num_rounds, seeds_per_round
    ));
    report.push_str(&format!(
        "- **Replication rate**: {:.0}% ({}/{}) rounds show exp_REFRESH > ctrl_REFRESH\n",
        replication_rate * 100.0, refresh_positive, num_rounds
    ));
    report.push_str(&format!(
        "- **Sign test**: p = {:.6} ({})\n",
        sign_p,
        if sign_p < 0.001 { "*** highly significant" }
        else if sign_p < 0.01 { "** significant" }
        else if sign_p < 0.05 { "* significant" }
        else { "not significant" }
    ));
    report.push_str(&format!(
        "- **Effect size**: mean d = {:.3} ({})\n",
        avg(&all_refresh_d),
        if avg(&all_refresh_d).abs() >= 0.8 { "large" }
        else if avg(&all_refresh_d).abs() >= 0.5 { "medium" }
        else if avg(&all_refresh_d).abs() >= 0.2 { "small" }
        else { "negligible" }
    ));
    if replication_rate >= 0.8 {
        report.push_str("\n**REPLICATED**: The operational closure constraint reliably drives increased REFRESH behavior across independent trials.\n");
    } else if replication_rate >= 0.6 {
        report.push_str("\n**PARTIALLY REPLICATED**: Effect is present but inconsistent across independent trials.\n");
    } else {
        report.push_str("\n**NOT REPLICATED**: Effect does not consistently appear across independent trials.\n");
    }

    report.push_str("\n---\n*EXP-001~003: 100-round independent replication — operational closure core test*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-001-replication";
    fs::write(format!("{}/replication_100rounds.md", dir), &report).expect("write");

    // Per-round CSV
    let mut csv = fs::File::create(format!("{}/data/rounds_data.csv", dir)).expect("csv");
    writeln!(csv, "round,base_seed,exp_refresh,ctrl_refresh,refresh_diff,eat_diff,pop_diff,refresh_p,refresh_d,exp_survived,ctrl_survived").unwrap();
    for r in &results {
        let base_seed = (r.round as u64) * 100 + 10_000;
        writeln!(csv, "{},{},{:.6},{:.6},{:.6},{:.6},{:.2},{:.6},{:.4},{},{}",
            r.round + 1, base_seed,
            r.exp_refresh, r.ctrl_refresh, r.refresh_diff, r.eat_diff, r.pop_diff,
            r.refresh_p, r.refresh_d, r.exp_survived, r.ctrl_survived).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-001-replication/replication_100rounds.md");
    eprintln!("CSV: docs/experiments/EXP-001-replication/data/rounds_data.csv");
    println!("{}", report);
}

// ============================================================================
// EXP-009: 100-Round Independent Replication
// Real CPU接入 — CPU availability → food modulation
// ============================================================================

fn run_replication_exp009() {
    use rayon::prelude::*;

    let num_rounds = 100usize;

    eprintln!("EXP-009: 100-Round Independent Replication (CPU-modulated food)");
    eprintln!("==================================================================");
    eprintln!("  {} rounds: CPU-food (hash-based) vs constant-food, 500k ticks\n", num_rounds);

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-009-replication/data");

    // One round of real-cpu experiment: food modulated by a hash-based CPU signal
    // (replaces sysinfo which is not stable in parallel; uses per-round deterministic hash)
    struct Exp009Round {
        round: usize,
        cpu_eat: f64,
        const_eat: f64,
        cpu_refresh: f64,
        const_refresh: f64,
        cpu_pop: f64,
        const_pop: f64,
        eat_diff: f64,  // cpu_eat - const_eat
        survived_cpu: bool,
        survived_const: bool,
    }

    let rounds: Vec<usize> = (0..num_rounds).collect();

    let results: Vec<Exp009Round> = rounds.par_iter().map(|&round| {
        let seed = (round as u64) * 137 + 20_000;

        let base_food: i32 = 300;
        let total_ticks: u64 = 500_000;
        let cpu_sample_interval: u64 = 100;

        // CPU-modulated world
        let mut cpu_config = CellConfig::experimental();
        cpu_config.cell_energy_max = 50;
        cpu_config.food_per_tick = 0; // Manual food injection
        cpu_config.total_ticks = total_ticks;
        cpu_config.snapshot_interval = 5_000;
        cpu_config.genome_dump_interval = 0;

        let mut cpu_world = CellWorld::new(cpu_config.clone(), seed);
        for _ in 0..10 { cpu_world.add_organism(cell_seed_a(&cpu_config)); }
        for _ in 0..10 { cpu_world.add_organism(cell_seed_b(&cpu_config)); }

        // Constant-food world (same average food as CPU world)
        let avg_available: f32 = 0.65; // expected average of floor(0.3 + 0.7*(1-u)) where u~U[0,1]
        let const_food_per_interval = (base_food as f32 * avg_available) as i32;
        let mut const_config = CellConfig::experimental();
        const_config.cell_energy_max = 50;
        const_config.food_per_tick = 0;
        const_config.total_ticks = total_ticks;
        const_config.snapshot_interval = 5_000;
        const_config.genome_dump_interval = 0;

        let mut const_world = CellWorld::new(const_config.clone(), seed);
        for _ in 0..10 { const_world.add_organism(cell_seed_a(&const_config)); }
        for _ in 0..10 { const_world.add_organism(cell_seed_b(&const_config)); }

        // Run tick-by-tick with food injection
        for t in 0..total_ticks {
            if t % cpu_sample_interval == 0 {
                // Hash-based pseudo-CPU usage (deterministic, seed-dependent)
                let block = t / cpu_sample_interval;
                let hash = block.wrapping_mul(6364136223846793005u64)
                    .wrapping_add(seed.wrapping_mul(1442695040888963407u64))
                    >> 33;
                let cpu_usage = (hash % 100) as f32 / 100.0;
                let available = 0.3 + 0.7 * (1.0 - cpu_usage).max(0.0);
                let food = (base_food as f32 * available) as i32;
                cpu_world.food_pool += food;
                const_world.food_pool += const_food_per_interval;
            }
            cpu_world.tick();
            const_world.tick();
        }

        cpu_world.take_snapshot();
        const_world.take_snapshot();
        let cpu_ss = cell_compute_steady_state(&cpu_world.snapshots);
        let const_ss = cell_compute_steady_state(&const_world.snapshots);

        if round % 10 == 0 {
            eprintln!("  Round {}/{}: cpu_eat={:.4} const_eat={:.4}",
                round + 1, num_rounds, cpu_ss.eat_ratio, const_ss.eat_ratio);
        }

        Exp009Round {
            round,
            cpu_eat: cpu_ss.eat_ratio,
            const_eat: const_ss.eat_ratio,
            cpu_refresh: cpu_ss.refresh_ratio,
            const_refresh: const_ss.refresh_ratio,
            cpu_pop: cpu_ss.avg_population,
            const_pop: const_ss.avg_population,
            eat_diff: cpu_ss.eat_ratio - const_ss.eat_ratio,
            survived_cpu: cpu_ss.survived,
            survived_const: const_ss.survived,
        }
    }).collect();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = avg(v);
        ((v.iter().map(|x| (x - m).powi(2)).sum::<f64>()) / (v.len() - 1) as f64).sqrt()
    };
    let ci95 = |v: &[f64]| -> (f64, f64) {
        let m = avg(v);
        let s = sd(v);
        let se = s / (v.len() as f64).sqrt();
        (m - 1.984 * se, m + 1.984 * se)
    };

    let all_eat_diffs: Vec<f64> = results.iter().map(|r| r.eat_diff).collect();
    let eat_positive = results.iter().filter(|r| r.eat_diff > 0.0).count();
    let eat_positive_diff_pos = results.iter().filter(|r| r.eat_diff > 0.0).count();
    let survived_cpu = results.iter().filter(|r| r.survived_cpu).count();
    let survived_const = results.iter().filter(|r| r.survived_const).count();

    let (diff_ci_lo, diff_ci_hi) = ci95(&all_eat_diffs);

    // Sign test p
    let sign_z = (eat_positive as f64 - 50.0) / 5.0;
    let sign_p = {
        let z = sign_z.abs();
        let t = 1.0 / (1.0 + 0.3275911 * z);
        let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        2.0 * 0.3989422804 * (-z * z / 2.0).exp() * poly
    };

    let all_cpu_eats: Vec<f64> = results.iter().map(|r| r.cpu_eat).collect();
    let all_const_eats: Vec<f64> = results.iter().map(|r| r.const_eat).collect();
    let comp = stats::compare_groups("EAT", &all_cpu_eats, &all_const_eats);

    let mut report = String::new();
    report.push_str("# EXP-009: 100-Round Independent Replication (CPU-modulated food)\n\n");
    report.push_str("## Hypothesis\n\n");
    report.push_str("CPU-modulated food supply (variable, correlated with real workload) drives higher EAT rate compared to constant equivalent food supply, as organisms must adapt to temporal scarcity.\n\n");

    report.push_str("## Parameters\n\n");
    report.push_str("| Parameter | Value |\n");
    report.push_str("|-----------|-------|\n");
    report.push_str("| VM | Cell v3, CEM=50, R=5 |\n");
    report.push_str("| Ticks | 500,000 |\n");
    report.push_str(&format!("| Rounds | {} |\n", num_rounds));
    report.push_str("| CPU signal | Hash-based pseudo (deterministic per seed) |\n");
    report.push_str("| Base food | 300 units per 100-tick sample |\n");
    report.push_str("| CPU floor | 30% (food = base × (0.3 + 0.7×(1-cpu))) |\n");
    report.push_str("| Constant food | Same expected value (65% of base) |\n");
    report.push_str("\n");

    report.push_str("## Meta-Analysis Summary\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!("| Survived (CPU) | {}/{} |\n", survived_cpu, num_rounds));
    report.push_str(&format!("| Survived (Const) | {}/{} |\n", survived_const, num_rounds));
    report.push_str(&format!("| EAT diff > 0 (cpu > const) | **{}/{}** ({:.0}%) |\n",
        eat_positive_diff_pos, num_rounds, eat_positive_diff_pos as f64 / num_rounds as f64 * 100.0));
    report.push_str(&format!("| Sign test p | {:.6} |\n", sign_p));
    report.push_str(&format!("| Mean EAT diff (cpu-const) | **{:.4}** |\n", avg(&all_eat_diffs)));
    report.push_str(&format!("| SD EAT diff | {:.4} |\n", sd(&all_eat_diffs)));
    report.push_str(&format!("| 95% CI on mean diff | [{:.4}, {:.4}] |\n", diff_ci_lo, diff_ci_hi));
    report.push_str(&format!("| MW p (pooled 100 rounds) | {:.6} |\n", comp.p_value));
    report.push_str(&format!("| Cohen's d | {:.3} |\n", comp.cohens_d));
    report.push_str("\n");

    report.push_str("## Per-Round Results (100 rounds)\n\n");
    report.push_str("| Round | CPU EAT | Const EAT | Diff | CPU Pop | Const Pop | CPU Surv | Const Surv |\n");
    report.push_str("|-------|---------|----------|------|---------|----------|----------|------------|\n");
    for r in &results {
        let dir = if r.eat_diff > 0.0 { "+" } else { "-" };
        report.push_str(&format!("| {} | {:.4} | {:.4} | {}{:.4} | {:.1} | {:.1} | {} | {} |\n",
            r.round + 1,
            r.cpu_eat, r.const_eat, dir, r.eat_diff.abs(),
            r.cpu_pop, r.const_pop,
            if r.survived_cpu { "YES" } else { "NO" },
            if r.survived_const { "YES" } else { "NO" }));
    }
    report.push_str("\n");

    report.push_str("## Conclusion\n\n");
    report.push_str(&format!(
        "Across {} independent rounds (different seeds, independent initialization):\n\n",
        num_rounds
    ));
    report.push_str(&format!(
        "- CPU-variable food leads to **{:.0}%** of rounds where cpu_EAT > const_EAT\n",
        eat_positive_diff_pos as f64 / num_rounds as f64 * 100.0
    ));
    report.push_str(&format!("- Sign test p = {:.6}\n", sign_p));

    report.push_str("\n---\n*EXP-009: 100-round replication — CPU-modulated food EAT adaptation*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-009-replication";
    fs::write(format!("{}/replication_100rounds.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/rounds_data.csv", dir)).expect("csv");
    writeln!(csv, "round,seed,cpu_eat,const_eat,eat_diff,cpu_refresh,const_refresh,cpu_pop,const_pop,survived_cpu,survived_const").unwrap();
    for r in &results {
        let seed = (r.round as u64) * 137 + 20_000;
        writeln!(csv, "{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.2},{:.2},{},{}",
            r.round + 1, seed,
            r.cpu_eat, r.const_eat, r.eat_diff,
            r.cpu_refresh, r.const_refresh,
            r.cpu_pop, r.const_pop,
            r.survived_cpu, r.survived_const).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-009-replication/replication_100rounds.md");
    println!("{}", report);
}

// ============================================================================
// EXP-011: 100-Round Independent Replication (Signal Sensitivity)
// ============================================================================

fn run_replication_exp011() {
    use rayon::prelude::*;

    let num_rounds = 100usize;

    eprintln!("EXP-011: 100-Round Independent Replication (Signal Sensitivity)");
    eprintln!("==================================================================");
    eprintln!("  {} rounds: predictable signal (A) vs random (E) vs no signal (F)\n", num_rounds);

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-011-replication/data");

    // Each round: 1 seed, three conditions
    struct Exp011Round {
        round: usize,
        // Group A: square wave, delta=200 (predictable)
        a_eat: f64,
        // Group E: random signal, delta=200 (unpredictable)
        e_eat: f64,
        // Group F: no signal
        f_eat: f64,
        a_pop: f64,
        e_pop: f64,
        f_pop: f64,
        a_survived: bool,
        e_survived: bool,
        f_survived: bool,
        // EAT diff: A vs F (predictable signal advantage over no-signal)
        ae_diff: f64,
        af_diff: f64,
    }

    let rounds: Vec<usize> = (0..num_rounds).collect();

    let results: Vec<Exp011Round> = rounds.par_iter().map(|&round| {
        let seed = (round as u64) * 77 + 30_000;

        let group_a = signal::SenseMakingGroup::group_a(); // square d=200
        let group_e = signal::SenseMakingGroup::group_e(); // random d=200
        let group_f = signal::SenseMakingGroup::group_f(); // no signal

        let ss_a = run_sensemaking_trial(&group_a, seed);
        let ss_e = run_sensemaking_trial(&group_e, seed);
        let ss_f = run_sensemaking_trial(&group_f, seed);

        if round % 10 == 0 {
            eprintln!("  Round {}/{}: A_eat={:.4} E_eat={:.4} F_eat={:.4}",
                round + 1, num_rounds, ss_a.eat_ratio, ss_e.eat_ratio, ss_f.eat_ratio);
        }

        let ae_diff = ss_a.eat_ratio - ss_e.eat_ratio;
        let af_diff = ss_a.eat_ratio - ss_f.eat_ratio;

        Exp011Round {
            round,
            a_eat: ss_a.eat_ratio,
            e_eat: ss_e.eat_ratio,
            f_eat: ss_f.eat_ratio,
            a_pop: ss_a.avg_population,
            e_pop: ss_e.avg_population,
            f_pop: ss_f.avg_population,
            a_survived: ss_a.survived,
            e_survived: ss_e.survived,
            f_survived: ss_f.survived,
            ae_diff,
            af_diff,
        }
    }).collect();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = avg(v);
        ((v.iter().map(|x| (x - m).powi(2)).sum::<f64>()) / (v.len() - 1) as f64).sqrt()
    };
    let ci95 = |v: &[f64]| -> (f64, f64) {
        let m = avg(v);
        let s = sd(v);
        let se = s / (v.len() as f64).sqrt();
        (m - 1.984 * se, m + 1.984 * se)
    };

    let all_ae_diffs: Vec<f64> = results.iter().map(|r| r.ae_diff).collect();
    let all_af_diffs: Vec<f64> = results.iter().map(|r| r.af_diff).collect();
    let all_a_eats: Vec<f64> = results.iter().map(|r| r.a_eat).collect();
    let all_e_eats: Vec<f64> = results.iter().map(|r| r.e_eat).collect();
    let all_f_eats: Vec<f64> = results.iter().map(|r| r.f_eat).collect();

    let ae_positive = results.iter().filter(|r| r.ae_diff > 0.0).count();
    let af_positive = results.iter().filter(|r| r.af_diff > 0.0).count();

    let (ae_ci_lo, ae_ci_hi) = ci95(&all_ae_diffs);
    let (af_ci_lo, af_ci_hi) = ci95(&all_af_diffs);

    // Sign test p for A vs E
    let sign_z_ae = (ae_positive as f64 - 50.0) / 5.0;
    let sign_p_ae = {
        let z = sign_z_ae.abs();
        let t = 1.0 / (1.0 + 0.3275911 * z);
        let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        2.0 * 0.3989422804 * (-z * z / 2.0).exp() * poly
    };
    let sign_z_af = (af_positive as f64 - 50.0) / 5.0;
    let sign_p_af = {
        let z = sign_z_af.abs();
        let t = 1.0 / (1.0 + 0.3275911 * z);
        let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        2.0 * 0.3989422804 * (-z * z / 2.0).exp() * poly
    };

    let comp_ae = stats::compare_groups("A vs E EAT", &all_a_eats, &all_e_eats);
    let comp_af = stats::compare_groups("A vs F EAT", &all_a_eats, &all_f_eats);

    let mut report = String::new();
    report.push_str("# EXP-011: 100-Round Independent Replication (Signal Sensitivity)\n\n");
    report.push_str("## Hypothesis\n\n");
    report.push_str("Organisms exposed to predictable signals (Group A: square wave, delta=200) should exhibit higher EAT rates than those with random signals (Group E) or no signal (Group F), demonstrating signal-sensitive resource acquisition.\n\n");

    report.push_str("## Parameters\n\n");
    report.push_str("| Parameter | Value |\n");
    report.push_str("|-----------|-------|\n");
    report.push_str("| VM | Cell v3, CEM=50, R=5 |\n");
    report.push_str("| Ticks | 500,000 |\n");
    report.push_str(&format!("| Rounds | {} |\n", num_rounds));
    report.push_str("| Group A | Square wave (period=2000), delta=200 |\n");
    report.push_str("| Group E | Random signal, delta=200 |\n");
    report.push_str("| Group F | No signal (constant 0.5) |\n");
    report.push_str("| Seed F organisms | SAMPLE → STORE → EAT → DIGEST → LOAD → CMP |\n");
    report.push_str("\n");

    report.push_str("## Meta-Analysis Summary\n\n");
    report.push_str("| Comparison | Direction win | Sign p | Mean diff | 95% CI | MW p | d |\n");
    report.push_str("|-----------|--------------|--------|-----------|--------|------|---|\n");
    report.push_str(&format!("| A vs E (predict vs random) | {}/{} ({:.0}%) | {:.6} | {:.4} | [{:.4},{:.4}] | {:.6} | {:.3} |\n",
        ae_positive, num_rounds, ae_positive as f64 / num_rounds as f64 * 100.0,
        sign_p_ae, avg(&all_ae_diffs), ae_ci_lo, ae_ci_hi,
        comp_ae.p_value, comp_ae.cohens_d));
    report.push_str(&format!("| A vs F (predict vs no-signal) | {}/{} ({:.0}%) | {:.6} | {:.4} | [{:.4},{:.4}] | {:.6} | {:.3} |\n",
        af_positive, num_rounds, af_positive as f64 / num_rounds as f64 * 100.0,
        sign_p_af, avg(&all_af_diffs), af_ci_lo, af_ci_hi,
        comp_af.p_value, comp_af.cohens_d));
    report.push_str("\n");

    report.push_str("## Per-Round Results (100 rounds)\n\n");
    report.push_str("| Round | A EAT | E EAT | F EAT | A-E diff | A-F diff |\n");
    report.push_str("|-------|-------|-------|-------|---------|----------|\n");
    for r in &results {
        report.push_str(&format!("| {} | {:.4} | {:.4} | {:.4} | {:+.4} | {:+.4} |\n",
            r.round + 1, r.a_eat, r.e_eat, r.f_eat, r.ae_diff, r.af_diff));
    }
    report.push_str("\n");

    report.push_str("## Conclusion\n\n");
    report.push_str(&format!(
        "Across {} independent rounds:\n\n", num_rounds
    ));
    report.push_str(&format!(
        "- **A vs E**: {:.0}% rounds A_EAT > E_EAT, sign p = {:.6}\n",
        ae_positive as f64 / num_rounds as f64 * 100.0, sign_p_ae
    ));
    report.push_str(&format!(
        "- **A vs F**: {:.0}% rounds A_EAT > F_EAT, sign p = {:.6}\n",
        af_positive as f64 / num_rounds as f64 * 100.0, sign_p_af
    ));

    report.push_str("\n---\n*EXP-011: 100-round replication — signal sensitivity EAT adaptation*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-011-replication";
    fs::write(format!("{}/replication_100rounds.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/rounds_data.csv", dir)).expect("csv");
    writeln!(csv, "round,seed,a_eat,e_eat,f_eat,ae_diff,af_diff,a_pop,e_pop,f_pop,a_surv,e_surv,f_surv").unwrap();
    for r in &results {
        let seed = (r.round as u64) * 77 + 30_000;
        writeln!(csv, "{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.2},{:.2},{:.2},{},{},{}",
            r.round + 1, seed,
            r.a_eat, r.e_eat, r.f_eat, r.ae_diff, r.af_diff,
            r.a_pop, r.e_pop, r.f_pop,
            r.a_survived, r.e_survived, r.f_survived).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-011-replication/replication_100rounds.md");
    println!("{}", report);
}
