#!/bin/bash
# EXP-001~003: Operational Closure — Reproduce
# Frozen parameters: see params_frozen.sh

set -e
cd "$(dirname "$0")/../.."

# Build release binary
cargo build --release

# Run large-scale statistical analysis (100 seeds × 2M ticks, rayon parallel)
# Results → data/large_scale/statistical_analysis.md
RAYON_NUM_THREADS=12 cargo run --release -- --stats

# Run 100-round independent replication (5 seeds/round × 500k ticks)
# Results → docs/experiments/EXP-001-replication/replication_100rounds.md
RAYON_NUM_THREADS=12 cargo run --release -- --replicate001

echo ""
echo "Results:"
echo "  data/large_scale/statistical_analysis.md"
echo "  docs/experiments/EXP-001-replication/replication_100rounds.md"
echo "  docs/experiments/EXP-001-replication/replication_100rounds_v2_largescale.md"
