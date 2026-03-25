#!/bin/bash
# EXP-016: Dual-Pressure Value Gradient — FROZEN PARAMETERS
# Approved by Leader. Do NOT modify without approval.
export RAYON_NUM_THREADS=12

# VM
VM_MODE="cell_v3"
CEM=50
FRESHNESS_MAX=255
MUTATION_RATE=0.001

# Population
MAX_ORGANISMS=200
INIT_POP="20A+20B"

# Phase cycling (Groups 1 and 2)
PHASE_A_FOOD=20          # energy stress
PHASE_A_REFRESH_RADIUS=8 # easy maintenance
PHASE_B_FOOD=200         # energy abundant
PHASE_B_REFRESH_RADIUS=2 # hard maintenance
CYCLE_PERIOD=50000        # 25k per phase

# Group 3 (constant, no cycling)
CONSTANT_FOOD=110
CONSTANT_REFRESH_RADIUS=5

# Groups
# G1: freshness_decay=true  + phase cycling
# G2: freshness_decay=false + phase cycling
# G3: freshness_decay=true  + constant (no cycling)

# Run
TICKS=500000
SEEDS=30

# Reproduce
# cargo run --release -- --exp016 --threads 12
