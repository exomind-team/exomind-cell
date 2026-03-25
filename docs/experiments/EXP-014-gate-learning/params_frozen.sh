#!/bin/bash
# EXP-014: GATE Learning (History Impact) — FROZEN PARAMETERS
# Do NOT modify without Leader approval.

# VM Mode
VM_MODE="cell_v3"
DATA_CELL_GATING=true           # Enable GATE instruction

# Cell parameters
CEM=50
REFRESH_RADIUS=5
FRESHNESS_MAX=255
MUTATION_RATE=0.001

# Population
MAX_ORGANISMS=200
INIT_POP="20G+10A+10B"         # 20 Seed G (GATE-capable) + 10A + 10B

# Group 1: Abundant → Scarce
G1_FOOD_ABUNDANT=500            # First 10k ticks
G1_FOOD_SCARCE=50               # After 10k ticks
G1_SWITCH_TICK=10000

# Group 2: Always Scarce
G2_FOOD=50                      # Constant throughout

# Run parameters
TICKS=1000000                   # 1M ticks per run
ROUNDS=100                      # 100 independent rounds
SEEDS_PER_ROUND=10              # 10 seeds per round

# Reproduce
# cargo run --release -- --exp014
