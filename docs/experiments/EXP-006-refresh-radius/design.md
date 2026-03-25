# EXP-006: REFRESH Radius Gradient

## Hypothesis

The REFRESH radius R controls the "cost-benefit" of the REFRESH operation: larger R
refreshes more cells per instruction but requires longer code traversal. There exists
an optimal R that maximizes the operational closure selection pressure.

## Prediction

- Small R (1-2): REFRESH too costly relative to benefit → abandoned by evolution
- Mid R (3-5): Optimal range, maximal exp/ctrl divergence
- Large R (8+): REFRESH too broad → coverage drops, DIVIDE blocked

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 |
| CEM | 50 |
| R variants | 1, 2, 3, 5, 8 |
| freshness_decay | true (exp) vs false (ctrl) |
| seed | 42 |
| total_ticks | 500,000 |

## Status

Complete (`cargo run --release -- --cell`).
