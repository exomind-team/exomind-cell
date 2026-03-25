# EXP-009: Real CPU Interface

## Hypothesis

Organisms whose food supply is modulated by actual CPU availability (external environment
signal) will adapt their EAT timing compared to organisms with constant food supply,
demonstrating the system's ability to interface with real-world environmental data.

## Prediction

- CPU-modulated group: EAT rate adapts to CPU availability fluctuations
- Constant-food group: EAT rate stable, no temporal adaptation
- CPU-variable food tests whether organisms can evolve burst-feeding strategies

## Parameters

| Parameter | Value |
|-----------|-------|
| VM | Cell v3, CEM=50 |
| total_ticks | 500,000 |
| Base food | 50/tick (baseline) + CPU-variable bonus |
| CPU signal | Real sysinfo CPU usage (original) / hash-based pseudo (replication) |
| CPU floor | 30% (food = base × (0.3 + 0.7×(1-cpu))) |
| Constant food | 55/tick (matching average CPU-variable) |
| seed | 42 (original) / round-dependent (replication) |
| max_organisms | 200 |

## Status

Complete. 100-round replication: EAT diff 43/100 — not replicated at this scale.
