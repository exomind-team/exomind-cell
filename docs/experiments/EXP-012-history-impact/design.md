# EXP-012: History Impact (D2 Experience Learning)

## Hypothesis

Organisms with experience of abundance (historical context) will behave differently
in scarcity than organisms that never experienced abundance — demonstrating that
stored history (Data cell) influences current behavior (D2: experience learning).

## Prediction

- Group 1 (abundant→scarce): Different steady-state behavior than Group 2 (always scarce)
- Specifically: Group 1 retains higher EAT rate after switch (abundance memory)
- Data cell stores energy readings; LOAD+CMP conditions EAT on past experience

## Groups

| Group | Phase 1 (0-10k ticks) | Phase 2 (10k-500k ticks) |
|-------|----------------------|--------------------------|
| 1 | Abundant (500/tick) | Scarce (50/tick) |
| 2 | Scarce (50/tick) | Scarce (50/tick) |

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3, CEM=50, R=5 |
| total_ticks | 500,000 |
| seeds | 300-309 (10 seeds) |
| max_organisms | 200 |
| medium_size | 256 |
| switch_tick | 10,000 |
| Initial pop | 20 Seed F + 10 Seed A + 10 Seed B |

## Status

Complete (`cargo run --release -- --exp012`).
