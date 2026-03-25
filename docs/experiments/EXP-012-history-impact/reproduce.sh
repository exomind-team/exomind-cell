#!/bin/bash
# EXP-012: History Impact — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
RAYON_NUM_THREADS=12 cargo run --release -- --exp012
echo "Results: docs/experiments/EXP-012-history-impact/experiment.md"
