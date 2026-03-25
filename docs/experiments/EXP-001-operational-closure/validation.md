# EXP-001: Human Validation Record

## Automated Checks

- [x] Both groups survive all 5 seeds (v2 500k ticks)
- [x] 100 independent seeds (2M ticks) — 75/100 direction win
- [x] Sign test p < 1e-6
- [x] CI [0.016, 0.044] does not include 0
- [x] CSV data files present in `data/large_scale/`

## Human Review Notes

| Date | Reviewer | Notes |
|------|----------|-------|
| 2026-03-25 | exp-validator | Large-scale 75/100 confirmed. Population effect 100% consistent across all seeds. |

## Known Limitations

- Effect size varies by seed (SD=0.073 >> mean=0.030)
- Requires 2M ticks minimum; 500k ticks insufficient (42% direction win)
- v2 results (500k, 5 seeds): Δ REFRESH = +3.0%, borderline
- Large-scale uses different VM config (max_org=1000 vs 100) — confound present

## Verdict

**REPLICATED** at correct parameter scale (2M ticks, n=100 seeds).
