# EXP-009: CPU-Modulated Food — 100 Independent Rounds

## Frozen Parameters

- food_per_tick=50 (baseline) + 500 injection per 100 ticks
- CPU floor: 30%, formula: food += 500 * (0.3 + 0.7*(1-cpu))
- Control: constant 55/tick
- CEM=50, max=200, 20A+20B, 500k ticks
- 100 rounds (seeds 6000-6099), hash-based pseudo-CPU

## Results

Survived: CPU 100/100, Ctrl 100/100

| Metric | CPU-modulated | Constant | Diff | p | d |
|--------|-------------|----------|------|---|---|
| EAT% | 25.4 | 29.0 | -0.0354 | 0.0019 | -0.433 |
| REFRESH% | 8.0 | 8.4 | -0.0047 | 0.1728 | -0.059 |
| Population | 26.7 | 29.9 | -3.3 | 0.0000 | -0.711 |

## Direction Win Rate

- EAT (cpu > ctrl): 37/100 (37%)

---
*EXP-009: 100 independent rounds with frozen parameters*
