#!/bin/bash
# EXP-006: REFRESH Radius Gradient — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --cell
echo "Results: CELL_RESULTS.md (Experiment 2 section)"
