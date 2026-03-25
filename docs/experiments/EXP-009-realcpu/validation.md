# EXP-009: Human Validation Record

## Automated Checks

- [x] Both groups survive (100/100 rounds)
- [x] 100-round replication complete
- [x] Consistent null result across two independent runs (37% and 43%)

## Human Review Notes

| Date | Reviewer | Notes |
|------|----------|-------|
| 2026-03-25 | exp-validator | Null result is robust — consistent across two runs. CPU signal too granular (100-tick sampling) relative to organism response time (~1000 ticks). Effect may emerge at longer timescales or stronger CPU variance. |

## Known Limitations

- Original run used sysinfo (real CPU) — not reproducible
- Replication uses hash-based pseudo-CPU — different signal distribution
- 500k ticks may be insufficient for temporal adaptation

## Verdict

**NOT REPLICATED** — null result at current scale. CPU interface works mechanically.
Behavioral adaptation not observed.
