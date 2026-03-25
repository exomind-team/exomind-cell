# EXP-014: GATE Learning — 100 Independent Rounds

## Design

- GATE instruction + Seed G (evaluation + gated DIVIDE)
- 100 independent rounds, 10 seeds per round, 1M ticks each
- Each round: independent seed set, independent population, per-round p-value

## Meta-Analysis

| Metric | Direction win | p<0.05 wins | Mean diff | SD | Win rate |
|--------|-------------|------------|-----------|-----|----------|
| REFRESH | 92/100 | 68/100 | 0.0375 | 0.0272 | 92% |
| EAT | 74/100 | — | 0.0166 | 0.0248 | 74% |
| Population | 97/100 | — | -7.9 | 3.9 | 97% |

## Effect Size Distribution (REFRESH Cohen's d across rounds)

- Mean d: 0.552
- SD d: 0.405
- Positive d: 92/100 (92%)

## Conclusion

Across 100 truly independent experiments (each with 10 seeds, independent initialization, 1M ticks):
- REFRESH effect in predicted direction: **92%** of rounds
- REFRESH p<0.05 in predicted direction: **68%** of rounds
- Population effect: **97%** of rounds

---
*EXP-014: 100 independent rounds meta-analysis*
