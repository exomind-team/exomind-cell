# D0 Virtual Machine — Multi-Seed Experiment Results (v2)

## Changes from v1

- **E_MAX = 1000**: Energy cap prevents unbounded accumulation
- **5 random seeds**: Statistical validation across seeds 42, 137, 256, 999, 2026
- **Genome dumps**: Both oldest and most-evolved organisms saved every 10k ticks

## Multi-Seed Summary (steady-state averages, tick 50k-100k)

### Per-Seed Results

| Seed | Group | Survived | EAT% | REFRESH% | DIVIDE% | Low-E EAT% | Avg Pop | Avg Energy |
|------|-------|----------|------|----------|---------|-----------|---------|------------|
| 42 | Exp | YES | 25.5 | 21.6 | 5.1 | 21.7 | 40.6 | 676 |
| 42 | Ctrl | YES | 34.8 | 10.8 | 6.9 | 39.2 | 38.4 | 461 |
| 137 | Exp | YES | 29.3 | 19.9 | 8.4 | 33.7 | 36.7 | 400 |
| 137 | Ctrl | YES | 30.3 | 19.2 | 8.0 | 33.5 | 37.1 | 432 |
| 256 | Exp | YES | 28.8 | 27.8 | 0.4 | 0.0 | 14.4 | 972 |
| 256 | Ctrl | YES | 27.3 | 24.4 | 0.4 | 0.0 | 16.4 | 975 |
| 999 | Exp | YES | 25.3 | 25.3 | 1.5 | 0.0 | 18.0 | 891 |
| 999 | Ctrl | YES | 25.3 | 25.3 | 1.5 | 0.0 | 18.0 | 891 |
| 2026 | Exp | YES | 27.2 | 27.2 | 2.4 | 0.0 | 15.0 | 841 |
| 2026 | Ctrl | YES | 26.6 | 25.5 | 2.2 | 0.0 | 16.0 | 851 |

### Cross-Seed Averages (mean +/- std dev)

Survived: Exp 5/5, Ctrl 5/5

| Metric | Experimental | Control | Delta |
|--------|-------------|---------|-------|
| EAT ratio | 27.2% +/- 1.8% | 28.9% +/- 3.8% | -1.6% |
| REFRESH ratio | 24.4% +/- 3.5% | 21.0% +/- 6.3% | 3.3% |
| DIVIDE ratio | 3.6% +/- 3.2% | 3.8% +/- 3.4% | -0.2% |
| Low-E EAT rate | 11.1% +/- 15.8% | 14.5% +/- 20.0% | -3.4% |
| Avg population | 24.9 +/- 12.7 | 25.2 +/- 11.5 | -0.2 |
| Avg energy | 756.3 +/- 226.5 | 722.0 +/- 255.9 | 34.3 |

---

## Competition Experiment: Varying Food Pressure

Testing how food scarcity affects population dynamics with freshness_decay=true.

| Food/tick | Survived | Avg Pop | Avg Energy | EAT% | REFRESH% | DIVIDE% |
|-----------|----------|---------|-----------|------|----------|--------|
| 10 | YES | 4.0 | 998 | 29.2 | 29.2 | 0.0 |
| 20 | YES | 7.0 | 998 | 31.0 | 31.0 | 0.0 |
| 30 | YES | 11.0 | 998 | 31.4 | 30.1 | 0.0 |
| 40 | YES | 31.3 | 652 | 26.2 | 19.3 | 4.6 |
| 50 | YES | 40.6 | 676 | 25.5 | 21.6 | 5.1 |
| 75 | YES | 47.4 | 323 | 48.8 | 9.5 | 8.0 |
| 100 | YES | 52.7 | 439 | 46.4 | 14.6 | 8.6 |

---

## Stigmergy Experiment: Indirect Communication via Shared Medium

Testing whether organisms evolve to use EMIT/SAMPLE for indirect coordination.
Setup: 5 Seed A + 5 Seed B + 10 Seed C (stigmergy-capable), medium_size=256.

| Metric | With Medium | No Medium | Delta |
|--------|------------|-----------|-------|
| Survived | YES | YES | — |
| Avg population | 38.4 | 37.0 | 1.4 |
| Avg energy | 341 | 538 | -197 |
| EAT% | 25.7 | 32.4 | -6.6 |
| REFRESH% | 12.0 | 18.1 | -6.1 |
| DIVIDE% | 7.2 | 4.3 | 2.9 |

---

## Detailed Results (Seed 42)

# D0 Virtual Machine — Experiment Results

## Experiment Overview

**Hypothesis**: Freshness decay (operational closure constraint) drives evolution of conditional survival-priority behavior.

| Parameter | Value |
|-----------|-------|
| Population cap | 100 |
| Initial organisms | 10 Seed A + 10 Seed B |
| Food per tick | 50 |
| Mutation rate | 0.001 |
| Total ticks | 100,000 |
| Freshness max | 255 |
| Eat energy | 10 |
| Instruction cost | 1 |
| E_MAX | 1000 |

## Experimental Group (freshness_decay = true)

- **Final tick**: 100000
- **Final population**: 39
- **Average energy**: 604.97
- **Average code length**: 6.72
- **Average freshness**: 252.46
- **Max generation**: 620

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 5000 | 21 | 587.7 | 5.6 | 24.0789 | 23.4530 | 6.1644 |
| 10000 | 21 | 636.3 | 5.6 | 24.2670 | 23.6408 | 5.4037 |
| 15000 | 20 | 711.0 | 5.5 | 24.8405 | 24.9452 | 4.3121 |
| 20000 | 100 | 363.8 | 7.5 | 17.0005 | 15.9253 | 12.4305 |
| 25000 | 35 | 466.9 | 6.6 | 20.7606 | 20.3399 | 7.9148 |
| 30000 | 35 | 580.9 | 6.6 | 21.5670 | 21.5670 | 6.2014 |
| 35000 | 35 | 603.6 | 6.6 | 21.6628 | 21.6941 | 5.8177 |
| 40000 | 34 | 642.6 | 6.5 | 21.8927 | 22.1517 | 5.1043 |
| 45000 | 32 | 715.9 | 6.4 | 22.3681 | 22.8170 | 4.0434 |
| 50000 | 32 | 730.2 | 6.4 | 22.3813 | 23.2794 | 4.0447 |
| 55000 | 32 | 759.6 | 6.4 | 22.4024 | 23.2509 | 3.5965 |
| 60000 | 32 | 786.8 | 6.4 | 22.7157 | 23.6111 | 2.7891 |
| 65000 | 30 | 838.9 | 6.3 | 22.8432 | 23.8123 | 2.6062 |
| 70000 | 39 | 611.9 | 6.7 | 26.5603 | 20.2345 | 6.9226 |
| 75000 | 39 | 597.7 | 6.7 | 26.3009 | 21.4823 | 5.3049 |
| 80000 | 40 | 570.4 | 6.8 | 26.3558 | 21.4909 | 5.3273 |
| 85000 | 40 | 559.7 | 6.8 | 26.7565 | 21.1409 | 5.6182 |
| 90000 | 36 | 651.2 | 6.6 | 27.1151 | 21.1213 | 5.2816 |
| 95000 | 38 | 619.8 | 6.7 | 27.0924 | 21.1738 | 5.2667 |
| 100000 | 39 | 605.0 | 6.7 | 27.1000 | 21.1119 | 5.2968 |

## Control Group (freshness_decay = false)

- **Final tick**: 100000
- **Final population**: 40
- **Average energy**: 463.18
- **Average code length**: 6.75
- **Average freshness**: 255.00
- **Max generation**: 223

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 5000 | 21 | 587.7 | 5.6 | 24.0789 | 23.4530 | 6.1644 |
| 10000 | 21 | 636.3 | 5.6 | 24.2670 | 23.6408 | 5.4037 |
| 15000 | 20 | 711.0 | 5.5 | 24.8405 | 24.9452 | 4.3121 |
| 20000 | 100 | 972.9 | 7.5 | 27.8330 | 5.0060 | 12.2970 |
| 25000 | 40 | 366.5 | 6.8 | 28.2647 | 6.9909 | 11.1812 |
| 30000 | 38 | 385.1 | 6.7 | 34.4921 | 11.0115 | 7.7746 |
| 35000 | 38 | 385.5 | 6.7 | 34.4010 | 11.0238 | 7.7729 |
| 40000 | 35 | 417.8 | 6.6 | 35.5722 | 11.0124 | 7.7699 |
| 45000 | 37 | 396.1 | 6.6 | 34.3159 | 11.0241 | 7.7755 |
| 50000 | 38 | 452.1 | 6.7 | 34.6664 | 10.9594 | 7.0263 |
| 55000 | 41 | 429.3 | 6.8 | 34.7579 | 10.7608 | 6.9127 |
| 60000 | 38 | 461.7 | 6.7 | 34.7303 | 10.7075 | 6.9383 |
| 65000 | 37 | 475.9 | 6.6 | 34.7355 | 10.7612 | 6.9176 |
| 70000 | 37 | 476.3 | 6.6 | 34.8404 | 10.7513 | 6.9260 |
| 75000 | 37 | 475.5 | 6.6 | 34.7440 | 10.7818 | 6.9014 |
| 80000 | 38 | 462.2 | 6.7 | 34.6886 | 10.7993 | 6.8976 |
| 85000 | 37 | 475.7 | 6.6 | 36.0705 | 10.8345 | 6.8490 |
| 90000 | 37 | 474.2 | 6.6 | 34.7415 | 10.7496 | 6.9016 |
| 95000 | 37 | 478.2 | 6.6 | 34.6194 | 10.7696 | 6.8569 |
| 100000 | 40 | 463.2 | 6.8 | 34.7519 | 10.7104 | 6.6177 |


## Comparative Analysis

### Steady-State Averages (tick 50k-100k)

| Metric | Experimental | Control | Difference |
|--------|-------------|---------|------------|
| EAT ratio | 0.2549 | 0.3478 | -0.0929 |
| REFRESH ratio | 0.2163 | 0.1076 | 0.1087 |
| DIVIDE ratio | 0.0510 | 0.0688 | -0.0178 |
| Low-energy EAT rate | 0.2172 | 0.3920 | -0.1748 |
| Avg population | 40.6 | 38.4 | 2.2 |
| Avg energy | 676.3 | 460.8 | 215.5 |

---

*Generated by D0 VM v0.2.0 — Cognitive Life Science operational closure experiment*
