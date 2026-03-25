# EXP-007: Results

## Data Cell vs No Data Cell (CEM=50, seed=42, 500k ticks)

| Setup | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |
|-------|-------|----------|---------|-----------|------|---------|----------|--------|
| With Data | Exp | YES | 15.0 | 105.6 | 34.6 | 21.5 | 2.1 | 9.0 |
| With Data | Ctrl | YES | 26.4 | 59.9 | 16.2 | 14.2 | 13.7 | 7.7 |
| No Data | Exp | YES | 12.2 | 79.5 | 35.9 | 13.1 | 6.7 | 8.2 |
| No Data | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |

## Key Differences (With Data vs No Data, Exp group)

| Metric | With Data | No Data | Delta |
|--------|-----------|---------|-------|
| Avg Pop | 15.0 | 12.2 | +2.8 |
| Avg Energy | 105.6 | 79.5 | +26.1 |
| DIGEST% | 21.5 | 13.1 | +8.4 |
| REFRESH% | 2.1 | 6.7 | -4.6 |

## Interpretation

- With Data: Higher energy (105 vs 79) — energy tracking improves resource acquisition
- DIGEST increases (+8.4%) — Data cell organisms process food more efficiently
- REFRESH decreases (-4.6%) — energy tracking reduces unnecessary REFRESH calls
- Preliminary evidence for experience-based optimization, single seed

## Files

- `data/cell_data_exp.csv`, `data/cell_data_ctrl.csv`
- `data/cell_nodata_exp.csv`, `data/cell_nodata_ctrl.csv`
- `CELL_RESULTS.md` (Experiment 3 section)
