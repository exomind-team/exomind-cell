# Optimized GATE History Experiment

## Parameters (frozen)

- CEM=50, max=1000, 20G+30A+30B, 2M ticks
- Exp: food=500 first 500k ticks → food=50
- Ctrl: food=50 throughout
- GATE=true, 100 seeds

## Results

Survived: Exp 100/100, Ctrl 92/100

| Group | Avg Pop | Avg Energy | REFRESH% |
|-------|---------|-----------|----------|
| Abundant→Scarce | 18.6 | 56.6 | 12.7 |
| Always Scarce | 31.8 | 26.7 | 3.7 |

## Statistical Tests

- REFRESH: p=0.0000, d=1.117, direction win 82/100 (82%)
- Population: p=0.0000, d=-1.227, direction win 86/100

---
*Optimized GATE history: Paper I final experiment*
