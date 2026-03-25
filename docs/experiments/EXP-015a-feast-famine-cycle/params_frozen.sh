#!/bin/bash
# EXP-015a: Feast/Famine Cycling + GATE — FROZEN PARAMETERS
# Approved by Leader. Do NOT modify without approval.
export RAYON_NUM_THREADS=12

# VM
VM_MODE="cell_v3"
DATA_CELL_GATING=true
CEM=50
REFRESH_RADIUS=5
FRESHNESS_MAX=255
MUTATION_RATE=0.001

# Population
MAX_ORGANISMS=200
INIT_POP="20G+10A+10B"  # Seed G (GATE-capable) + baseline seeds

# Food cycling (key difference from EXP-014)
FOOD_HIGH=500            # feast phase
FOOD_LOW=50              # famine phase
CYCLE_PERIOD=20000       # ticks per full cycle (10k feast + 10k famine)
# Pattern: 0-10k feast, 10k-20k famine, 20k-30k feast, ...

# Control: constant food at average level
CTRL_FOOD=275            # (500+50)/2 = 275

# Run
TICKS=1000000
ROUNDS=100
SEEDS_PER_ROUND=10       # 10 seeds per round (matched to EXP-014)

# Reproduce
# cargo run --release -- --exp015a --threads 12
