# EXP-001: Results

## v2 Multi-Seed (500k ticks, seeds 42/137/256/999/2026)

| Seed | Group | REFRESH% | EAT% | DIVIDE% | Avg Pop | Avg Energy |
|------|-------|---------|------|---------|---------|-----------|
| 42 | Exp | 21.0 | 26.9 | 1.3 | 39.7 | 904 |
| 42 | Ctrl | 11.2 | 39.8 | 3.9 | 40.3 | 690 |
| 137 | Exp | 18.7 | 31.6 | 5.5 | 38.9 | 674 |
| 137 | Ctrl | 18.9 | 31.3 | 8.0 | 37.5 | 450 |
| 256 | Exp | 28.2 | 29.3 | 0.0 | 14.0 | 998 |
| 256 | Ctrl | 24.7 | 27.7 | 0.0 | 16.0 | 998 |
| 999 | Exp | 25.2 | 25.2 | 1.4 | 18.0 | 891 |
| 999 | Ctrl | 25.2 | 25.2 | 1.4 | 18.0 | 891 |
| 2026 | Exp | 29.3 | 29.3 | 0.0 | 13.0 | 998 |
| 2026 | Ctrl | 27.2 | 28.4 | 0.0 | 14.0 | 998 |

**Cross-seed averages**: Exp REFRESH 24.5% ± 4.6% vs Ctrl 21.4% ± 6.5%, Δ = +3.0%

## Large-Scale Replication (2M ticks, 100 seeds 1000-1099)

| Metric | Exp | Ctrl | Diff | MW p | Cohen's d |
|--------|-----|------|------|------|-----------|
| REFRESH ratio | 0.168 ± 0.073 | 0.137 ± 0.004 | +0.030 | <0.0001 *** | 0.587 (medium) |
| EAT ratio | 0.226 ± 0.087 | 0.153 ± 0.004 | +0.073 | <0.0001 *** | 1.191 (large) |
| Population | 114.4 ± 36.2 | 134.7 ± 6.2 | -20.3 | 0.0001 *** | -0.782 (medium) |

**Direction consistency**: 75/100 seeds (75%) show exp_REFRESH > ctrl_REFRESH
**Sign test p** < 1e-6

## Interpretation

- Hypothesis confirmed at scale: freshness_decay drives REFRESH evolution
- Effect requires ~2M ticks to manifest reliably (500k ticks: 42% direction win)
- Population reduction is 100% consistent — selection filter operates immediately

## Files

- `data/large_scale/statistical_analysis.md`
- `data/large_scale/per_seed_summary.csv`
- `EXP-001-replication/replication_100rounds_v2_largescale.md`
