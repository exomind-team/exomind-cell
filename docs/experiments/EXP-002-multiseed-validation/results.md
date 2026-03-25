# EXP-002: Results

## Summary (500k ticks, steady-state tick 250k-500k)

| Seed | Group | Survived | REFRESH% | EAT% | DIVIDE% | Avg Pop | Avg Energy |
|------|-------|----------|---------|------|---------|---------|-----------|
| 42 | Exp | YES | 21.0 | 26.9 | 1.3 | 39.7 | 904 |
| 42 | Ctrl | YES | 11.2 | 39.8 | 3.9 | 40.3 | 690 |
| 137 | Exp | YES | 18.7 | 31.6 | 5.5 | 38.9 | 674 |
| 137 | Ctrl | YES | 18.9 | 31.3 | 8.0 | 37.5 | 450 |
| 256 | Exp | YES | 28.2 | 29.3 | 0.0 | 14.0 | 998 |
| 256 | Ctrl | YES | 24.7 | 27.7 | 0.0 | 16.0 | 998 |
| 999 | Exp | YES | 25.2 | 25.2 | 1.4 | 18.0 | 891 |
| 999 | Ctrl | YES | 25.2 | 25.2 | 1.4 | 18.0 | 891 |
| 2026 | Exp | YES | 29.3 | 29.3 | 0.0 | 13.0 | 998 |
| 2026 | Ctrl | YES | 27.2 | 28.4 | 0.0 | 14.0 | 998 |

## Cross-Seed Averages

| Metric | Experimental | Control | Delta |
|--------|-------------|---------|-------|
| EAT% | 28.4% ± 2.5% | 30.5% ± 5.7% | -2.0% |
| REFRESH% | 24.5% ± 4.6% | 21.4% ± 6.5% | +3.0% |
| DIVIDE% | 1.7% ± 2.3% | 2.7% ± 3.4% | -1.0% |

## Interpretation

- Exp REFRESH marginally higher (+3.0% Δ) but high variance obscures signal at n=5
- Seeds 999 and 2026 show identical exp/ctrl (convergent dynamics, no selection differential)
- Seed 42 shows clearest divergence: REFRESH 21% (exp) vs 11% (ctrl)

## Files

- `data/v3_exp_42.csv`, `data/v3_ctrl_42.csv` (and other seeds)
- `RESULTS.md` (combined report)
