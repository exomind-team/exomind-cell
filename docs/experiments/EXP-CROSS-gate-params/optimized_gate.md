# Optimized Parameters + GATE History Effect

**Fills the missing cell in the 2x2 matrix: Optimized + GATE + History switching**

## Parameters (frozen)

- CEM=50, max_organisms=1000, food_per_tick=500→50
- Initial: 20 Seed G + 30 Seed A + 30 Seed B
- GATE=true, data_cell_gating=true
- Total: 2M ticks, 100 seeds (9000-9099)
- Switch point: food=500 first 500k ticks → food=50 remaining 1.5M ticks
- Threads: 8 (coexisting with EXP-015a)

## Groups

| Group | Food schedule | Purpose |
|-------|--------------|---------|
| Exp (abundant→scarce) | 500 for 500k → 50 for 1.5M | History: organisms experienced abundance |
| Ctrl (always scarce) | 50 throughout | Baseline: no abundance history |

## Results

| Metric | Exp (abundant→scarce) | Ctrl (always scarce) |
|--------|----------------------|---------------------|
| Survived | 100/100 | 92/100 |
| Avg Population | 18.6 | 31.8 |
| Avg Energy | 56.6 | 26.7 |
| REFRESH% | **12.7%** | **3.7%** |

## Statistical Tests

| Test | Value |
|------|-------|
| REFRESH direction win | **82/100 (82%)** |
| Mann-Whitney p (REFRESH) | **p < 0.0001** |
| Cohen's d (REFRESH) | **1.117** (large effect) |
| Population direction (Exp < Ctrl) | 86/100 |
| Mann-Whitney p (Population) | **p < 0.0001** |
| Cohen's d (Population) | **-1.227** (large effect) |

## Updated 2x2 Matrix

| | No GATE | GATE |
|---|---------|------|
| **Optimized** (food=500, max=1000, 2M) | **75%** direction win (EXP-014) | **82%** direction win, d=1.117 **(this experiment)** |
| **Non-optimized** (food=50, max=200, 500k) | **42%** direction win | **44%** direction win, d=0.086 (EXP-CROSS G4) |

## Interpretation

1. **GATE amplifies history effect under optimized parameters**: 82% direction win vs 75% (no GATE), both with p<0.001
2. **GATE has no effect under non-optimized parameters**: 44% vs 42%, both statistically insignificant
3. **Parameter scale is the prerequisite**: GATE only matters when organisms have sufficient resources (food=500, max=1000) to develop complex behaviors during the abundance phase
4. **Trade-off pattern**: Exp group has lower population (18.6 vs 31.8) but higher energy (56.6 vs 26.7) and much higher REFRESH (12.7% vs 3.7%) — history teaches REFRESH-priority at the cost of reproduction
5. **Survival asymmetry**: 100% vs 92% — abundance history confers survival advantage even after resource crash

## Data

- Per-seed CSV: `../EXP-OPT-GATE/data/per_seed.csv`
- Full report: `../EXP-OPT-GATE/experiment.md`

---
*Generated 2026-03-25. Optimized GATE history experiment, 100 seeds × 2M ticks.*
