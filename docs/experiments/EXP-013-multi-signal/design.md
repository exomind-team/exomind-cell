# EXP-013: Multi-Signal Environment (D2 Experience Learning Advanced)

## Status: PLANNED (not yet implemented)

## Hypothesis

Organisms in an environment with multiple simultaneous signals (different channels,
different predictive delays, different correlations) will develop selective attention —
reading only the signal most correlated with their food source, ignoring others.

## Prediction

- Organisms evolve to SAMPLE the specific medium channel most predictive of their food
- Different sub-populations converge on different signal channels
- Signal selectivity is a form of evolved attention / stimulus discrimination
- This extends EXP-011 from 1 signal to N signals

## Proposed Parameters

| Parameter | Proposed Value |
|-----------|---------------|
| VM | Cell v3, CEM=50, R=5 |
| total_ticks | 1,000,000 |
| medium_size | 512 (multiple channels) |
| Signal channels | 4 channels: predictive/random/delayed/noise |
| Food types | 2-4 types, each correlated with a specific channel |
| seeds | 100 (for replication) |
| max_organisms | 500 |

## Experimental Design

### Signal Channels
- Channel 0: Square wave, period=2000, δ=200 (predictive for food type A)
- Channel 1: Random noise (no correlation)
- Channel 2: Square wave, period=3000, δ=300 (predictive for food type B)
- Channel 3: Inverted channel 0 (anti-correlated)

### Food Types
- Food A: energy=10, appears when channel 0 is high
- Food B: energy=15, appears when channel 2 is high
- Food C: energy=5, constant (background)

### Measurement
- SAMPLE frequency per channel (which channels organisms actually read)
- Population divergence into signal-specialized sub-populations
- Fitness advantage of selective vs non-selective organisms

## Implementation Notes

- Requires medium channel indexing in SAMPLE instruction
- May need new Seed H (multi-channel sampling strategy)
- Extend `run_sensemaking_trial()` to support multi-channel food injection

## Why Not Yet Implemented

- EXP-011 signal sensitivity result not yet replicated — prerequisite
- EXP-013 is the "D2 advanced" experiment, depends on D1 results
- Implementation requires medium channel extension (~200 lines)

## Files

None yet — design only.
