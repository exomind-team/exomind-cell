# EXP-011: Results

## Parameters

Cell v3, CEM=50, R=5, 500k ticks, 30 seeds/group, max_org=200, rayon parallel.

## Group Steady-State Means (tick 250k–500k)

| Group | Signal | Pop | Energy | EAT% | REFRESH% | DIVIDE% |
|-------|--------|-----|--------|------|----------|---------|
| A | Square wave T=2000, δ=200 | 11.4 | 60.5 | 28.9 | 17.0 | 5.7 |
| B | Sine wave T=2000, δ=200 | 13.8 | 50.6 | 31.7 | 13.0 | 7.2 |
| C | CPU hash, δ=200 | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| D | Square wave T=2000, δ=0 | 14.2 | 51.8 | 29.1 | 14.3 | 7.0 |
| E | Random, δ=200 | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| F | No signal | 4.3 | 94.2 | 25.1 | 24.1 | 0.7 |

## Statistical Tests

| Comparison | Mann-Whitney p | Cohen's d | Result |
|-----------|----------------|-----------|--------|
| A vs D (predict vs sync) | 0.43 | -0.04 | Not significant |
| A vs E (predict vs random) | 0.58 | 0.11 | Not significant |
| A vs F (signal vs none) | **0.03** | **0.72** | **Significant** |

## Interpretation

1. **Signal presence matters** (A vs F: p=0.03, d=0.72): Signal modulation significantly alters ecological structure. Group F (no signal) has lower population but higher per-organism energy — fewer organisms in a stable environment with no food variability.

2. **Signal predictability not yet exploited** (A vs D: p=0.43, A vs E: p=0.58): Organisms cannot distinguish predictable square wave from synchronous or random signals at 500k ticks. No selective advantage for prediction ability.

3. **Sensitivity-to-Prediction Gap**: This marks the boundary between D0 (operational closure) and D1 (sense-making). Organisms are *sensitive* to signal presence but do not *predict* from signal structure.

## Files

- `experiment.md` (detailed multi-group report)
- `validation.md`
- `data/` (per-seed CSVs by group)
