# EXP-007: Data Cell Exploration

## Hypothesis

Data cells (writable persistent storage within the organism body) enable experience-based
decision-making, allowing organisms to store energy readings and condition behavior on history.

## Prediction

- With Data cell (Seed D): Higher avg energy, different DIVIDE timing
- Seed D can write energy readings to data cell and condition EAT on stored threshold
- Data cell provides minimal memory: 1 byte per cell

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 |
| CEM | 50, R=5 |
| seed | 42 |
| total_ticks | 500,000 |
| Condition A | 5 Seed A + 5 Seed B + 10 Seed D (with Data cell) |
| Condition B | 10 Seed A + 10 Seed B (without Data cell) |
| freshness_decay | true (exp) vs false (ctrl) |

## Seed D Genome

`EAT → DIGEST → SENSE_SELF → STORE → REFRESH → DIVIDE → JMP`
The STORE writes energy reading to Data cell; LOAD can conditionally branch.

## Status

Complete (`cargo run --release -- --cell`).
