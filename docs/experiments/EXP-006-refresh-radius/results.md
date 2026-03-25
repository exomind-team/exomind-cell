# EXP-006: Results

## REFRESH Radius Gradient (CEM=50, seed=42, 500k ticks)

| R | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |
|---|-------|----------|---------|-----------|------|---------|----------|--------|
| 1 | Exp | YES | 7.7 | 294.4 | 25.4 | 12.0 | 17.3 | 10.2 |
| 1 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |
| 2 | Exp | YES | 7.7 | 294.4 | 25.4 | 12.0 | 17.3 | 10.2 |
| 2 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |
| 3 | Exp | YES | 11.9 | 84.9 | 36.8 | 16.0 | 11.5 | 9.4 |
| 3 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |
| 5 | Exp | YES | 12.2 | 79.5 | 35.9 | 13.1 | 6.7 | 8.2 |
| 5 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |
| 8 | Exp | YES | 14.3 | 91.8 | 29.3 | 12.4 | 16.2 | 7.9 |
| 8 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |

**Note**: Ctrl is identical across all R values — ctrl organisms ignore R (no freshness decay).

## Interpretation

- R=1,2: Exp REFRESH (17.3%) > Ctrl (14.8%) — constraint felt, REFRESH retained
- R=3: Exp REFRESH (11.5%) < Ctrl (14.8%) — intermediate regime, high EAT instead
- R=5: Exp REFRESH (6.7%) minimal — organisms prefer EAT over REFRESH at this R
- R=8: Exp REFRESH (16.2%) recovers — broader coverage makes REFRESH more efficient
- Energy is highest at R=1,2 (294) — surviving organisms very energy-rich

## Files

- `data/cellR*_exp.csv`, `data/cellR*_ctrl.csv`
- `CELL_RESULTS.md` (Experiment 2 section)
