# EXP-014: Results

## 100-Round Meta-Analysis (10 seeds/round, 1M ticks)

| Metric | Direction win | p<0.05 wins | Mean diff | SD | Mean d |
|--------|-------------|------------|-----------|-----|--------|
| REFRESH | **92/100 (92%)** | 68/100 | +0.0375 | 0.0272 | 0.552 |
| EAT | 74/100 (74%) | — | +0.0166 | 0.0248 | — |
| Population | 97/100 (97%) | — | -7.9 | 3.9 | — |

## Comparison with EXP-012

| Feature | EXP-012 (LOAD+CMP) | EXP-014 (GATE) |
|---------|-------------------|----------------|
| Mechanism | Manual load → compare → branch | Single GATE instruction |
| REFRESH win rate | n.s. (null) | 92% |
| Circuit stability | Low (multi-instruction, fragile) | High (single instruction) |
| Ticks | 500k | 1M |

## Interpretation

- GATE instruction as gene regulatory switch is robust: 92% replication rate
- Population effect (97%) shows abundance history creates lasting ecological niche
- Mean REFRESH d=0.552 (medium effect) — comparable to EXP-001 large-scale
- 8% non-replication: failure analysis in `failure_analysis.md`

## Files

- `experiment.md` (100-round meta-analysis report)
- `failure_analysis.md` (8% failure modes)
- `data/per_round.csv`
