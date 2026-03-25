#!/bin/bash
# EXP-007: Data Cell — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --cell
echo "Results: CELL_RESULTS.md (Experiment 3 section)"
