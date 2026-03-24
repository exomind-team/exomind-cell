# Experiment Index

## Standard Parameters

| Parameter | Value |
|-----------|-------|
| Population cap | 100 |
| Initial organisms | 10 Seed A + 10 Seed B |
| Food per tick | 50 |
| Mutation rate | 0.001 |
| Total ticks | 100,000 |
| Freshness max | 255 |
| E_MAX | 1000 |
| Eat energy | 10 |
| Instruction cost | 1 |
| Refresh cost | 1 |
| Divide cost | 30 |
| Initial energy | 100 |
| Snapshot interval | 1,000 ticks |
| Genome dump interval | 10,000 ticks |

## Experiment 1: Operational Closure (5 seeds)

**Question**: Does freshness decay drive evolution of self-maintenance behavior?

| File | Group | Seed | Description |
|------|-------|------|-------------|
| `experimental_seed_42.csv` | Experimental | 42 | freshness_decay = true |
| `control_seed_42.csv` | Control | 42 | freshness_decay = false |
| `experimental_seed_137.csv` | Experimental | 137 | freshness_decay = true |
| `control_seed_137.csv` | Control | 137 | freshness_decay = false |
| `experimental_seed_256.csv` | Experimental | 256 | freshness_decay = true |
| `control_seed_256.csv` | Control | 256 | freshness_decay = false |
| `experimental_seed_999.csv` | Experimental | 999 | freshness_decay = true |
| `control_seed_999.csv` | Control | 999 | freshness_decay = false |
| `experimental_seed_2026.csv` | Experimental | 2026 | freshness_decay = true |
| `control_seed_2026.csv` | Control | 2026 | freshness_decay = false |

**Result**: REFRESH ratio is 19.7% (+/- 1.0%) in experimental vs 13.9% (+/- 5.4%) in control. Freshness decay maintains REFRESH under positive selection.

## Experiment 2: Food Competition (7 levels)

**Question**: How does food scarcity affect population dynamics and behavior?

| File | Food/tick | Description |
|------|-----------|-------------|
| `competition_food_10.csv` | 10 | Extreme scarcity |
| `competition_food_20.csv` | 20 | High scarcity |
| `competition_food_30.csv` | 30 | Moderate scarcity |
| `competition_food_40.csv` | 40 | Reproduction threshold |
| `competition_food_50.csv` | 50 | Standard (same as Exp 1) |
| `competition_food_75.csv` | 75 | Moderate abundance |
| `competition_food_100.csv` | 100 | High abundance |

**Result**: food_per_tick ~40 is the threshold where DIVIDE first appears. Below 40, 0% DIVIDE (pure survival). Above 40, 6-8% DIVIDE.

## Genome Dumps

Each experiment has a corresponding `*_genomes.txt` file containing the code of the oldest and most-evolved organisms at 10k-tick intervals.

## CSV Column Reference

```
tick, population, avg_energy, avg_code_length, avg_age, avg_freshness,
total_eat, total_refresh, total_divide, total_instructions,
eat_ratio, refresh_ratio, divide_ratio,
low_energy_eat_rate, low_freshness_refresh_rate, max_generation
```

All ratios are computed over the 1000-tick snapshot interval.

## Full Analysis

See [RESULTS.md](../RESULTS.md) for the complete analysis with cross-seed statistics and interpretation.
