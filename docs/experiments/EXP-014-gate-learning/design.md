# EXP-014: GATE Learning (Data Cell as Gene Regulation Switch)

## Hypothesis

The GATE instruction (executes next instruction only if Data cell > threshold) acts
as a genetically regulated switch: organisms that experienced abundance will have
higher Data cell values → GATE opens → conditional DIVIDE fires.

## Prediction

- Group 1 (abundant→scarce): Higher REFRESH in steady-state after switch
- Difference persists because GATE is a more stable circuit than LOAD+CMP
- 100-round replication: >70% direction win for REFRESH effect

## Groups

| Group | Phase 1 (0-10k) | Phase 2 (10k-1M) |
|-------|----------------|------------------|
| 1 | Abundant (500/tick) | Scarce (50/tick) |
| 2 | Scarce (50/tick) | Scarce (50/tick) |

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3, CEM=50 |
| data_cell_gating | true (GATE instruction enabled) |
| total_ticks | 1,000,000 (1M) |
| rounds | 100 |
| seeds_per_round | 10 |
| Initial pop | 20 Seed G + 10 Seed A + 10 Seed B |

## Seed G Genome

`EAT → DIGEST → SENSE_SELF → STORE → GATE → DIVIDE → REFRESH → JMP`
GATE fires DIVIDE only when stored energy exceeds threshold.

## Status

Complete. 100 rounds: REFRESH direction win 92/100 (92%) — REPLICATED.
