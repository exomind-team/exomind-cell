# EXP-011: Sense-Making (Signal Prediction)

## Hypothesis

Organisms that can SAMPLE environmental signals and condition behavior on them will
achieve higher EAT rates when signals predict food availability (delta>0 delay),
demonstrating signal-sensitive resource acquisition (D1+ sense-making).

## Prediction

- Group A (square wave, delta=200): Higher EAT than Group D (synchronous, delta=0)
- Group A: Higher EAT than Group E (random signal, unpredictable)
- Group A: Higher EAT than Group F (no signal)
- Advantage comes from predictive window: sample signal 200 ticks before food change

## Groups

| Group | Signal | Delta | Description |
|-------|--------|-------|-------------|
| A | Square wave (period=2000) | 200 | Predictable, 200-tick lead |
| B | Sine wave (period=2000) | 200 | Predictable smooth |
| C | Real CPU | 200 | External signal |
| D | Square wave (period=2000) | 0 | Synchronous (no prediction benefit) |
| E | Random | 200 | Unpredictable |
| F | None (constant) | 0 | No signal baseline |

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3, CEM=50, R=5 |
| total_ticks | 500,000 |
| seeds | 200-229 (30 per group, original) / round-dependent (replication) |
| max_organisms | 200 |
| medium_size | 256 |
| base_food | 300 units/10 ticks |
| Signal organisms | Seed F (SAMPLE → STORE → EAT → DIGEST → LOAD → CMP) |

## Status

Complete (30 seeds original). 100-round replication: A vs E 44%, A vs F 30% — not replicated.
