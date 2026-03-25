# EXP-014: GATE Learning (Data Cell Gene Regulation)

## Design

- GATE instruction: reads adjacent Data cell, if value==0 skips next Code cell
- Seed G: evaluation module (SENSE→EAT→SENSE→CMP→STORE) + GATE→DIVIDE
- Only divides when Data cell > 0 (= energy improved after eating)
- Group 1: abundant (500) first 10k ticks, then scarce (50)
- Group 2: always scarce (50)
- 1M ticks, 100 seeds, CEM=50, data_cell_gating=true

## Results

| Group | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-------|----------|---------|-----------|------|----------|--------|
| Abundant→Scarce | 100/100 | 26.4 | 33.9 | 26.2 | 5.2 | 2.9 |
| Always Scarce | 97/100 | 34.2 | 27.1 | 24.7 | 2.3 | 2.5 |

## Statistical Tests

| Metric | Diff | 95% CI | MW p | d |
|--------|------|--------|------|---|
| EAT | 0.0147 | [0.0008, 0.0294] | 0.0201 | 0.277 |
| REFRESH | 0.0285 | [0.0117, 0.0449] | 0.0000 | 0.459 |
| DIVIDE | 0.0038 | [-0.0018, 0.0097] | 0.0000 | 0.183 |
| Population | -7.7934 | [-9.8216, -5.6616] | 0.0000 | -1.037 |

## Conclusion

GATE mechanism produces measurable behavioral difference between history groups.

---
*EXP-014: GATE learning experiment*
