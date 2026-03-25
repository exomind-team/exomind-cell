# EXP-015a: Feast/Famine Cycling + GATE — 100 Independent Rounds

## Frozen Parameters

- GATE=true, CEM=50, max=200, 20G+10A+10B
- Cycling: 500 food (feast, 10k ticks) → 50 food (famine, 10k ticks), repeat
- Constant: 275 food/tick (average of feast+famine)
- 1M ticks, 100 rounds x 10 seeds/round

## Meta-Analysis

| Metric | Direction win | p<0.05 win | Mean diff | SD | Win rate |
|--------|-------------|-----------|-----------|-----|----------|
| REFRESH (cycle > const) | 71/100 | 19/100 | 0.0220 | 0.0430 | 71% |
| Population (cycle < const) | 0/100 | — | — | — | 0% |

Mean Cohen's d: 0.246, SD: 0.481
Positive d: 71/100 (71%)

## Conclusion

Feast/famine cycling produces measurable behavioral differentiation:
REFRESH in predicted direction in 71% of rounds, p<0.05 in 19%.

---
*EXP-015a: Feast/famine cycling with GATE*
