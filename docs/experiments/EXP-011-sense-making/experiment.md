# EXP-011: Sense-Making (Signal Prediction)

## Hypothesis

Organisms under operational closure will evolve to utilize predictive environmental signals for survival advantage, demonstrating primitive "sense-making."

## Mechanism

Signal written to medium[0]. Food tracks signal with delta-tick delay. Seed F can SAMPLE signal, store it, compare with previous reading, and eat more when signal changes.

## Groups (6)

| Group | Signal | Delta | Purpose |
|-------|--------|-------|---------|
| A | Square wave (T=2000) | 200 | Predictable + prediction window |
| B | Sine wave (T=2000) | 200 | Smooth predictable |
| C | CPU hash | 200 | Noisy predictable |
| D | Square wave | 0 | Synchronous (no prediction advantage) |
| E | Random | 200 | Unpredictable (noise control) |
| F | None | -- | Baseline |

## Parameters

Cell v3, CEM=50, R=5, 500k ticks, 30 seeds/group, max_org=200, rayon parallel.

## Reproduce

```bash
cargo run --release -- --exp011
```

## Results

| Group | Pop | Energy | EAT% | REFRESH% | DIVIDE% |
|-------|-----|--------|------|----------|---------|
| A (square d200) | 11.4 | 60.5 | 28.9 | 17.0 | 5.7 |
| B (sine d200) | 13.8 | 50.6 | 31.7 | 13.0 | 7.2 |
| C (cpu d200) | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| D (square d0) | 14.2 | 51.8 | 29.1 | 14.3 | 7.0 |
| E (random d200) | 14.3 | 56.4 | 28.1 | 14.7 | 6.3 |
| F (no signal) | 4.3 | 94.2 | 25.1 | 24.1 | 0.7 |

## Statistical Tests

| Comparison | p | d | Result |
|-----------|---|---|--------|
| A vs D (predict vs sync) | 0.43 | -0.04 | Not significant |
| A vs E (predict vs random) | 0.58 | 0.11 | Not significant |
| A vs F (signal vs none) | **0.03** | **0.72** | **Significant** |

## Conclusions

1. Signal modulation significantly changes behavior (A vs F: p=0.03)
2. Signal predictability not yet exploited (A vs D: p=0.43)
3. "Sensitivity-to-prediction gap" marks boundary between D0 and D1
