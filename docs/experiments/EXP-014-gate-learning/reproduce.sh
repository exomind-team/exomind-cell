#!/bin/bash
# EXP-014: GATE Learning — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
RAYON_NUM_THREADS=12 cargo run --release -- --exp014
echo "Results: docs/experiments/EXP-014-gate-learning/experiment.md"
echo "CSV: docs/experiments/EXP-014-gate-learning/data/per_round.csv"
