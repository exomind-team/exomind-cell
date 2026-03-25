# EXP-009: Results

## 100-Round Replication (hash-based pseudo CPU, 500k ticks)

From `EXP-009-replication/replication_100rounds.md`:

| Metric | Value |
|--------|-------|
| EAT diff > 0 (cpu > const) | 43/100 (43%) |
| Sign test p | 0.10 |
| Mean EAT diff | -0.0009 |
| MW p | 0.97 |
| Cohen's d | -0.012 |

## Earlier Run (100 seeds, seeds 6000-6099)

From `data/per_round.csv` in this directory:

| Metric | Value |
|--------|-------|
| EAT diff > 0 | 37/100 (37%) |
| Mean EAT diff | -0.035 |

## Interpretation

- CPU variability has negligible effect on EAT rate (d ≈ -0.01)
- Organisms do not adapt burst-feeding to CPU fluctuations at 500k tick scale
- Result consistent across two independent 100-round runs
- Hypothesis not confirmed: variable food does not drive higher EAT adaptation

## Files

- `data/per_round.csv` — earlier 100-round run
- `EXP-009-replication/replication_100rounds.md` — most recent 100-round run
