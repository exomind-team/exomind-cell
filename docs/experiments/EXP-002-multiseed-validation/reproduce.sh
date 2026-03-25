#!/bin/bash
# EXP-002: Multi-Seed Validation — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
# Runs 5-seed validation as part of v2 experiments
cargo run --release -- --run-v2
echo "Results: RESULTS.md"
