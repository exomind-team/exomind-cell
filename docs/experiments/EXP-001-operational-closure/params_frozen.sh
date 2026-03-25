#!/bin/bash
# EXP-001~003: Operational Closure — FROZEN PARAMETERS
export RAYON_NUM_THREADS=12  # Max 12 threads (of 32), leave headroom for system
# Do NOT modify without Leader approval.

# VM Mode
VM_MODE="cell_v3"              # Cell-based VM with per-cell freshness

# Cell parameters
CEM=50                          # cell_energy_max
REFRESH_RADIUS=5                # REFRESH covers ip-5 to ip+5
FRESHNESS_MAX=255               # freshness countdown max
MUTATION_RATE=0.001             # per-instruction mutation probability on DIVIDE

# Population
MAX_ORGANISMS=1000              # population cap
INIT_POP="50A+50B"             # 50 Seed A + 50 Seed B
FOOD_PER_TICK=500               # food injected per tick

# Run parameters
TICKS=2000000                   # 2M ticks per run
SEEDS=100                       # seeds 1000-1099
SNAPSHOT_INTERVAL=10000         # stats every 10k ticks

# Groups
# EXP-001: FRESHNESS_DECAY=true (exp) vs false (ctrl), single seed (42)
# EXP-002: same as EXP-001 but 5 seeds [42,137,256,999,2026]
# EXP-003: E_MAX comparison (E_MAX=1000 vs unlimited), seed=42
# Large-scale: 100 seeds, both groups

# Reproduce (uses existing --stats mode which has these exact params)
# cargo run --release -- --stats
