#!/bin/bash
# Reproduce EXP-010: Multi-Food Type Experiment
# Run from repo root: bash data/experiments/EXP-010/reproduce.sh
cargo run --release -- --cell
# Results appear in CELL_RESULTS.md (Experiment 5 section)
# Raw CSVs: data/cell_multifood_exp.csv, data/cell_multifood_ctrl.csv, data/cell_simplefood_exp.csv
