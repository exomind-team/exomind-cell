#!/bin/bash
# EXP-005: Cell v3 Multi-Seed — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --cell
echo "Results: CELL_RESULTS.md (Experiment 1 section)"
