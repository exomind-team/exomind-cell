# Parameter Analysis: Why Some Experiments Have Mass Extinction

## The Problem

Several experiment replications show populations collapsing to 0-1 survivors, with food_pool accumulating to millions. The original 100-seed experiment (EXP, `--stats`) succeeded. Why?

## Parameter Comparison

| Parameter | Original 100-seed (SUCCESS) | EXP-014 gate trial | Default experimental() | EXP-009 replication |
|-----------|---------------------------|--------------------|-----------------------|---------------------|
| **max_organisms** | **1000** | 200 | 100 | 200 |
| **food_per_tick** | **500** | 50 (scarce) / 500 (abundant) | 50 | ~3 (CPU modulated) |
| **cell_energy_max** | **50** | 50 | **20** | 50 |
| **initial organisms** | **50A + 50B** | 20G + 10A + 10B | 10A + 10B | 20A + 20B |
| total_ticks | 2,000,000 | 1,000,000 | 500,000 | 500,000 |
| freshness_decay | true | true | true | true |
| refresh_radius | 5 | 5 | 5 | 5 |

## Root Causes

### 1. Default CEM=20 is too low for Cell v3 survival

The default `CellConfig::experimental()` uses `cell_energy_max = 20`. At CEM=20, each Energy cell holds max 20 energy. Seed A has 2 Energy cells = 40 max energy. With instruction_cost=1, that's only 40 instructions before needing food. The 4-instruction loop (EAT+DIGEST+REFRESH+JMP) costs 4 energy/loop + 1 refresh_cost = 5/loop. So 8 loops before starvation. If food_pool is empty for even a few ticks, organism dies.

The successful experiment uses CEM=50 explicitly.

### 2. food_per_tick=50 is marginal for Cell v3

With 20 organisms eating, each consuming ~5 energy/loop and eating every 4 ticks: 20 * 5 / 4 = 25 food/tick consumed. food_per_tick=50 barely covers this with a 2x margin. Any spike in population (from DIVIDE) creates temporary starvation.

The successful experiment uses food_per_tick=500 with max_organisms=1000.

### 3. EXP-009 replication used food_per_tick ~3 (too low)

The CPU-modulated experiment's base_food=300 per 100 ticks = 3/tick average. With 30% floor = 0.9/tick minimum. This is catastrophically low for 40 organisms.

The original successful EXP-009 used food_per_tick=50 baseline + 300 bonus per 10 ticks = 50 + 30 = 80 effective food/tick.

### 4. Initial population matters

The successful experiment started with 100 organisms (50A+50B). Replication runs started with 20-40. Larger initial populations have more genetic diversity and are more resilient to early stochastic die-offs.

## Fix Recommendations

| Experiment | Fix |
|-----------|-----|
| Default CellConfig | Change CEM from 20 to 50 |
| EXP-009 replication | Match original: food_per_tick=50, base_food injection per 10 ticks |
| EXP-014 | Already using CEM=50, food_per_tick=50 for scarce (correct) |
| All Cell v3 experiments | Use food_per_tick >= 50, CEM >= 50, max_organisms >= 200 |

## The Successful Original Parameters

For reference, the parameters that produced ALL p<0.001:
```
cell_energy_max = 50
max_organisms = 1000
food_per_tick = 500
total_ticks = 2_000_000
initial: 50 Seed A + 50 Seed B
freshness_max = 255
refresh_radius = 5
mutation_rate = 0.001
```

These should be the baseline for any Cell v3 experiment going forward.
