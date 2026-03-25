# EXP-010 Results: Multi-Food Type Experiment

## Summary

| Setup | Group | Pop | Energy | EAT% | REFRESH% | DIVIDE% |
|-------|-------|-----|--------|------|----------|---------|
| Multi-food | Exp | 100 | 137 | 12.5 | **22.5** | 10.0 |
| Multi-food | Ctrl | 100 | 140 | 13.8 | **13.8** | 11.1 |
| Simple-only | Exp | 12 | 80 | 35.9 | 6.7 | 8.2 |

## Analysis

1. **REFRESH signature persists**: Exp 22.5% vs Ctrl 13.8% (+8.7%). Operational closure drives higher REFRESH even with abundant food.

2. **Population at max**: Both multi-food groups saturated at max_organisms=100. Total food input (30*5 + 10*20 = 350 energy/tick) sustains full population.

3. **Simple-only scarcity**: Standard food (50/tick) only supports 12 organisms, confirming that multi-food dramatically increases carrying capacity.

4. **No selective feeding observed**: Current EAT picks food randomly (50/50). Organisms cannot sense or choose food type. Selective feeding would require a SENSE_FOOD instruction (future work).

## Conclusion

Operational closure signature (REFRESH > Ctrl) is robust across food environments: scarce single-food, abundant multi-food. The constraint is structural, not resource-dependent.

Selective feeding behavior (D1 value evaluation) requires additional VM capabilities (food type sensing). This is a natural next step.
