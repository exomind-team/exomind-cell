# EXP-008: Results

## Growth (ALLOC) vs Baseline (CEM=50, seed=42, 500k ticks)

| Setup | Group | Survived | Avg Pop | Avg Cells | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|-------|----------|---------|----------|-----------|------|----------|--------|
| With ALLOC | Exp | YES | 22.3 | 11.1 | 30.6 | 25.8 | 12.3 | 9.6 |
| With ALLOC | Ctrl | YES | 24.1 | 13.7 | 58.5 | 16.5 | 12.6 | 8.1 |
| No ALLOC | Exp | YES | 12.2 | N/A | 79.5 | 35.9 | 6.7 | 8.2 |

## Key Comparisons

| Metric | With ALLOC Exp | No ALLOC Exp | Delta |
|--------|---------------|-------------|-------|
| Avg Pop | 22.3 | 12.2 | +10.1 |
| Avg Energy | 30.6 | 79.5 | -49.0 |
| EAT% | 25.8 | 35.9 | -10.1 |
| REFRESH% | 12.3 | 6.7 | +5.6 |
| DIVIDE% | 9.6 | 8.2 | +1.4 |

## Interpretation

- With ALLOC: Higher population (22 vs 12) — body growth enables more stable survival
- With ALLOC: Lower energy per organism (30 vs 79) — energy distributed across more cells
- Cell count varies: 11.1 (Exp) vs 13.7 (Ctrl) — freshness decay constrains growth
- ALLOC organisms reduce EAT frequency (distributed storage buffers fluctuations)

## Files

- `data/cell_growth_exp.csv`, `data/cell_growth_ctrl.csv`
- `data/cell_nogrow_exp.csv`
- `CELL_RESULTS.md` (Experiment 4 section)
