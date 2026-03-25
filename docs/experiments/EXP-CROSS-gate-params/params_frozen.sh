#!/bin/bash
# EXP-CROSS: GATE × Parameter 2x2 Matrix — FROZEN
# Pre-approved: based on existing frozen baselines
export RAYON_NUM_THREADS=12

# Group 1: Optimized + No GATE (existing data from --stats)
# CEM=50, food=500, max=1000, 50A+50B, 2M ticks, GATE=false

# Group 2: Optimized + GATE (NEW — need to run)
# CEM=50, food=500, max=1000, 30A+30B+40G, 2M ticks, GATE=true
# History: abundant(500) first 10k ticks, then food=500 continues (no switch — test GATE in stable env)

# Group 3: Non-optimized + No GATE (existing data from cell experiments)
# CEM=50, food=50, max=200, 10A+10B, 500k ticks, GATE=false

# Group 4: Non-optimized + GATE (NEW — need to run)
# CEM=50, food=50, max=200, 10A+10B+20G, 500k ticks, GATE=true

# 100 seeds per group
SEEDS=100

# Reproduce: cargo run --release -- --exp-cross --threads 12
