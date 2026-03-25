# EXP-011: Sense-Making (Signal Prediction) Experiment

## Hypothesis

Organisms under operational closure (freshness decay) will evolve to utilize predictive environmental signals for survival advantage, demonstrating primitive "sense-making" — the construction of meaning from environmental cues through lived experience.

## Mechanism

1. Environmental signal written to stigmergy medium channel 0
2. Food supply tracks the signal with a delta-tick delay
3. Organisms with SAMPLE instruction can read the signal
4. Organisms that "predict" food changes by reading the signal eat more efficiently

## 6 Experimental Groups

| Group | Signal Type | Delta | Purpose |
|-------|------------|-------|---------|
| A | Square wave (T=2000) | 200 | Predictable signal with prediction window |
| B | Sine wave (T=2000) | 200 | Smooth predictable signal |
| C | Real CPU (hash-based) | 200 | Noisy predictable signal |
| D | Square wave (T=2000) | 0 | Synchronous (no prediction advantage) |
| E | Random | 200 | Unpredictable signal (noise control) |
| F | None (constant) | 0 | Baseline (no signal) |

## Predictions

1. Group A (predictable, delayed): highest EAT efficiency — organisms exploit the prediction window
2. Group D (synchronous): no prediction advantage — signal and food change simultaneously
3. Group E (random): no prediction possible — noise provides no survival benefit
4. Group F (no signal): baseline behavior
5. A > D in EAT ratio if prediction is being used
6. A > E in EAT ratio if signal structure matters (not just noise)

## Falsification

- If A == D == E in EAT ratio: signal prediction is not evolving
- If A == F: signal has no behavioral effect

## Parameters

- Cell v3, CEM=50, R=5, freshness_max=255
- 500k ticks, max_organisms=200
- 30 seeds per group (seeds 200-229)
- Base food: 300 per 10 ticks, modulated by delayed signal
- Food formula: base_food * (0.2 + 0.8 * delayed_signal)
- Signal injected to medium[0] as (signal * 200) u8
- Seed mix: 10 Seed A + 10 Seed B + 20 Seed F (prediction-capable)
- rayon parallel execution

## Seed F Design

```
SAMPLE(0)         // r0 = medium[0] (current signal)
STORE r0           // save to Data cell
EAT               // eat
DIGEST             // process food
LOAD Data → r1    // r1 = previous signal reading
CMP r0, r1        // signal rising?
JNZ extra_eat     // yes → eat more
REFRESH → JMP     // normal loop
extra_eat:
  EAT → REFRESH → JMP  // aggressive eating when signal changes
```

## Reproduce

```bash
cargo run --release -- --exp011
```
