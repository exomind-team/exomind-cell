# D0 Virtual Machine — Experiment Results v3 (500k ticks)

## Changes from v2

- **500k ticks** (was 100k) — 5x longer runs for deeper evolutionary divergence
- **E_MAX impact analysis** — comparing E_MAX=1000 vs unlimited to isolate energy cap effects
- **Steady-state window**: tick 250k-500k (second half)

## Multi-Seed Summary (steady-state, tick 250k-500k)

### Per-Seed Results

| Seed | Group | Survived | EAT% | REFRESH% | DIVIDE% | Low-E EAT% | Avg Pop | Avg Energy |
|------|-------|----------|------|----------|---------|-----------|---------|------------|
| 42 | Exp | YES | 26.9 | 21.0 | 1.3 | 26.4 | 39.7 | 904 |
| 42 | Ctrl | YES | 39.8 | 11.2 | 3.9 | 53.6 | 40.3 | 690 |
| 137 | Exp | YES | 31.6 | 18.7 | 5.5 | 38.2 | 38.9 | 674 |
| 137 | Ctrl | YES | 31.3 | 18.9 | 8.0 | 33.6 | 37.5 | 450 |
| 256 | Exp | YES | 29.3 | 28.2 | 0.0 | 0.0 | 14.0 | 998 |
| 256 | Ctrl | YES | 27.7 | 24.7 | 0.0 | 0.0 | 16.0 | 998 |
| 999 | Exp | YES | 25.2 | 25.2 | 1.4 | 0.0 | 18.0 | 891 |
| 999 | Ctrl | YES | 25.2 | 25.2 | 1.4 | 0.0 | 18.0 | 891 |
| 2026 | Exp | YES | 29.3 | 29.3 | 0.0 | 0.0 | 13.0 | 998 |
| 2026 | Ctrl | YES | 28.4 | 27.2 | 0.0 | 0.0 | 14.0 | 998 |

### Cross-Seed Averages (mean +/- std dev)

Survived: Exp 5/5, Ctrl 5/5

| Metric | Experimental | Control | Delta |
|--------|-------------|---------|-------|
| EAT ratio | 28.4% +/- 2.5% | 30.5% +/- 5.7% | -2.0% |
| REFRESH ratio | 24.5% +/- 4.6% | 21.4% +/- 6.5% | 3.0% |
| DIVIDE ratio | 1.7% +/- 2.3% | 2.7% +/- 3.4% | -1.0% |
| Low-E EAT rate | 12.9% +/- 18.2% | 17.4% +/- 24.9% | -4.5% |
| Avg population | 24.7 +/- 13.4 | 25.2 +/- 12.7 | -0.4 |
| Avg energy | 892.7 +/- 132.4 | 805.3 +/- 234.9 | 87.5 |

---

## E_MAX Impact Analysis (seed=42, 500k ticks)

Comparing E_MAX=1000 (capped) vs E_MAX=unlimited to determine if the energy cap
is responsible for the low-energy EAT rate inversion seen in some v2 seeds.

| Metric | E_MAX=1000 Exp | E_MAX=1000 Ctrl | E_MAX=unlim Exp | E_MAX=unlim Ctrl |
|--------|---------------|----------------|----------------|------------------|
| EAT ratio | 26.9% | 39.8% | 36.1% | 41.9% |
| REFRESH ratio | 21.0% | 11.2% | 13.3% | 11.4% |
| DIVIDE ratio | 1.3% | 3.9% | 6.2% | 3.8% |
| Low-E EAT rate | 26.4% | 53.6% | 48.0% | 58.9% |
| Avg population | 40 | 40 | 23 | 22 |
| Avg energy | 904 | 690 | 271869 | 374877 |

---

## Detailed Results (Seed 42, E_MAX=1000)

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

- **Final tick**: 500000
- **Final population**: 37
- **Average energy**: 997.51
- **Average code length**: 6.65
- **Average freshness**: 253.32
- **Max generation**: 1047

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 25000 | 35 | 466.9 | 6.6 | 20.7606 | 20.3399 | 7.9148 |
| 50000 | 32 | 730.2 | 6.4 | 22.3813 | 23.2794 | 4.0447 |
| 75000 | 39 | 597.7 | 6.7 | 26.3009 | 21.4823 | 5.3049 |
| 100000 | 39 | 605.0 | 6.7 | 27.1000 | 21.1119 | 5.2968 |
| 125000 | 39 | 625.3 | 6.7 | 27.5805 | 20.8469 | 5.2743 |
| 150000 | 38 | 715.5 | 6.7 | 26.7003 | 20.9134 | 4.2887 |
| 175000 | 36 | 807.5 | 6.6 | 26.5348 | 21.0198 | 3.5679 |
| 200000 | 38 | 821.9 | 6.7 | 27.0276 | 21.0020 | 2.9938 |
| 225000 | 38 | 870.2 | 6.7 | 26.4380 | 21.0730 | 2.3195 |
| 250000 | 41 | 828.3 | 6.8 | 26.4710 | 20.9765 | 2.1029 |
| 275000 | 39 | 887.8 | 6.7 | 26.5737 | 21.0626 | 1.7427 |
| 300000 | 39 | 873.9 | 6.7 | 26.9446 | 21.0713 | 2.0758 |
| 325000 | 41 | 856.2 | 6.8 | 27.0856 | 21.0885 | 1.7809 |
| 350000 | 39 | 899.5 | 6.7 | 27.0555 | 21.0961 | 1.8035 |
| 375000 | 40 | 901.4 | 6.8 | 27.1945 | 21.0804 | 1.5130 |
| 400000 | 40 | 925.0 | 6.8 | 26.9116 | 21.1231 | 1.2180 |
| 425000 | 41 | 903.7 | 6.8 | 26.8987 | 21.1432 | 1.2090 |
| 450000 | 41 | 902.4 | 6.8 | 26.9141 | 19.8069 | 1.2535 |
| 475000 | 37 | 997.8 | 6.6 | 26.7757 | 21.7486 | 0.0000 |
| 500000 | 37 | 997.5 | 6.6 | 26.7703 | 21.7676 | 0.0000 |

## Control Group (freshness_decay = false)

- **Final tick**: 500000
- **Final population**: 41
- **Average energy**: 689.80
- **Average code length**: 6.78
- **Average freshness**: 255.00
- **Max generation**: 600

### Population Dynamics

| Tick | Population | Avg Energy | Avg Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----------|-----------|-------------|------|----------|--------|
| 0 | 20 | 100.0 | 5.5 | 0.0000 | 0.0000 | 0.0000 |
| 25000 | 40 | 366.5 | 6.8 | 28.2647 | 6.9909 | 11.1812 |
| 50000 | 38 | 452.1 | 6.7 | 34.6664 | 10.9594 | 7.0263 |
| 75000 | 37 | 475.5 | 6.6 | 34.7440 | 10.7818 | 6.9014 |
| 100000 | 40 | 463.2 | 6.8 | 34.7519 | 10.7104 | 6.6177 |
| 125000 | 39 | 526.0 | 6.7 | 34.7175 | 10.8648 | 6.1105 |
| 150000 | 40 | 537.3 | 6.8 | 35.3176 | 10.8507 | 5.7189 |
| 175000 | 41 | 618.8 | 6.8 | 35.4147 | 11.1785 | 4.7066 |
| 200000 | 39 | 701.4 | 6.7 | 35.7076 | 11.0598 | 4.1936 |
| 225000 | 41 | 667.0 | 6.8 | 35.9929 | 11.0640 | 4.1462 |
| 250000 | 40 | 684.0 | 6.8 | 39.8321 | 11.1835 | 4.0449 |
| 275000 | 40 | 682.7 | 6.8 | 39.7885 | 11.2061 | 4.0475 |
| 300000 | 42 | 649.6 | 6.8 | 39.7665 | 11.1901 | 4.0448 |
| 325000 | 39 | 701.5 | 6.7 | 39.8612 | 11.1542 | 4.0565 |
| 350000 | 39 | 701.1 | 6.7 | 39.8221 | 11.1759 | 4.0662 |
| 375000 | 41 | 667.4 | 6.8 | 39.8478 | 11.1542 | 4.0599 |
| 400000 | 41 | 689.7 | 6.8 | 39.8599 | 11.1393 | 3.7822 |
| 425000 | 41 | 689.7 | 6.8 | 39.8372 | 11.1593 | 3.7790 |
| 450000 | 40 | 706.1 | 6.8 | 39.8466 | 11.1451 | 3.7816 |
| 475000 | 41 | 689.8 | 6.8 | 39.8549 | 11.1393 | 3.7723 |
| 500000 | 41 | 689.8 | 6.8 | 39.8381 | 11.1638 | 3.7805 |


## Comparative Analysis

### Steady-State Averages (tick 50k-100k)

| Metric | Experimental | Control | Difference |
|--------|-------------|---------|------------|
| EAT ratio | 0.2688 | 0.3983 | -0.1295 |
| REFRESH ratio | 0.2103 | 0.1116 | 0.0987 |
| DIVIDE ratio | 0.0134 | 0.0392 | -0.0258 |
| Low-energy EAT rate | 0.2641 | 0.5364 | -0.2723 |
| Avg population | 39.7 | 40.3 | -0.6 |
| Avg energy | 903.8 | 689.9 | 213.9 |

---

*Generated by D0 VM v0.2.0 — Cognitive Life Science operational closure experiment*
