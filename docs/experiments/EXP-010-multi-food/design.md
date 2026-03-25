# EXP-010: Multi-Food Type (D1 Value Evaluation)

## Hypothesis

When organisms face two food types (simple: low energy, instant vs complex: high energy,
requires extra DIGEST), those with freshness decay will develop differential food
preference, preferring complex food when energy allows.

## Prediction

- With multi-food + freshness_decay: Higher complex food uptake ratio
- Without freshness_decay (ctrl): No preference differentiation
- Multi-food environment is a prerequisite for value evaluation behavior (D1)

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 |
| CEM | 50, R=5 |
| seed | 42 |
| total_ticks | 500,000 |
| Simple food | energy=5, instant (no DIGEST needed) |
| Complex food | energy=20, requires DIGEST |
| multi_food | true |
| food_per_tick | 0 (replaced by multi_food injection) |
| freshness_decay | true (exp) vs false (ctrl) |

## Status

Complete (`cargo run --release -- --cell`).
