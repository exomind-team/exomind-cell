# EXP-011: Human Validation Record

## Automated Checks

- [x] All 6 groups survive
- [x] 100-round replication complete
- [x] A vs F direction reversed (unexpected — F more stable)

## Human Review Notes

| Date | Reviewer | Notes |
|------|----------|-------|
| 2026-03-25 | exp-validator | A vs F reversal: Group F (no signal) has constant 0.5×base food = most stable supply. Group A with delta=200 delay has variable food that lags behind signal, creating predictable but variable supply. Organism adaptation to prediction requires longer evolution time. |

## Known Limitations

- Seed F genome (SAMPLE → CMP) is hardcoded — mutation may destroy signal-reading
- 500k ticks may be insufficient for signal-conditional strategy to evolve
- Single-seed per round in 100-round version

## Verdict

**NOT REPLICATED** — predictive signal advantage not observed at current scale.
F group's supply stability dominates. Longer runs or stronger signal correlation needed.
