#!/bin/bash
# Reproduce EXP-011: Sense-Making Signal Prediction
# 6 groups x 30 seeds = 180 runs (rayon parallel)
cargo run --release -- --exp011
