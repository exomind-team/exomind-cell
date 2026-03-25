# EXP-014: GATE Learning (Data Cell Gene Regulation)

## Design

- GATE instruction: reads adjacent Data cell, if value==0 skips next Code cell
- Seed G: evaluation module (SENSE→EAT→SENSE→CMP→STORE) + GATE→DIVIDE
- Only divides when Data cell > 0 (= energy improved after eating)
- Group 1: abundant (500) first 10k ticks, then scarce (50)
- Group 2: always scarce (50)
- 1M ticks, 10 seeds, CEM=50, data_cell_gating=true

## Results

| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|----------|---------|-----------|------|----------|--------|
| Abundant→Scarce | 10/10 | 24.8 | 38.1 | 24.5 | 7.3 | 2.7 |
| Always Scarce | 10/10 | 36.5 | 26.9 | 23.8 | 1.0 | 2.2 |

## Statistical Tests

| Metric | Diff | 95% CI | MW p | d |
|--------|------|--------|------|---|
| EAT | 0.0068 | [-0.0214, 0.0354] | 0.5967 | 0.198 |
| REFRESH | 0.0633 | [0.0141, 0.1196] | 0.0233 | 0.980 |
| DIVIDE | 0.0045 | [-0.0060, 0.0179] | 0.1736 | 0.304 |

## Conclusion

GATE mechanism does not yet produce significant history-dependent behavior at this scale.

---
*EXP-014: GATE learning experiment*
