#!/bin/bash
# EXP-003: E_MAX Impact — Reproduce
cd "$(dirname "$0")/../.."
cargo build --release
cargo run --release -- --run-v2
echo "Results: RESULTS.md (E_MAX Impact Analysis section)"
