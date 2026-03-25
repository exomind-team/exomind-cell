#!/bin/bash
# EXP-009: Real CPU — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release

# 100-round replication (hash-based, reproducible)
RAYON_NUM_THREADS=12 cargo run --release -- --replicate009

echo "Results: docs/experiments/EXP-009-replication/replication_100rounds.md"
echo "CSV: docs/experiments/EXP-009-replication/data/rounds_data.csv"
