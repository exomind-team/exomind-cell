# D0 VM Experiment Registry

## Experiment Numbering

| ID | Name | VM | Ticks | Seeds | Key Parameter | Status |
|----|------|----|-------|-------|---------------|--------|
| EXP-001 | Operational closure vs control | v2 | 100k | 42 | freshness_decay on/off | Complete |
| EXP-002 | Multi-seed validation (E_MAX=1000) | v2 | 500k | 42,137,256,999,2026 | E_MAX=1000 | Complete |
| EXP-003 | E_MAX impact analysis | v2 | 500k | 42 | E_MAX=1000 vs unlimited | Complete |
| EXP-004 | Stigmergy communication | v2 | 100k | 42 | medium_size=256 vs 0 | Complete |
| EXP-005 | Cell v3 multi-seed (CEM=50) | v3 | 500k | 42,137,256,999,2026 | CEM=50, R=5 | Complete |
| EXP-006 | REFRESH radius gradient | v3 | 500k | 42 | R=1,2,3,5,8; CEM=50 | Complete |
| EXP-007 | Data cell exploration | v3 | 500k | 42 | with/without Data cell | Complete |

---

## EXP-001: Operational Closure vs Control

- **Hypothesis**: Freshness decay drives evolution of conditional survival-priority behavior
- **Parameters**: food_per_tick=50, E_MAX=1000, 100k ticks, seed=42
- **Prediction**: Exp group shows higher REFRESH ratio than control
- **Data**: `data/experimental_group.csv`, `data/control_group.csv`
- **Reproduce**: `cargo run --release`
- **Result**: REFRESH retained by selection in exp (23%), declining by drift in ctrl (19%)

## EXP-002: Multi-Seed Validation

- **Hypothesis**: EXP-001 results hold across 5 random seeds
- **Parameters**: 500k ticks, seeds=[42,137,256,999,2026], E_MAX=1000
- **Data**: `data/v3_exp_*.csv`, `data/v3_ctrl_*.csv`
- **Reproduce**: `cargo run --release`
- **Result**: REFRESH Exp 24.5%+/-4.6% vs Ctrl 21.4%+/-6.5%. Confirmed across 5 seeds

## EXP-003: E_MAX Impact Analysis

- **Hypothesis**: Energy cap amplifies exp/ctrl behavioral difference
- **Parameters**: seed=42, 500k ticks, E_MAX=1000 vs unlimited
- **Data**: `data/v3_emax_unlimited_*.csv`
- **Reproduce**: `cargo run --release`
- **Result**: With cap REFRESH delta=+9.8%, without cap delta=+1.9%

## EXP-004: Stigmergy Communication

- **Hypothesis**: Shared medium enables indirect coordination
- **Parameters**: seed=42, 100k ticks, medium_size=256 vs 0
- **Data**: `data/stigmergy_*.csv`
- **Result**: With medium DIVIDE +2.9%, REFRESH -6.1%. Signal-triggered DIVIDE works

## EXP-005: Cell v3 Multi-Seed (CEM=50)

- **Hypothesis**: Per-cell freshness produces measurably different behavior
- **Parameters**: CEM=50, R=5, 500k ticks, 5 seeds
- **Data**: `data/cell50_exp_*.csv`, `data/cell50_ctrl_*.csv`
- **Reproduce**: `cargo run --release -- --cell`
- **Result**: Exp REFRESH variance (1-24%) >> Ctrl (~14%). Strategy diversification confirmed

## EXP-006: REFRESH Radius Gradient

- **Hypothesis**: R controls strictness of operational closure constraint
- **Parameters**: CEM=50, seed=42, 500k ticks, R=1/2/3/5/8
- **Data**: `data/cellR*_exp.csv`, `data/cellR*_ctrl.csv`
- **Reproduce**: `cargo run --release -- --cell`
- **Result**: R=1,2: REFRESH=0% (abandoned). R=3: max divergence. R=8: REFRESH=18%, DIVIDE=0%

## EXP-007: Data Cell Exploration

- **Hypothesis**: Data cells enable experience-based decision-making
- **Parameters**: CEM=50, R=5, seed=42, 500k ticks, with/without Seed D
- **Data**: `data/cell_data_*.csv`, `data/cell_nodata_*.csv`
- **Reproduce**: `cargo run --release -- --cell`
- **Result**: With Data: energy 119 vs 79, DIVIDE 10.3% vs 8.9%. Preliminary

---

## CSV Columns

### v2: `tick,population,avg_energy,avg_code_length,avg_age,avg_freshness,total_eat,total_refresh,total_divide,total_instructions,eat_ratio,refresh_ratio,divide_ratio,low_energy_eat_rate,low_freshness_refresh_rate,max_generation`

### v3: `tick,population,avg_energy,avg_cell_count,avg_code_count,avg_freshness,max_generation,eat_ratio,digest_ratio,refresh_ratio,divide_ratio,total_instructions`
