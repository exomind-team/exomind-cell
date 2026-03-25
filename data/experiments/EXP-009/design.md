# EXP-009: Real CPU Data-Driven (v3, CEM=50, seed=42)
- **Hypothesis**: Variable resource environment shifts survival strategy
- **Parameters**: food = base_food * (0.3 + 0.7*(1-cpu_usage)), 500k ticks
- **Data**: data/realcpu_results.md, data/cpu_log.csv
- **Result**: EAT +119%, REFRESH -96%, DIVIDE +58% vs constant food
