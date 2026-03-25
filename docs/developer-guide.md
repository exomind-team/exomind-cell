# ExoMind Cell — Developer Guide

## Experiment Workflow

### Parameter Freeze Protocol

**Lesson learned**: Early experiments had parameter mismatches (CEM=20 vs 50, food=3 vs 500) causing mass extinction. Now all experiments must follow:

1. Write `params_frozen.sh` with ALL parameters before running
2. Send to Leader for review
3. Only run after approval
4. Include `export RAYON_NUM_THREADS=12` (leave CPU headroom)

### Baseline Parameters (Cell v3)

These produced all p<0.001 in 100-seed validation:
```
CEM=50, food_per_tick=500, max_organisms=1000
50 Seed A + 50 Seed B, 2_000_000 ticks
freshness_max=255, refresh_radius=5, mutation_rate=0.001
```

### Running Experiments

```bash
cargo run --release -- --stats --threads 12     # 100-seed operational closure
cargo run --release -- --exp014 --threads 12    # GATE history (100 rounds)
cargo run --release -- --exp-cross --threads 12 # 2x2 GATE x parameter matrix
cargo run --release -- --gradient --threads 12  # Value gradient analysis
cargo run --release -- --knockout               # Knockout analysis
cargo run --release -- --real-cpu --threads 12  # CPU-modulated food
```

## Bilingual Figure Generation

```bash
# Chinese (default, saves to paper figures directory)
python scripts/gen_paper_figures.py --lang cn

# English (saves to specified directory)
python scripts/gen_paper_figures.py --lang en --output figures_en/

# All 7 figures:
# fig_refresh_distribution  — REFRESH distribution histogram + boxplot
# fig_gate_history          — GATE history effect bars + distribution
# fig_2x2_matrix            — GATE x parameter interaction heatmap
# fig_instruction_ratios    — EAT/REFRESH/DIVIDE grouped bars
# fig_knockout              — Essential instruction stacked bars
# fig_value_gradient        — Energy-bucketed instruction ratios
# fig_r_gradient            — REFRESH radius effect
```

Label dictionary in script: 42 switchable labels via `L['key']`.

## Analysis Tools

### Knockout Analysis (`src/knockout.rs`)

Single-site NOP replacement to find essential instructions.

```bash
cargo run --release -- --knockout
```

Output: `docs/experiments/EXP-014-gate-learning/knockout_analysis.md`

Categories: lethal / severe_defect / mild_defect / neutral / beneficial

Key finding: minimum operational closure = 3 instructions (EAT + DIGEST + REFRESH).

### Statistical Tests (`src/stats.rs`)

- `bootstrap_ci(exp, ctrl, n_resamples, seed)` — 95% CI of mean difference
- `mann_whitney_u(exp, ctrl)` — non-parametric test (U, p-value)
- `ks_test(a, b)` — distribution difference (D, p-value)
- `cohens_d(exp, ctrl)` — effect size
- `compare_groups(name, exp, ctrl)` — all tests in one call

### Energy Bucket Tracking

Added to `CellWorld.energy_buckets: [[u64; 4]; 5]`:
- 5 buckets: 0-20%, 20-40%, 40-60%, 60-80%, 80-100% of energy capacity
- 4 counters per bucket: [EAT, REFRESH, DIVIDE, total]

## Module Structure

```
src/
  instruction.rs  — 15 instructions (including GATE)
  organism.rs     — v2 organisms + config
  world.rs        — v2 world engine
  cell_vm.rs      — v3 cell-based VM (main experiment platform)
  experiment.rs   — v2 experiment runner
  stats.rs        — statistical tests
  signal.rs       — signal generation for EXP-011
  knockout.rs     — single-site knockout analysis
  tui.rs          — terminal visualization
  main.rs         — CLI entry point (~2600 lines, all experiment runners)
```

## Key Lessons

1. **CEM=20 kills Cell v3**: default experimental() uses CEM=20 which is too low. Always set CEM=50 explicitly.
2. **food_per_tick=50 is marginal**: use 500 for reliable populations.
3. **sysinfo measures self**: real CPU experiments measure the experiment's own CPU usage. Use hash-based pseudo-CPU for reproducible parallel runs.
4. **GATE is neutral knockout**: it's a cognitive enhancement, not a survival requirement. This is theoretically important.
5. **92% > 100%**: imperfect replication (8% failure from mutation disrupting STORE) is stronger evidence than perfect replication because it shows the mechanism is genuinely evolutionary.
