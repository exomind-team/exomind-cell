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
mod lineage014;
mod knockout;

use std::fs;
use std::io::Write as IoWrite;
use organism::Config;
use experiment::{run_experiment, analyze_and_report, compute_steady_state, SteadyState};
use cell_vm::{CellConfig, CellWorld, cell_seed_a, cell_seed_b, cell_seed_f, cell_seed_g, cell_compute_steady_state, run_cell_experiment, run_cell_data_experiment, run_cell_growth_experiment};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Thread limit: --threads N (default 12, leaves headroom for system)
    let num_threads = args.iter().position(|a| a == "--threads")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(12);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap_or(()); // ignore if already initialized

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

    // Knockout analysis: cargo run -- --knockout
    if args.iter().any(|a| a == "--knockout") {
        run_knockout_analysis();
        return;
    }

    // Optimized GATE history: cargo run -- --opt-gate
    if args.iter().any(|a| a == "--opt-gate") {
        run_opt_gate();
        return;
    }

    // Value gradient analysis: cargo run -- --gradient
    if args.iter().any(|a| a == "--gradient") {
        run_gradient();
        return;
    }

    // GATE x Parameter cross experiment: cargo run -- --exp-cross
    if args.iter().any(|a| a == "--exp-cross") {
        run_exp_cross();
        return;
    }

    // EXP-015a feast/famine cycling: cargo run -- --exp015a
    if args.iter().any(|a| a == "--exp015a") {
        run_exp015a();
        return;
    }

    // Lineage tracking demo: cargo run --release -- --lineage
    if args.iter().any(|a| a == "--lineage") {
        run_lineage_demo();
        return;
    }

    // EXP-014 lineage analysis: cargo run --release -- --lineage014
    if args.iter().any(|a| a == "--lineage014") {
        lineage014::run_exp014_lineage();
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

/// Run one EXP-009 round: CPU-modulated food vs constant food.
/// Uses hash-based pseudo-CPU for reproducibility in parallel.
fn run_cpu_round(seed: u64) -> (cell_vm::CellSteadyState, cell_vm::CellSteadyState) {
    use cell_vm::{CellConfig, CellWorld, cell_seed_a, cell_seed_b};

    let total_ticks: u64 = 500_000;
    let base_food: i32 = 500;
    let cpu_interval: u64 = 100;

    // CPU-modulated group
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.food_per_tick = 50; // baseline
    config.total_ticks = total_ticks;
    config.max_organisms = 200;

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..20 { world.add_organism(cell_seed_b(&config)); }

    // Constant food control (matching avg effective food)
    let mut ctrl_config = config.clone();
    ctrl_config.food_per_tick = 55; // 50 baseline + ~5 avg injection
    let mut ctrl_world = CellWorld::new(ctrl_config.clone(), seed);
    for _ in 0..20 { ctrl_world.add_organism(cell_seed_a(&ctrl_config)); }
    for _ in 0..20 { ctrl_world.add_organism(cell_seed_b(&ctrl_config)); }

    for t in 0..total_ticks {
        // Hash-based pseudo-CPU signal (deterministic, varies with tick and seed)
        if t % cpu_interval == 0 {
            let block = t / cpu_interval;
            let hash = block.wrapping_mul(6364136223846793005).wrapping_add(seed) >> 33;
            let cpu_usage = (hash % 100) as f32 / 100.0;
            let available = 0.3 + 0.7 * (1.0 - cpu_usage).max(0.0);
            let food = (base_food as f32 * available) as i32;
            world.food_pool += food;
        }
        world.tick();
        ctrl_world.tick();
    }

    world.take_snapshot();
    ctrl_world.take_snapshot();
    (cell_compute_steady_state(&world.snapshots), cell_compute_steady_state(&ctrl_world.snapshots))
}

fn run_realcpu_experiment() {
    use rayon::prelude::*;

    eprintln!("EXP-009: CPU-Modulated Food — 100 Independent Rounds");
    eprintln!("====================================================");
    eprintln!("  FROZEN PARAMS: food=50 base + 500 injection/100tick, CEM=50");
    eprintln!("  100 rounds, 500k ticks, --threads limited\n");

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-009-realcpu/data");

    let rounds: Vec<u64> = (6000..6100).collect(); // 100 seeds

    let results: Vec<(u64, cell_vm::CellSteadyState, cell_vm::CellSteadyState)> = rounds.par_iter()
        .map(|&seed| {
            if seed % 10 == 0 { eprintln!("  Round seed {}...", seed); }
            let (cpu, ctrl) = run_cpu_round(seed);
            (seed, cpu, ctrl)
        })
        .collect();

    // Extract metrics
    let cpu_eat: Vec<f64> = results.iter().map(|(_, c, _)| c.eat_ratio).collect();
    let ctrl_eat: Vec<f64> = results.iter().map(|(_, _, c)| c.eat_ratio).collect();
    let cpu_ref: Vec<f64> = results.iter().map(|(_, c, _)| c.refresh_ratio).collect();
    let ctrl_ref: Vec<f64> = results.iter().map(|(_, _, c)| c.refresh_ratio).collect();
    let cpu_pop: Vec<f64> = results.iter().map(|(_, c, _)| c.avg_population).collect();
    let ctrl_pop: Vec<f64> = results.iter().map(|(_, _, c)| c.avg_population).collect();

    let comp_eat = stats::compare_groups("EAT", &cpu_eat, &ctrl_eat);
    let comp_ref = stats::compare_groups("REFRESH", &cpu_ref, &ctrl_ref);
    let comp_pop = stats::compare_groups("Population", &cpu_pop, &ctrl_pop);

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let survived_cpu = results.iter().filter(|(_, c, _)| c.survived).count();
    let survived_ctrl = results.iter().filter(|(_, _, c)| c.survived).count();

    // Direction wins
    let eat_positive = cpu_eat.iter().zip(ctrl_eat.iter()).filter(|(a, b)| a > b).count();
    let ref_diff: Vec<f64> = cpu_ref.iter().zip(ctrl_ref.iter()).map(|(a, b)| a - b).collect();

    let mut report = String::new();
    report.push_str("# EXP-009: CPU-Modulated Food — 100 Independent Rounds\n\n");
    report.push_str("## Frozen Parameters\n\n");
    report.push_str("- food_per_tick=50 (baseline) + 500 injection per 100 ticks\n");
    report.push_str("- CPU floor: 30%, formula: food += 500 * (0.3 + 0.7*(1-cpu))\n");
    report.push_str("- Control: constant 55/tick\n");
    report.push_str("- CEM=50, max=200, 20A+20B, 500k ticks\n");
    report.push_str("- 100 rounds (seeds 6000-6099), hash-based pseudo-CPU\n\n");

    report.push_str("## Results\n\n");
    report.push_str(&format!("Survived: CPU {}/100, Ctrl {}/100\n\n", survived_cpu, survived_ctrl));
    report.push_str("| Metric | CPU-modulated | Constant | Diff | p | d |\n");
    report.push_str("|--------|-------------|----------|------|---|---|\n");
    report.push_str(&format!("| EAT% | {:.1} | {:.1} | {:.4} | {:.4} | {:.3} |\n",
        avg(&cpu_eat)*100.0, avg(&ctrl_eat)*100.0, comp_eat.mean_diff, comp_eat.p_value, comp_eat.cohens_d));
    report.push_str(&format!("| REFRESH% | {:.1} | {:.1} | {:.4} | {:.4} | {:.3} |\n",
        avg(&cpu_ref)*100.0, avg(&ctrl_ref)*100.0, comp_ref.mean_diff, comp_ref.p_value, comp_ref.cohens_d));
    report.push_str(&format!("| Population | {:.1} | {:.1} | {:.1} | {:.4} | {:.3} |\n",
        avg(&cpu_pop), avg(&ctrl_pop), comp_pop.mean_diff, comp_pop.p_value, comp_pop.cohens_d));

    report.push_str(&format!("\n## Direction Win Rate\n\n"));
    report.push_str(&format!("- EAT (cpu > ctrl): {}/100 ({:.0}%)\n", eat_positive, eat_positive as f64));

    report.push_str("\n---\n*EXP-009: 100 independent rounds with frozen parameters*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-009-realcpu";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/per_round.csv", dir)).expect("csv");
    writeln!(csv, "seed,cpu_eat,ctrl_eat,cpu_refresh,ctrl_refresh,cpu_pop,ctrl_pop").unwrap();
    for (i, &seed) in rounds.iter().enumerate() {
        writeln!(csv, "{},{:.6},{:.6},{:.6},{:.6},{:.2},{:.2}",
            seed, cpu_eat[i], ctrl_eat[i], cpu_ref[i], ctrl_ref[i], cpu_pop[i], ctrl_pop[i]).unwrap();
    }

    eprintln!("\nResults: docs/experiments/EXP-009-realcpu/experiment.md");
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

        let base_food_per_tick: i32 = 50; // baseline food per tick (same as normal runs)
        let total_ticks: u64 = 500_000;
        let cpu_sample_interval: u64 = 100;

        // CPU-modulated world: base food + CPU-variable bonus
        // Every cpu_sample_interval ticks, inject a bonus proportional to CPU availability
        // cpu_bonus = base_food * (cpu_available - 0.5) scaled around baseline
        let bonus_base: i32 = 150; // bonus food per 100-tick sample (1.5 food/tick on top of 50/tick)

        let mut cpu_config = CellConfig::experimental();
        cpu_config.cell_energy_max = 50;
        cpu_config.food_per_tick = base_food_per_tick; // steady baseline
        cpu_config.total_ticks = total_ticks;
        cpu_config.snapshot_interval = 5_000;
        cpu_config.genome_dump_interval = 0;

        let mut cpu_world = CellWorld::new(cpu_config.clone(), seed);
        for _ in 0..10 { cpu_world.add_organism(cell_seed_a(&cpu_config)); }
        for _ in 0..10 { cpu_world.add_organism(cell_seed_b(&cpu_config)); }

        // Constant-food world (same average food as CPU world = base_food_per_tick + bonus_base/2 per tick)
        let const_food_per_tick = base_food_per_tick + bonus_base / cpu_sample_interval as i32;
        let mut const_config = CellConfig::experimental();
        const_config.cell_energy_max = 50;
        const_config.food_per_tick = const_food_per_tick;
        const_config.total_ticks = total_ticks;
        const_config.snapshot_interval = 5_000;
        const_config.genome_dump_interval = 0;

        let mut const_world = CellWorld::new(const_config.clone(), seed);
        for _ in 0..10 { const_world.add_organism(cell_seed_a(&const_config)); }
        for _ in 0..10 { const_world.add_organism(cell_seed_b(&const_config)); }

        // Run tick-by-tick: cpu world gets variable bonus injection every 100 ticks
        for t in 0..total_ticks {
            if t % cpu_sample_interval == 0 {
                // Hash-based pseudo-CPU usage (deterministic, seed-dependent)
                let block = t / cpu_sample_interval;
                let hash = block.wrapping_mul(6364136223846793005u64)
                    .wrapping_add(seed.wrapping_mul(1442695040888963407u64))
                    >> 33;
                let cpu_usage = (hash % 100) as f32 / 100.0;
                // CPU available: high cpu_usage → less food (organism at mercy of environment)
                let available = 0.3 + 0.7 * (1.0 - cpu_usage).max(0.0);
                let bonus = (bonus_base as f32 * available) as i32;
                cpu_world.food_pool += bonus;
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

// ============================================================================
// EXP-015a: Feast/Famine Cycling + GATE
// ============================================================================

fn run_cycle_trial(seed: u64, cycling: bool) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.total_ticks = 1_000_000;
    config.max_organisms = 200;
    config.data_cell_gating = true;
    config.snapshot_interval = 1000;
    config.genome_dump_interval = 0;

    // Cycling group: start at feast level; constant group: fixed average
    if cycling {
        config.food_per_tick = 500; // will be modulated in loop
    } else {
        config.food_per_tick = 275; // (500+50)/2
    }

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_g(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    let cycle_period: u64 = 20_000; // 10k feast + 10k famine

    for t in 0..config.total_ticks {
        if cycling {
            let phase = t % cycle_period;
            world.config.food_per_tick = if phase < cycle_period / 2 { 500 } else { 50 };
        }
        world.tick();
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_exp015a() {
    use rayon::prelude::*;

    let num_rounds = 100;
    let seeds_per_round = 10;

    eprintln!("EXP-015a: Feast/Famine Cycling + GATE");
    eprintln!("=====================================");
    eprintln!("  {} rounds x {} seeds, 1M ticks, cycle=20k (10k feast/10k famine)\n", num_rounds, seeds_per_round);

    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-015a-feast-famine-cycle/data");

    struct RoundResult015 {
        round: usize,
        refresh_diff: f64,
        eat_diff: f64,
        pop_diff: f64,
        refresh_p: f64,
        refresh_d: f64,
    }

    let rounds: Vec<usize> = (0..num_rounds).collect();
    let results: Vec<RoundResult015> = rounds.par_iter().map(|&round| {
        let base_seed = (round as u64) * 1000 + 8000;
        let seeds: Vec<u64> = (base_seed..base_seed + seeds_per_round as u64).collect();

        let cycling: Vec<cell_vm::CellSteadyState> = seeds.iter()
            .map(|&s| run_cycle_trial(s, true)).collect();
        let constant: Vec<cell_vm::CellSteadyState> = seeds.iter()
            .map(|&s| run_cycle_trial(s, false)).collect();

        let c_ref: Vec<f64> = cycling.iter().map(|r| r.refresh_ratio).collect();
        let k_ref: Vec<f64> = constant.iter().map(|r| r.refresh_ratio).collect();
        let c_eat: Vec<f64> = cycling.iter().map(|r| r.eat_ratio).collect();
        let k_eat: Vec<f64> = constant.iter().map(|r| r.eat_ratio).collect();
        let c_pop: Vec<f64> = cycling.iter().map(|r| r.avg_population).collect();
        let k_pop: Vec<f64> = constant.iter().map(|r| r.avg_population).collect();

        let comp = stats::compare_groups("REF", &c_ref, &k_ref);
        let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;

        if round % 10 == 0 { eprintln!("  Round {}/{}", round + 1, num_rounds); }

        RoundResult015 {
            round,
            refresh_diff: avg(&c_ref) - avg(&k_ref),
            eat_diff: avg(&c_eat) - avg(&k_eat),
            pop_diff: avg(&c_pop) - avg(&k_pop),
            refresh_p: comp.p_value,
            refresh_d: comp.cohens_d,
        }
    }).collect();

    // Meta-analysis
    let ref_positive = results.iter().filter(|r| r.refresh_diff > 0.0).count();
    let ref_sig = results.iter().filter(|r| r.refresh_p < 0.05 && r.refresh_diff > 0.0).count();
    let pop_diff_neg = results.iter().filter(|r| r.pop_diff < 0.0).count();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = avg(v);
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };

    let all_ref_d: Vec<f64> = results.iter().map(|r| r.refresh_diff).collect();
    let all_d: Vec<f64> = results.iter().map(|r| r.refresh_d).collect();

    let mut report = String::new();
    report.push_str("# EXP-015a: Feast/Famine Cycling + GATE — 100 Independent Rounds\n\n");
    report.push_str("## Frozen Parameters\n\n");
    report.push_str("- GATE=true, CEM=50, max=200, 20G+10A+10B\n");
    report.push_str("- Cycling: 500 food (feast, 10k ticks) → 50 food (famine, 10k ticks), repeat\n");
    report.push_str("- Constant: 275 food/tick (average of feast+famine)\n");
    report.push_str("- 1M ticks, 100 rounds x 10 seeds/round\n\n");

    report.push_str("## Meta-Analysis\n\n");
    report.push_str("| Metric | Direction win | p<0.05 win | Mean diff | SD | Win rate |\n");
    report.push_str("|--------|-------------|-----------|-----------|-----|----------|\n");
    report.push_str(&format!(
        "| REFRESH (cycle > const) | {}/{} | {}/{} | {:.4} | {:.4} | {:.0}% |\n",
        ref_positive, num_rounds, ref_sig, num_rounds,
        avg(&all_ref_d), sd(&all_ref_d), ref_positive as f64 / num_rounds as f64 * 100.0,
    ));
    report.push_str(&format!(
        "| Population (cycle < const) | {}/{} | — | — | — | {:.0}% |\n",
        pop_diff_neg, num_rounds, pop_diff_neg as f64 / num_rounds as f64 * 100.0,
    ));

    report.push_str(&format!("\nMean Cohen's d: {:.3}, SD: {:.3}\n", avg(&all_d), sd(&all_d)));
    report.push_str(&format!("Positive d: {}/{} ({:.0}%)\n", results.iter().filter(|r| r.refresh_d > 0.0).count(), num_rounds,
        results.iter().filter(|r| r.refresh_d > 0.0).count() as f64 / num_rounds as f64 * 100.0));

    report.push_str("\n## Conclusion\n\n");
    if ref_positive > 60 {
        report.push_str(&format!(
            "Feast/famine cycling produces measurable behavioral differentiation:\n\
            REFRESH in predicted direction in {}% of rounds, p<0.05 in {}%.\n",
            ref_positive, ref_sig));
    } else {
        report.push_str("Feast/famine cycling does not consistently produce REFRESH differentiation.\n");
    }

    report.push_str("\n---\n*EXP-015a: Feast/famine cycling with GATE*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-015a-feast-famine-cycle";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/per_round.csv", dir)).expect("csv");
    writeln!(csv, "round,refresh_diff,eat_diff,pop_diff,refresh_p,refresh_d").unwrap();
    for r in &results {
        writeln!(csv, "{},{:.6},{:.6},{:.2},{:.6},{:.4}",
            r.round, r.refresh_diff, r.eat_diff, r.pop_diff, r.refresh_p, r.refresh_d).unwrap();
    }

    eprintln!("\nResults: {}/experiment.md", dir);
    println!("{}", report);
}

// ============================================================================
// Lineage Tracking Demo
// ============================================================================

fn run_lineage_demo() {
    eprintln!("Lineage Tracking Demo");
    eprintln!("======================");
    eprintln!("  50k ticks, seed G initial pop, lineage_tracking=true");

    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.total_ticks = 50_000;
    config.max_organisms = 200;
    config.data_cell_gating = true;
    config.lineage_tracking = true;
    config.snapshot_interval = 5000;
    config.genome_dump_interval = 0;

    let mut world = CellWorld::new(config.clone(), 42);
    for _ in 0..20 { world.add_organism(cell_seed_g(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    for t in 0..config.total_ticks {
        world.tick();
        if t % 10_000 == 0 {
            let alive = world.organisms.iter().filter(|o| o.alive).count();
            eprintln!("  tick {} — {} alive, {} birth records", t, alive, world.lineage.records.len());
        }
    }

    let out_dir = "D:/project/d0-vm/docs/experiments/lineage-demo";
    let csv_path = format!("{}/lineage.csv", out_dir);
    std::fs::create_dir_all(out_dir).ok();
    world.write_lineage_csv(&csv_path).expect("failed to write lineage CSV");

    let total_births = world.lineage.records.len();
    let mutations: usize = world.lineage.records.iter().map(|r| r.mutation_sites.len()).sum();
    let mutated = world.lineage.records.iter().filter(|r| !r.mutation_sites.is_empty()).count();

    // Trace ancestry of last organism
    let last_alive = world.organisms.iter().filter(|o| o.alive).last().map(|o| o.id);
    let ancestry_depth = if let Some(id) = last_alive {
        world.lineage.ancestors(id).len()
    } else { 0 };

    eprintln!("\n=== Results ===");
    eprintln!("  Total birth records: {}", total_births);
    eprintln!("  Births with mutation: {} ({:.1}%)", mutated, 100.0 * mutated as f64 / total_births.max(1) as f64);
    eprintln!("  Total mutation events: {}", mutations);
    eprintln!("  Ancestry depth (last organism): {}", ancestry_depth);
    eprintln!("  CSV: {}", csv_path);
}

// ============================================================================
// EXP-CROSS: GATE × Parameter 2×2 Matrix
// ============================================================================

fn run_cross_trial(seed: u64, optimized: bool, gate: bool, abundant_first: bool) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.data_cell_gating = gate;
    config.genome_dump_interval = 0;

    if optimized {
        config.max_organisms = 1000;
        config.food_per_tick = if abundant_first { 500 } else { 500 }; // optimized always has good food
        config.total_ticks = 2_000_000;
        config.snapshot_interval = 10_000;
    } else {
        config.max_organisms = 200;
        config.food_per_tick = if abundant_first { 500 } else { 50 };
        config.total_ticks = 500_000;
        config.snapshot_interval = 1000;
    }

    let mut world = CellWorld::new(config.clone(), seed);

    if gate {
        if optimized {
            for _ in 0..30 { world.add_organism(cell_seed_a(&config)); }
            for _ in 0..30 { world.add_organism(cell_seed_b(&config)); }
            for _ in 0..40 { world.add_organism(cell_seed_g(&config)); }
        } else {
            for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
            for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }
            for _ in 0..20 { world.add_organism(cell_seed_g(&config)); }
        }
    } else {
        if optimized {
            for _ in 0..50 { world.add_organism(cell_seed_a(&config)); }
            for _ in 0..50 { world.add_organism(cell_seed_b(&config)); }
        } else {
            for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
            for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }
        }
    }

    // For history experiments: switch food at tick 10k
    if abundant_first && !optimized {
        let switch_tick: u64 = 10_000;
        for t in 0..config.total_ticks {
            if t == switch_tick { world.config.food_per_tick = 50; }
            world.tick();
        }
    } else {
        world.run();
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_exp_cross() {
    use rayon::prelude::*;

    eprintln!("EXP-CROSS: GATE x Parameter 2x2 Matrix");
    eprintln!("========================================\n");

    let seeds: Vec<u64> = (7000..7100).collect(); // 100 seeds
    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-CROSS-gate-params/data");

    // Group 2: Optimized + GATE (history: abundant→abundant, testing GATE in good env)
    eprintln!("Group 2: Optimized + GATE (100 seeds, 2M ticks)...");
    let g2: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_cross_trial(s, true, true, false)).collect();

    // Group 4: Non-optimized + GATE (history: abundant→scarce at 10k)
    eprintln!("Group 4: Non-optimized + GATE, abundant→scarce (100 seeds, 500k ticks)...");
    let g4_exp: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_cross_trial(s, false, true, true)).collect();

    // Group 4 control: Non-optimized + GATE, always scarce
    eprintln!("Group 4 ctrl: Non-optimized + GATE, always scarce...");
    let g4_ctrl: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_cross_trial(s, false, true, false)).collect();

    // Group 3 equivalent: Non-optimized + No GATE, for comparison
    eprintln!("Group 3: Non-optimized + No GATE...");
    let g3: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| run_cross_trial(s, false, false, false)).collect();

    // Compute history replication rate for Group 4 (GATE)
    let g4_ref_exp: Vec<f64> = g4_exp.iter().map(|r| r.refresh_ratio).collect();
    let g4_ref_ctrl: Vec<f64> = g4_ctrl.iter().map(|r| r.refresh_ratio).collect();
    let g4_ref_positive = g4_ref_exp.iter().zip(g4_ref_ctrl.iter())
        .filter(|(e, c)| e > c).count();

    let comp_g4 = stats::compare_groups("REFRESH", &g4_ref_exp, &g4_ref_ctrl);

    // Group 2 stats
    let g2_survived = g2.iter().filter(|r| r.survived).count();
    let g2_ref: Vec<f64> = g2.iter().map(|r| r.refresh_ratio).collect();
    let g2_pop: Vec<f64> = g2.iter().map(|r| r.avg_population).collect();

    // Group 3 stats
    let g3_survived = g3.iter().filter(|r| r.survived).count();
    let g3_ref: Vec<f64> = g3.iter().map(|r| r.refresh_ratio).collect();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;

    let mut report = String::new();
    report.push_str("# EXP-CROSS: GATE x Parameter 2x2 Matrix\n\n");
    report.push_str("## 2x2 Matrix\n\n");
    report.push_str("| | No GATE | GATE |\n");
    report.push_str("|---|---------|------|\n");
    report.push_str(&format!(
        "| Optimized (food=500, max=1000, 2M) | G1: existing data (p<0.001) | G2: {}/100 survived, REFRESH {:.1}%, Pop {:.0} |\n",
        g2_survived, avg(&g2_ref) * 100.0, avg(&g2_pop)));
    report.push_str(&format!(
        "| Non-optimized (food=50, max=200, 500k) | G3: {}/100 survived, REFRESH {:.1}% | G4: history repl rate **{:.0}%** (p={:.4}) |\n",
        g3_survived, avg(&g3_ref) * 100.0,
        g4_ref_positive as f64, comp_g4.p_value));

    report.push_str("\n## Group 4 Detail (Non-optimized + GATE, history experiment)\n\n");
    report.push_str(&format!("- Abundant→Scarce REFRESH: {:.1}%\n", avg(&g4_ref_exp) * 100.0));
    report.push_str(&format!("- Always Scarce REFRESH: {:.1}%\n", avg(&g4_ref_ctrl) * 100.0));
    report.push_str(&format!("- Direction win: {}/100 ({:.0}%)\n", g4_ref_positive, g4_ref_positive as f64));
    report.push_str(&format!("- MW p={:.4}, d={:.3}\n", comp_g4.p_value, comp_g4.cohens_d));

    report.push_str("\n---\n*EXP-CROSS: GATE x Parameter interaction*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-CROSS-gate-params";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/per_seed.csv", dir)).expect("csv");
    writeln!(csv, "group,seed,survived,refresh,eat,divide,pop,energy").unwrap();
    for (i, &seed) in seeds.iter().enumerate() {
        let r = &g2[i]; writeln!(csv, "g2_opt_gate,{},{},{:.6},{:.6},{:.6},{:.2},{:.2}", seed, r.survived, r.refresh_ratio, r.eat_ratio, r.divide_ratio, r.avg_population, r.avg_energy).unwrap();
        let r = &g3[i]; writeln!(csv, "g3_nonopt_nogate,{},{},{:.6},{:.6},{:.6},{:.2},{:.2}", seed, r.survived, r.refresh_ratio, r.eat_ratio, r.divide_ratio, r.avg_population, r.avg_energy).unwrap();
        let r = &g4_exp[i]; writeln!(csv, "g4_nonopt_gate_exp,{},{},{:.6},{:.6},{:.6},{:.2},{:.2}", seed, r.survived, r.refresh_ratio, r.eat_ratio, r.divide_ratio, r.avg_population, r.avg_energy).unwrap();
        let r = &g4_ctrl[i]; writeln!(csv, "g4_nonopt_gate_ctrl,{},{},{:.6},{:.6},{:.6},{:.2},{:.2}", seed, r.survived, r.refresh_ratio, r.eat_ratio, r.divide_ratio, r.avg_population, r.avg_energy).unwrap();
    }

    eprintln!("\nResults: {}/experiment.md", dir);
    println!("{}", report);
}

// ============================================================================
// Knockout analysis on seed organisms
// ============================================================================

fn run_knockout_analysis() {
    use cell_vm::{cell_seed_a, cell_seed_b, cell_seed_g, CellConfig};

    eprintln!("Knockout Analysis: Seed A, B, G");
    eprintln!("================================\n");

    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.data_cell_gating = true;

    let seeds = [
        ("Seed_A (minimal survival)", cell_seed_a(&config)),
        ("Seed_B (conditional divide)", cell_seed_b(&config)),
        ("Seed_G (GATE evaluation)", cell_seed_g(&config)),
    ];

    let mut full_report = String::new();
    full_report.push_str("# Knockout Analysis: Minimum Essential Instruction Set\n\n");
    full_report.push_str("Each Code cell replaced with NOP one at a time. Sandbox: 10k ticks, abundant food.\n\n");

    let mut all_csv = String::from("organism,position,cell_index,instruction,result,survival,energy,pop\n");

    for (name, org) in &seeds {
        eprintln!("Analyzing {}...", name);
        let results = knockout::analyze_knockout(org, &config, 42, 10_000);

        full_report.push_str(&format!("## {}\n\n", name));
        full_report.push_str(&format!("Code cells: {}, Total cells: {}\n\n", org.code_count(), org.cells.len()));
        full_report.push_str("| Pos | Instruction | Result | Survival | Energy | Pop |\n");
        full_report.push_str("|-----|------------|--------|----------|--------|-----|\n");
        for r in &results {
            full_report.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n",
                r.position, r.original, r.category.label(), r.survival_ticks, r.final_energy, r.final_pop));
        }
        full_report.push_str(&format!("\n{}\n\n", knockout::summarize(&results)));

        for r in &results {
            all_csv.push_str(&format!("{},{},{},{},{},{},{},{}\n",
                name, r.position, r.cell_index, r.original, r.category.label(), r.survival_ticks, r.final_energy, r.final_pop));
        }
    }

    let dir = "D:/project/d0-vm/docs/experiments/EXP-014-gate-learning";
    fs::write(format!("{}/knockout_analysis.md", dir), &full_report).expect("write");
    fs::write(format!("{}/data/knockout.csv", dir), &all_csv).expect("write");
    eprintln!("\nResults: {}/knockout_analysis.md", dir);
    println!("{}", full_report);
}

// ============================================================================
// Optimized params + GATE history experiment (Paper I final blocker)
// ============================================================================

fn run_opt_gate_trial(seed: u64, abundant_first: bool) -> cell_vm::CellSteadyState {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.max_organisms = 1000;
    config.total_ticks = 2_000_000;
    config.data_cell_gating = true;
    config.snapshot_interval = 10_000;
    config.genome_dump_interval = 0;

    if abundant_first {
        config.food_per_tick = 500; // abundant first 500k, then scarce
    } else {
        config.food_per_tick = 50; // always scarce
    }

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..20 { world.add_organism(cell_seed_g(&config)); }
    for _ in 0..30 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..30 { world.add_organism(cell_seed_b(&config)); }

    let switch_tick: u64 = 500_000; // switch at 500k (25% of 2M)

    for t in 0..config.total_ticks {
        if abundant_first && t == switch_tick {
            world.config.food_per_tick = 50;
        }
        world.tick();
    }

    world.take_snapshot();
    cell_compute_steady_state(&world.snapshots)
}

fn run_opt_gate() {
    use rayon::prelude::*;

    eprintln!("Optimized GATE History Experiment (Paper I final)");
    eprintln!("=================================================");
    eprintln!("  CEM=50, food=500→50 at 500k, max=1000, 20G+30A+30B");
    eprintln!("  100 seeds, 2M ticks, --threads limited\n");

    let seeds: Vec<u64> = (9000..9100).collect();
    let _ = fs::create_dir_all("D:/project/d0-vm/docs/experiments/EXP-OPT-GATE/data");

    eprintln!("Running abundant→scarce...");
    let exp: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| { if s % 20 == 0 { eprintln!("  seed {}...", s); } run_opt_gate_trial(s, true) }).collect();

    eprintln!("Running always scarce...");
    let ctrl: Vec<cell_vm::CellSteadyState> = seeds.par_iter()
        .map(|&s| { if s % 20 == 0 { eprintln!("  seed {}...", s); } run_opt_gate_trial(s, false) }).collect();

    let exp_ref: Vec<f64> = exp.iter().map(|r| r.refresh_ratio).collect();
    let ctrl_ref: Vec<f64> = ctrl.iter().map(|r| r.refresh_ratio).collect();
    let exp_pop: Vec<f64> = exp.iter().map(|r| r.avg_population).collect();
    let ctrl_pop: Vec<f64> = ctrl.iter().map(|r| r.avg_population).collect();

    let comp_ref = stats::compare_groups("REFRESH", &exp_ref, &ctrl_ref);
    let comp_pop = stats::compare_groups("Population", &exp_pop, &ctrl_pop);

    let ref_positive = exp_ref.iter().zip(ctrl_ref.iter()).filter(|(e, c)| e > c).count();
    let pop_neg = exp_pop.iter().zip(ctrl_pop.iter()).filter(|(e, c)| e < c).count();

    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let surv_exp = exp.iter().filter(|r| r.survived).count();
    let surv_ctrl = ctrl.iter().filter(|r| r.survived).count();

    let mut report = String::new();
    report.push_str("# Optimized GATE History Experiment\n\n");
    report.push_str("## Parameters (frozen)\n\n");
    report.push_str("- CEM=50, max=1000, 20G+30A+30B, 2M ticks\n");
    report.push_str("- Exp: food=500 first 500k ticks → food=50\n");
    report.push_str("- Ctrl: food=50 throughout\n");
    report.push_str("- GATE=true, 100 seeds\n\n");

    report.push_str("## Results\n\n");
    report.push_str(&format!("Survived: Exp {}/100, Ctrl {}/100\n\n", surv_exp, surv_ctrl));
    report.push_str("| Group | Avg Pop | Avg Energy | REFRESH% |\n");
    report.push_str("|-------|---------|-----------|----------|\n");
    report.push_str(&format!("| Abundant→Scarce | {:.1} | {:.1} | {:.1} |\n",
        avg(&exp_pop), avg(&exp.iter().map(|r| r.avg_energy).collect::<Vec<_>>()), avg(&exp_ref)*100.0));
    report.push_str(&format!("| Always Scarce | {:.1} | {:.1} | {:.1} |\n",
        avg(&ctrl_pop), avg(&ctrl.iter().map(|r| r.avg_energy).collect::<Vec<_>>()), avg(&ctrl_ref)*100.0));

    report.push_str("\n## Statistical Tests\n\n");
    report.push_str(&format!("- REFRESH: p={:.4}, d={:.3}, direction win {}/100 ({:.0}%)\n",
        comp_ref.p_value, comp_ref.cohens_d, ref_positive, ref_positive as f64));
    report.push_str(&format!("- Population: p={:.4}, d={:.3}, direction win {}/100\n",
        comp_pop.p_value, comp_pop.cohens_d, pop_neg));

    report.push_str("\n---\n*Optimized GATE history: Paper I final experiment*\n");

    let dir = "D:/project/d0-vm/docs/experiments/EXP-OPT-GATE";
    fs::write(format!("{}/experiment.md", dir), &report).expect("write");

    let mut csv = fs::File::create(format!("{}/data/per_seed.csv", dir)).expect("csv");
    writeln!(csv, "seed,group,survived,pop,energy,refresh,eat,divide").unwrap();
    for (i, &seed) in seeds.iter().enumerate() {
        let r = &exp[i]; writeln!(csv, "{},exp,{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.refresh_ratio, r.eat_ratio, r.divide_ratio).unwrap();
        let r = &ctrl[i]; writeln!(csv, "{},ctrl,{},{:.2},{:.2},{:.6},{:.6},{:.6}",
            seed, r.survived, r.avg_population, r.avg_energy, r.refresh_ratio, r.eat_ratio, r.divide_ratio).unwrap();
    }

    eprintln!("\nResults: {}/experiment.md", dir);
    println!("{}", report);
}

// ============================================================================
// Value Gradient: energy-bucketed instruction analysis
// ============================================================================

fn run_gradient_trial(seed: u64, freshness_decay: bool) -> [[u64; 4]; 5] {
    let mut config = CellConfig::experimental();
    config.cell_energy_max = 50;
    config.max_organisms = 1000;
    config.food_per_tick = 500;
    config.total_ticks = 2_000_000;
    config.snapshot_interval = 100_000;
    config.genome_dump_interval = 0;
    config.freshness_decay = freshness_decay;

    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..50 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..50 { world.add_organism(cell_seed_b(&config)); }
    world.run();
    world.energy_buckets
}

fn run_gradient() {
    use rayon::prelude::*;
    eprintln!("Value Gradient Analysis (100 seeds, 2M ticks)\n");

    let seeds: Vec<u64> = (3000..3100).collect();

    eprintln!("Running exp (decay=true)...");
    let exp_b: Vec<[[u64; 4]; 5]> = seeds.par_iter().map(|&s| run_gradient_trial(s, true)).collect();
    eprintln!("Running ctrl (decay=false)...");
    let ctrl_b: Vec<[[u64; 4]; 5]> = seeds.par_iter().map(|&s| run_gradient_trial(s, false)).collect();

    let mut ea = [[0u64; 4]; 5];
    let mut ca = [[0u64; 4]; 5];
    for b in &exp_b { for i in 0..5 { for j in 0..4 { ea[i][j] += b[i][j]; } } }
    for b in &ctrl_b { for i in 0..5 { for j in 0..4 { ca[i][j] += b[i][j]; } } }

    let dir = "D:/project/d0-vm/docs/experiments/value-gradient";
    let _ = std::fs::create_dir_all(dir);
    let labels = ["0-20%", "20-40%", "40-60%", "60-80%", "80-100%"];

    let mut csv = std::fs::File::create(format!("{}/gradient_data.csv", dir)).expect("csv");
    use std::io::Write;
    writeln!(csv, "group,bucket,eat,refresh,divide,total,eat_pct,refresh_pct,divide_pct").unwrap();
    for (name, agg) in [("exp", &ea), ("ctrl", &ca)] {
        for i in 0..5 {
            let t = agg[i][3].max(1);
            writeln!(csv, "{},{},{},{},{},{},{:.4},{:.4},{:.4}", name, labels[i],
                agg[i][0], agg[i][1], agg[i][2], t,
                agg[i][0] as f64/t as f64, agg[i][1] as f64/t as f64, agg[i][2] as f64/t as f64).unwrap();
        }
    }

    let mut rpt = String::new();
    rpt.push_str("# Value Gradient Analysis\n\n");
    for (name, agg) in [("Experimental", &ea), ("Control", &ca)] {
        rpt.push_str(&format!("## {}\n\n| Bucket | EAT% | REFRESH% | DIVIDE% |\n|--------|------|----------|--------|\n", name));
        for i in 0..5 {
            let t = agg[i][3].max(1) as f64;
            rpt.push_str(&format!("| {} | {:.1} | {:.1} | {:.1} |\n", labels[i],
                agg[i][0] as f64/t*100.0, agg[i][1] as f64/t*100.0, agg[i][2] as f64/t*100.0));
        }
        rpt.push('\n');
    }
    std::fs::write(format!("{}/analysis.md", dir), &rpt).expect("write");
    eprintln!("\nResults: {}/analysis.md", dir);
    println!("{}", rpt);
}
