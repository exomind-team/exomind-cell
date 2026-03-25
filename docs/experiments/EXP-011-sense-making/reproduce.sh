#!/bin/bash
# EXP-011: Sense-Making — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release

# Original 30-seed run (6 groups × 30 seeds)
RAYON_NUM_THREADS=12 cargo run --release -- --exp011

# 100-round independent replication
RAYON_NUM_THREADS=12 cargo run --release -- --replicate011

echo "Results:"
echo "  data/experiments/EXP-011/results.md (30-seed original)"
echo "  docs/experiments/EXP-011-replication/replication_100rounds.md (100-round)"
