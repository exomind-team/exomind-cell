#!/bin/bash
# EXP-008: Growth (ALLOC) — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --cell
echo "Results: CELL_RESULTS.md (Experiment 4 section)"
