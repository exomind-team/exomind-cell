# EXP-003: E_MAX Impact Analysis

## Hypothesis

The energy cap (E_MAX) amplifies the behavioral divergence between exp/ctrl groups.
When organisms cannot store unlimited energy, the freshness constraint creates a
stronger selection pressure for conditional survival behavior.

## Prediction

- E_MAX=1000: Larger exp/ctrl REFRESH delta than E_MAX=unlimited
- E_MAX=unlimited: Organisms can "save up" energy, reducing urgency of EAT

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | v2 |
| freshness_decay | true (exp) vs false (ctrl) |
| seed | 42 |
| total_ticks | 500,000 |
| E_MAX variants | 1000 vs i32::MAX (unlimited) |
| max_organisms | 100 |
| food_per_tick | 50 |

## Status

Complete (run as part of `--run-v2` mode).
