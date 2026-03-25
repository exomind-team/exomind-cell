# EXP-010: Multi-Food Type Experiment (Value Evaluation, D1)

## Hypothesis

Freshness decay (operational closure) drives selective feeding behavior when multiple food types are available: organisms under survival pressure will develop different food utilization patterns compared to organisms without decay.

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 |
| CEM | 50 |
| R | 5 |
| freshness_max | 255 |
| total_ticks | 500,000 |
| seed | 42 |
| max_organisms | 100 |
| Simple food | energy=5, rate=30/tick |
| Complex food | energy=20, rate=10/tick |

## Groups

1. **Multi-food Exp**: freshness_decay=true, both food types
2. **Multi-food Ctrl**: freshness_decay=false, both food types
3. **Simple-only Exp**: freshness_decay=true, only standard food (food_per_tick=50)

## Predictions

1. Multi-food Exp group will show higher REFRESH than Multi-food Ctrl (operational closure signature persists)
2. Multi-food groups will sustain larger populations than simple-only (more total energy available)
3. Selective feeding (distinguishing simple vs complex) requires organisms to sense food type -- not possible in current random-pick implementation

## Falsification

If Multi-food Exp REFRESH% equals Multi-food Ctrl REFRESH% --> operational closure does not affect behavior even with food variety

## Data Files

- `raw/cell_multifood_exp.csv`
- `raw/cell_multifood_ctrl.csv`
- `raw/cell_simplefood_exp.csv`
