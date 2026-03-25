# EXP-001: Operational Closure vs Control

## Hypothesis

Freshness decay (operational closure constraint) drives the evolution of conditional
survival-priority behavior. Organisms under `freshness_decay=true` will evolve to
execute REFRESH more frequently than organisms under `freshness_decay=false`.

## Prediction

- `exp` (freshness_decay=true): REFRESH ratio > ctrl REFRESH ratio
- `exp`: Higher low-energy EAT rate (conditional survival priority)
- Both groups survive (effect is behavioral, not extinction)

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | v2 (organism-level freshness) |
| freshness_decay | true (exp) vs false (ctrl) |
| max_organisms | 100 |
| food_per_tick | 50 |
| E_MAX | 1000 |
| mutation_rate | 0.001 |
| total_ticks | 500,000 |
| seeds | 42, 137, 256, 999, 2026 |
| Initial pop | 10 Seed A + 10 Seed B |

## Key Variable

`freshness_decay: bool` — the single switch separating exp from ctrl.

## Status

Complete. Large-scale replication: 75/100 seeds (2M ticks) — REPLICATED.
