#!/bin/bash
# EXP-009: Real CPU Food Modulation — FROZEN PARAMETERS
# Do NOT modify without Leader approval.

# VM Mode
VM_MODE="cell_v3"

# Cell parameters
CEM=50
REFRESH_RADIUS=5
FRESHNESS_MAX=255
MUTATION_RATE=0.001

# Population
MAX_ORGANISMS=200
INIT_POP="20A+20B"
FOOD_PER_TICK=50                # Baseline food (always present)

# CPU modulation
BASE_FOOD_INJECTION=500         # Extra food injected every CPU_SAMPLE_INTERVAL ticks
CPU_SAMPLE_INTERVAL=100         # Sample CPU every 100 ticks
CPU_FLOOR=0.3                   # Minimum 30% of base always injected
# Formula: food += base_food_injection * (0.3 + 0.7 * (1.0 - cpu_usage))
# Effective avg food: ~50/tick baseline + ~2.35/tick injection = ~52/tick total

# Control group
CTRL_FOOD_PER_TICK=55           # Constant food matching avg of CPU group (50 + 5)

# Run parameters
TICKS=500000
ROUNDS=100                      # 100 independent rounds
SEEDS_PER_ROUND=1               # 1 seed per round (each round = 1 complete experiment)

# Reproduce
# cargo run --release -- --real-cpu
# (needs modification to support 100 rounds — current code runs 1 round)
