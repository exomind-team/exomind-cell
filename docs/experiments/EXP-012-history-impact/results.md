# EXP-012: Results

## Summary (10 seeds, 500k ticks)

| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|----------|---------|-----------|------|----------|--------|
| Abundant→Scarce | 10/10 | 25.1 | 31.5 | 26.5 | 8.5 | 9.5 |
| Always Scarce | 10/10 | 24.4 | 32.6 | 29.0 | 8.9 | 8.8 |

## Statistical Tests

| Metric | Diff | 95% CI | MW p | d | Verdict |
|--------|------|--------|------|---|---------|
| EAT | -0.0251 | [-0.0821, 0.0360] | 0.364 | -0.358 | n.s. |
| REFRESH | -0.0038 | [-0.0879, 0.0771] | 0.650 | -0.038 | n.s. |
| DIVIDE | +0.0067 | [-0.0207, 0.0307] | 0.174 | 0.218 | n.s. |
| Population | +0.62 | [-6.3, 6.9] | 0.762 | 0.078 | n.s. |

## Interpretation

- No significant differences between groups on any metric
- Data cell contents do not measurably influence evolved strategies at 10 seeds/500k ticks
- Mechanistic issue: Seed F's LOAD+CMP checks data cell, but mutation disrupts this gene
  circuit before it can be selected for
- History effect may require longer evolution time (>500k ticks) to stabilize

## Files

- `experiment.md` (combined design + results)
- `data/per_seed.csv`
