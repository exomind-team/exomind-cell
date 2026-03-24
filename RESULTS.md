# D0 Virtual Machine — Multi-Seed Experiment Results (v2)

## Changes from v1

- **E_MAX = 1000**: Energy cap prevents unbounded accumulation
- **5 random seeds**: Statistical validation across seeds 42, 137, 256, 999, 2026
- **Genome dumps**: Longest-lived organism's code saved every 10k ticks

## Multi-Seed Summary (steady-state averages, tick 50k-100k)

### Per-Seed Results

| Seed | Group | Survived | EAT% | REFRESH% | DIVIDE% | Low-E EAT% | Avg Pop | Avg Energy |
|------|-------|----------|------|----------|---------|-----------|---------|------------|
| 42 | Exp | YES | 28.3 | 20.4 | 7.2 | 32.7 | 38.0 | 495 |
| 42 | Ctrl | YES | 28.4 | 11.3 | 5.4 | 34.5 | 41.3 | 638 |
| 137 | Exp | YES | 20.1 | 20.1 | 7.5 | 18.1 | 40.0 | 487 |
| 137 | Ctrl | YES | 31.8 | 9.3 | 3.1 | 32.1 | 43.9 | 788 |
| 256 | Exp | YES | 26.3 | 17.9 | 8.8 | 22.1 | 60.3 | 629 |
| 256 | Ctrl | YES | 31.7 | 9.4 | 8.8 | 20.2 | 66.2 | 729 |
| 999 | Exp | YES | 29.2 | 20.0 | 9.3 | 34.1 | 36.1 | 353 |
| 999 | Ctrl | YES | 29.2 | 20.0 | 9.3 | 34.1 | 36.1 | 353 |
| 2026 | Exp | YES | 29.9 | 20.2 | 9.4 | 28.3 | 45.9 | 524 |
| 2026 | Ctrl | YES | 29.2 | 19.6 | 9.0 | 28.2 | 45.4 | 545 |

### Cross-Seed Averages (mean +/- std dev)

Survived: Exp 5/5, Ctrl 5/5

| Metric | Experimental | Control | Delta |
|--------|-------------|---------|-------|
| EAT ratio | 26.8% +/- 3.9% | 30.1% +/- 1.6% | -3.3% |
| REFRESH ratio | 19.7% +/- 1.0% | 13.9% +/- 5.4% | 5.8% |
| DIVIDE ratio | 8.4% +/- 1.0% | 7.1% +/- 2.8% | 1.3% |
| Low-E EAT rate | 27.1% +/- 6.9% | 29.8% +/- 6.0% | -2.8% |
| Avg population | 44.1 +/- 9.8 | 46.6 +/- 11.5 | -2.5 |
| Avg energy | 497.6 +/- 98.8 | 610.6 +/- 171.0 | -112.9 |

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

## Experimental Group (freshness_decay = true)

- **Final tick**: 100000
- **Final population**: 42
- **Average energy**: 533.07
- **Average code length**: 6.81
- **Average freshness**: 252.29
- **Max generation**: 394

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 5000 | 21 | 578.7 | 5.6 | 23.5665 | 24.4107 | 5.8667 |
| 10000 | 36 | 419.3 | 6.6 | 20.0492 | 20.3561 | 8.7335 |
| 15000 | 33 | 528.1 | 6.5 | 21.0209 | 21.4462 | 6.9627 |
| 20000 | 29 | 637.1 | 6.3 | 21.9481 | 21.9824 | 5.4690 |
| 25000 | 28 | 657.8 | 6.2 | 22.2863 | 22.7349 | 5.1301 |
| 30000 | 28 | 691.9 | 6.2 | 22.3073 | 22.3180 | 4.6039 |
| 35000 | 40 | 431.8 | 6.8 | 27.7052 | 18.8092 | 9.0858 |
| 40000 | 35 | 486.7 | 6.6 | 27.9932 | 20.2534 | 7.7266 |
| 45000 | 38 | 443.9 | 6.7 | 27.9200 | 20.2875 | 7.7307 |
| 50000 | 35 | 475.1 | 6.6 | 28.0612 | 20.3260 | 7.8204 |
| 55000 | 37 | 450.2 | 6.6 | 28.3486 | 20.3018 | 8.0522 |
| 60000 | 38 | 437.2 | 6.7 | 28.3366 | 20.3538 | 8.0496 |
| 65000 | 42 | 415.6 | 6.8 | 28.0134 | 20.4227 | 7.6843 |
| 70000 | 37 | 501.4 | 6.6 | 28.1000 | 20.5061 | 7.2743 |
| 75000 | 37 | 502.6 | 6.6 | 28.1687 | 20.4217 | 7.3226 |
| 80000 | 39 | 498.6 | 6.7 | 28.2447 | 20.3950 | 7.0750 |
| 85000 | 36 | 544.8 | 6.6 | 28.2746 | 20.4040 | 7.0231 |
| 90000 | 37 | 542.6 | 6.6 | 28.4035 | 20.4634 | 6.7962 |
| 95000 | 37 | 578.9 | 6.6 | 28.3232 | 20.5051 | 6.3092 |
| 100000 | 42 | 533.1 | 6.8 | 28.3307 | 20.3891 | 6.0167 |
| 100000 | 42 | 533.1 | 6.8 | 28.3307 | 20.3891 | 6.0167 |

## Control Group (freshness_decay = false)

- **Final tick**: 100000
- **Final population**: 40
- **Average energy**: 699.62
- **Average code length**: 6.75
- **Average freshness**: 255.00
- **Max generation**: 412

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 5000 | 21 | 578.7 | 5.6 | 23.5665 | 24.4107 | 5.8667 |
| 10000 | 37 | 394.6 | 6.6 | 20.2642 | 15.4529 | 9.7329 |
| 15000 | 39 | 455.5 | 6.7 | 20.9052 | 14.5258 | 8.9016 |
| 20000 | 39 | 505.9 | 6.7 | 20.8892 | 14.1691 | 8.1302 |
| 25000 | 39 | 506.5 | 6.7 | 20.8634 | 14.0761 | 8.1647 |
| 30000 | 38 | 568.3 | 6.7 | 28.2337 | 11.6246 | 7.4512 |
| 35000 | 40 | 557.6 | 6.8 | 27.5850 | 16.4552 | 6.4108 |
| 40000 | 38 | 616.4 | 6.7 | 27.6077 | 13.2181 | 5.9838 |
| 45000 | 40 | 612.0 | 6.8 | 28.0398 | 11.4315 | 6.0196 |
| 50000 | 41 | 596.7 | 6.8 | 28.0099 | 11.4238 | 6.0368 |
| 55000 | 41 | 620.4 | 6.8 | 28.0266 | 11.4368 | 5.6730 |
| 60000 | 42 | 628.4 | 6.8 | 28.4243 | 11.3299 | 5.4367 |
| 65000 | 40 | 658.6 | 6.8 | 28.3844 | 11.3223 | 5.4224 |
| 70000 | 41 | 642.9 | 6.8 | 28.4797 | 11.2974 | 5.4803 |
| 75000 | 41 | 644.0 | 6.8 | 28.3918 | 11.3489 | 5.3850 |
| 80000 | 41 | 643.8 | 6.8 | 28.3882 | 11.3228 | 5.4251 |
| 85000 | 43 | 613.9 | 6.8 | 28.3651 | 11.3213 | 5.4069 |
| 90000 | 42 | 628.1 | 6.8 | 28.3922 | 11.3219 | 5.4062 |
| 95000 | 42 | 652.2 | 6.8 | 28.7251 | 11.2638 | 5.1147 |
| 100000 | 40 | 699.6 | 6.8 | 28.7247 | 11.3985 | 4.6933 |
| 100000 | 40 | 699.6 | 6.8 | 28.7247 | 11.3985 | 4.6933 |

## Comparative Analysis

### Steady-State Averages (tick 50k-100k)

| Metric | Experimental | Control | Difference |
|--------|-------------|---------|------------|
| EAT ratio | 0.2825 | 0.2840 | -0.0015 |
| REFRESH ratio | 0.2041 | 0.1134 | 0.0907 |
| DIVIDE ratio | 0.0725 | 0.0542 | 0.0183 |
| Low-energy EAT rate | 0.3269 | 0.3453 | -0.0184 |
| Low-freshness REFRESH rate | 0.0000 | N/A | — |
| Avg population | 38.0 | 41.3 | -3.3 |
| Max generation | 356.7 | 338.3 | 18.4 |

### Key Experimental Questions

**Q1: Did the experimental group evolve "low energy → prioritize EAT/REFRESH over DIVIDE" behavior?**

YES — Both EAT and REFRESH remain significant in the instruction mix, indicating organisms evolved to maintain both energy and freshness.

**Q2: Does the control group lack this conditional behavior?**

NO — Control group still shows REFRESH usage. This may indicate REFRESH is being retained for other reasons (e.g., genetic drift).

**Q3: Population dynamics differences?**

- Experimental group survived to end: **YES**
- Control group survived to end: **YES**

**Q4: Code evolution direction differences?**

- Experimental: code length 5.5 → 6.8
- Control: code length 5.5 → 6.8

---

## Methodology Notes

- Both groups use the same random seed for reproducibility
- Seed A = minimal self-sustaining (EAT + REFRESH loop)
- Seed B = self-sustaining + conditional DIVIDE
- Mutation: per-instruction replacement with probability 0.001 during DIVIDE
- Statistics sampled every 1000 ticks
- CSV data files available for detailed analysis

## Raw Data Files

- `experimental_group.csv` — Experimental group snapshots
- `control_group.csv` — Control group snapshots

---

*Generated by D0 VM v0.1.0 — Cognitive Life Science operational closure experiment*
