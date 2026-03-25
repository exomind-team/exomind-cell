# EXP-006: Human Validation Record

## Automated Checks

- [x] All R values survive both groups
- [x] Ctrl is stable across all R (expected — no freshness decay)
- [x] Non-monotonic REFRESH vs R pattern observed

## Human Review Notes

| Date | Reviewer | Notes |
|------|----------|-------|
| 2026-03-25 | exp-validator | Non-monotonic pattern interesting but single seed. R=5 chosen for standard experiments — reasonable choice given intermediate behavior. |

## Verdict

**CONFIRMED** (single seed). R=5 is the frozen standard. No replication needed for parameter scan.
