# EXP-012: History Impact (D2 Experience Learning)

## Design

- Group 1: abundant food (500/tick) for first 10k ticks, then scarce (50/tick)
- Group 2: scarce food (50/tick) from start
- Same genomes (Seed F with Data cell), same seeds
- If history matters: Group 1 behavior differs from Group 2 after switch

## Results (10 seeds per group)

| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|----------|---------|-----------|------|----------|--------|
| Abundant→Scarce | 10/10 | 25.1 | 31.5 | 26.5 | 8.5 | 9.5 |
| Always Scarce | 10/10 | 24.4 | 32.6 | 29.0 | 8.9 | 8.8 |

## Statistical Tests

| Metric | Diff | 95% CI | MW p | d | KS D | KS p |
|--------|------|--------|------|---|------|------|
| EAT | -0.0251 | [-0.0821, 0.0360] | 0.3643 | -0.358 | 0.300 | 0.7060 |
| REFRESH | -0.0038 | [-0.0879, 0.0771] | 0.6501 | -0.038 | 0.200 | 1.0000 |
| DIVIDE | 0.0067 | [-0.0207, 0.0307] | 0.1736 | 0.218 | 0.400 | 0.3141 |
| Population | 0.6215 | [-6.3478, 6.9606] | 0.7624 | 0.078 | 0.200 | 1.0000 |

## Conclusion

History does NOT measurably affect behavior in current design — Data cell contents do not significantly influence evolved strategies at this scale.

---
*EXP-012: History impact experiment*
