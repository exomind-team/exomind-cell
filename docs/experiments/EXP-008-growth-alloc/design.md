# EXP-008: Growth (ALLOC) Exploration

## Hypothesis

Body growth (allocating new Energy cells via ALLOC instruction) enables organisms
to increase their energy storage capacity dynamically, providing adaptive advantage
when energy is abundant.

## Prediction

- With ALLOC (Seed E): Higher avg_cell_count than No ALLOC baseline
- With ALLOC: Population adapts cell count to energy availability
- Body growth trades immediate reproduction for increased storage capacity

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3 |
| CEM | 50, R=5 |
| seed | 42 |
| total_ticks | 500,000 |
| Condition A | With ALLOC seeds (Seed E) |
| Condition B | Without ALLOC (Seed A + B baseline) |
| freshness_decay | true (exp) vs false (ctrl) |

## Seed E Genome

`EAT → DIGEST → SENSE_SELF → CMP → JNZ → ALLOC → REFRESH → DIVIDE → JMP`
ALLOC fires when energy exceeds threshold — body grows by adding Energy cell.

## Status

Complete (`cargo run --release -- --cell`).
