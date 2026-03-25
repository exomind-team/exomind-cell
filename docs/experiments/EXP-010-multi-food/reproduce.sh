#!/bin/bash
# EXP-010: Multi-Food Type — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --cell
echo "Results: CELL_RESULTS.md (Experiment 5 section)"
