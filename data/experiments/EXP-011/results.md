# EXP-011: Sense-Making (Signal Prediction) Results

## Design

- Signal written to medium[0], food tracks signal with delta-tick delay
- Organisms with SAMPLE can read signal, organisms without cannot
- Seed F: SAMPLE → STORE → EAT → DIGEST → LOAD → CMP → conditional extra EAT

## Summary (30 seeds per group)

| Group | Signal | Delta | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|--------|-------|----------|---------|-----------|------|----------|--------|
| A_square_d200 | Square | 200 | 30/30 | 11.4 | 60.5 | 28.9 | 17.0 | 5.7 |
| B_sine_d200 | Sine | 200 | 30/30 | 13.8 | 50.6 | 31.7 | 13.0 | 7.2 |
| C_realcpu_d200 | Real CPU | 200 | 30/30 | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| D_square_d0 | Square | 0 | 30/30 | 14.2 | 51.8 | 29.1 | 14.3 | 7.0 |
| E_random_d200 | Random | 200 | 30/30 | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| F_nosignal | None | 0 | 30/30 | 4.3 | 94.2 | 25.1 | 24.1 | 0.7 |

## Statistical Comparisons

| Comparison | Metric | Diff | 95% CI | MW p | d | KS D | KS p |
|-----------|--------|------|--------|------|---|------|------|
| A(d200) vs D(d0) | EAT | -0.0025 | [-0.0363, 0.0333] | 0.4333 | -0.037 | 0.433 | 0.0046 |
| A(predict) vs E(random) | EAT | 0.0081 | [-0.0290, 0.0465] | 0.5844 | 0.109 | 0.267 | 0.2005 |
| A(signal) vs F(none) | EAT | 0.0377 | [0.0127, 0.0652] | 0.0298 | 0.723 | 0.533 | 0.0002 |

---
*EXP-011: Sense-making signal prediction experiment*
