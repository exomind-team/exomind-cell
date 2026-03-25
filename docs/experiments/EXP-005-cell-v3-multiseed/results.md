# EXP-005: Results

## Cell v3 Multi-Seed (CEM=50, R=5, 500k ticks)

| Seed | Group | Survived | Avg Pop | Avg Energy | EAT% | DIGEST% | REFRESH% | DIVIDE% |
|------|-------|----------|---------|-----------|------|---------|----------|--------|
| 42 | Exp | YES | 12.2 | 79.5 | 35.9 | 13.1 | 6.7 | 8.2 |
| 42 | Ctrl | YES | 26.0 | 55.6 | 15.8 | 14.2 | 14.8 | 8.0 |
| 137 | Exp | YES | 7.2 | 122.5 | 24.0 | 14.6 | 3.9 | 10.4 |
| 137 | Ctrl | YES | 26.7 | 62.7 | 16.0 | 14.2 | 13.6 | 7.5 |
| 256 | Exp | YES | 1.0 | 98.0 | 25.0 | 25.0 | 25.0 | 0.0 |
| 256 | Ctrl | YES | 23.4 | 66.0 | 16.4 | 14.8 | 12.7 | 8.0 |
| 999 | Exp | YES | 13.0 | 88.5 | 34.4 | 12.6 | 21.5 | 8.9 |
| 999 | Ctrl | YES | 27.1 | 66.8 | 14.7 | 14.8 | 13.8 | 7.2 |
| 2026 | Exp | YES | 8.0 | 116.3 | 24.8 | 14.0 | 1.1 | 12.1 |
| 2026 | Ctrl | YES | 26.7 | 64.0 | 14.6 | 14.2 | 14.2 | 7.5 |

## Key Observations

- **Ctrl REFRESH is remarkably stable**: 12.7–14.8% across all seeds (SD < 1%)
- **Exp REFRESH highly variable**: 1.1–25.0% across seeds (SD ≈ 9%)
- **Exp population smaller**: consistently lower than ctrl (selection pressure visible)
- **Exp energy higher**: organisms that survive are more energy-efficient

## Files

- `data/cell50_exp_*.csv`, `data/cell50_ctrl_*.csv`
- `CELL_RESULTS.md` (Experiment 1 section)
