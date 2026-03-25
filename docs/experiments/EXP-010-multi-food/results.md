# EXP-010: Results

## Multi-Food Type (CEM=50, seed=42, 500k ticks)

| Setup | Group | Survived | Pop | Energy | EAT% | REFRESH% | DIVIDE% |
|-------|-------|----------|-----|--------|------|----------|--------|
| Multi-food | Exp | YES | 100.0 | 136.7 | 12.5 | 22.5 | 10.0 |
| Multi-food | Ctrl | YES | 100.0 | 140.3 | 13.8 | 13.8 | 11.1 |
| Simple-only | Exp | YES | 12.2 | 79.5 | 35.9 | 6.7 | 8.2 |

## Key Comparisons

### Multi-food vs Simple-only (Exp group)

| Metric | Multi-food | Simple-only | Delta |
|--------|-----------|-------------|-------|
| Population | 100 | 12.2 | +87.8 |
| Avg Energy | 136.7 | 79.5 | +57.2 |
| EAT% | 12.5 | 35.9 | -23.4 |
| REFRESH% | 22.5 | 6.7 | +15.8 |

### Multi-food: Exp vs Ctrl

| Metric | Exp | Ctrl | Delta |
|--------|-----|------|-------|
| Population | 100 | 100 | 0 |
| Avg Energy | 136.7 | 140.3 | -3.6 |
| EAT% | 12.5 | 13.8 | -1.3 |
| REFRESH% | 22.5 | 13.8 | +8.7 |

## Interpretation

- Multi-food dramatically increases population (12 → 100) due to higher energy density
- REFRESH higher in exp than ctrl within multi-food (+8.7%) — freshness constraint active
- Population hits cap (100) in multi-food — food is abundant enough to saturate
- EAT drops in multi-food (12% vs 36%) because complex food provides 4x energy per EAT

## Files

- `data/cell_multifood_exp.csv`, `data/cell_multifood_ctrl.csv`
- `data/cell_simplefood_exp.csv`
- `CELL_RESULTS.md` (Experiment 5 section)
