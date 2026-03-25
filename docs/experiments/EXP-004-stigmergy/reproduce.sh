#!/bin/bash
# EXP-004: Stigmergy — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
# v2 stigmergy mode (includes Seed C organisms)
cargo run --release -- --run-v2
# Note: stigmergy experiment is embedded in v2 run
echo "Results: RESULTS.md"
