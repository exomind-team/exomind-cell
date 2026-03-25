# EXP-006: REFRESH Radius Gradient (v3, CEM=50, seed=42)
- **Hypothesis**: R controls strictness of operational closure
- **Parameters**: R=1,2,3,5,8; CEM=50, seed=42, 500k ticks
- **Data**: data/cellR*_exp.csv, data/cellR*_ctrl.csv
- **Result**: R=1,2: REFRESH=0%. R=3: max divergence. R=8: REFRESH=18%
