# EXP-004: Stigmergy Communication

## Hypothesis

A shared signaling medium (stigmergy) enables indirect coordination among organisms,
leading to behavioral specialization (division of labor) — specifically increased DIVIDE.

## Prediction

- With medium (medium_size=256): Higher DIVIDE% than without medium
- With medium: Organisms using EMIT/SAMPLE can time reproduction signals
- REFRESH may decrease (medium maintenance offloads coordination cost)

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | v2 with stigmergy extension |
| seed | 42 |
| total_ticks | 100,000 |
| medium_size | 256 (exp) vs 0 (ctrl) |
| Initial pop | 5 Seed A + 5 Seed B + 10 Seed C |
| emit_cost | 1 |
| sample_cost | 1 |

## Status

Complete (run as part of `--run-v2` mode with `--stigmergy` flag).
