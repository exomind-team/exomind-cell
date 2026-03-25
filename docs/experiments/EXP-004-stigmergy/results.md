# EXP-004: Results

## Summary (seed=42, 100k ticks)

From `docs/experiments.md`:

| Metric | With Medium | Without Medium | Delta |
|--------|-------------|----------------|-------|
| DIVIDE% | higher | baseline | +2.9% |
| REFRESH% | lower | baseline | -6.1% |

## Interpretation

- With medium: Signal-triggered DIVIDE works (+2.9%)
- REFRESH decreases with medium (-6.1%) — organisms trade self-maintenance for reproduction signals
- Medium enables coordination: organisms respond to chemical signals rather than pure energy state

## Note

EXP-004 data files (`data/stigmergy_*.csv`) may not be current — experiment predates Cell v3.
Results summary from `docs/experiments.md` registry only.

## Files

- `data/stigmergy_exp.csv`, `data/stigmergy_ctrl.csv` (if present)
