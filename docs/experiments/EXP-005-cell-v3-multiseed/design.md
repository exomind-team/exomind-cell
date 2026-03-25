# EXP-005: Cell v3 Multi-Seed (CEM=50)

## Hypothesis

Cell v3 (per-cell freshness) produces qualitatively different behavioral dynamics
than v2 (organism-level freshness), with higher variance in evolved strategies.

## Prediction

- Exp group (freshness_decay=true) shows high REFRESH variance across seeds
- Ctrl group (freshness_decay=false) shows low, consistent REFRESH (~14%)
- Per-cell freshness creates stronger and more varied selection pressure

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 (per-cell freshness) |
| CEM (cell_energy_max) | 50 |
| refresh_radius (R) | 5 |
| total_ticks | 500,000 |
| seeds | 42, 137, 256, 999, 2026 |
| freshness_decay | true (exp) vs false (ctrl) |
| max_organisms | 100 |
| food_per_tick | 50 |
| Initial pop | 10 Seed A + 10 Seed B |

## Status

Complete (`cargo run --release -- --cell`).
