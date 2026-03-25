# EXP-002: Multi-Seed Validation

## Hypothesis

EXP-001 results hold across multiple random seeds, ruling out seed-specific artifacts.

## Prediction

- Both exp/ctrl survive all seeds
- Exp REFRESH% consistently higher than ctrl across seeds
- Behavioral divergence is seed-independent

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | v2 (organism-level freshness) |
| freshness_decay | true (exp) vs false (ctrl) |
| E_MAX | 1000 |
| total_ticks | 500,000 |
| seeds | 42, 137, 256, 999, 2026 |
| max_organisms | 100 |
| food_per_tick | 50 |
| Initial pop | 10 Seed A + 10 Seed B |

## Status

Complete (run as part of `--run-v2` mode).
