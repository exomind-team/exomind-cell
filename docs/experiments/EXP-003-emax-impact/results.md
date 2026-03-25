# EXP-003: Results

## E_MAX Comparison (seed=42, 500k ticks)

| Metric | E_MAX=1000 Exp | E_MAX=1000 Ctrl | E_MAX=unlim Exp | E_MAX=unlim Ctrl |
|--------|---------------|----------------|----------------|------------------|
| EAT% | 26.9 | 39.8 | 36.1 | 41.9 |
| REFRESH% | 21.0 | 11.2 | 13.3 | 11.4 |
| DIVIDE% | 1.3 | 3.9 | 6.2 | 3.8 |
| Avg Pop | 40 | 40 | 23 | 22 |
| Avg Energy | 904 | 690 | 271,869 | 374,877 |

## REFRESH Delta

| E_MAX | Exp REFRESH | Ctrl REFRESH | Delta |
|-------|------------|-------------|-------|
| 1000 | 21.0% | 11.2% | **+9.8%** |
| Unlimited | 13.3% | 11.4% | **+1.9%** |

## Interpretation

- E_MAX=1000 produces 5x larger REFRESH delta than unlimited
- With unlimited energy, freshness constraint less urgent (organisms buffer against decay)
- Energy cap is a key amplifier of operational closure selection pressure
- Unlimited E_MAX: massive average energy (270k), organisms effectively immortal via EAT

## Files

- `data/v3_emax_unlimited_exp.csv`, `data/v3_emax_unlimited_ctrl.csv`
- `RESULTS.md` (E_MAX Impact Analysis section)
