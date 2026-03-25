# EXP-016: Dual-Pressure Value Gradient (Energy x Freshness Priority)

## Hypothesis

Under simultaneous energy and freshness pressure, organisms with freshness decay will develop gradient-based priority behavior: whichever dimension is closer to the death boundary gets addressed first. Control organisms (no decay) will show flat behavior since only energy matters.

## Theoretical Basis

From the value gradient analysis (r=0.62): organisms already show energy-dependent EAT/REFRESH ratios. But that analysis aggregated all organisms. EXP-016 isolates the **decision point**: when both energy AND freshness are low, which gets priority?

In biological terms: when both starving and injured, does an organism eat first or repair first? The answer should depend on which threat is more immediate.

## Mechanism

### Environmental manipulation

Cycle between two stress phases:
- **Phase A (energy stress)**: food_per_tick=20 (low), but REFRESH_RADIUS=8 (easy maintenance)
- **Phase B (freshness stress)**: food_per_tick=200 (high), but REFRESH_RADIUS=2 (hard maintenance)
- Cycle every 50k ticks (10 cycles in 500k total)

This forces organisms into alternating single-dimension stress. The question: do they adapt their EAT/REFRESH ratio to the current stressor?

### Measurement

Per-phase instruction ratios:
- In Phase A: EAT% should be higher (energy is the bottleneck)
- In Phase B: REFRESH% should be higher (freshness is the bottleneck)
- The RATIO of EAT/REFRESH should flip between phases

### Control

Control group (no decay): should show high EAT in both phases (only energy matters), REFRESH constant regardless of phase.

## Groups (3)

| Group | Decay | Phase cycling | Purpose |
|-------|-------|-------------|---------|
| 1 | ON | A/B alternating | Does behavior adapt to current stressor? |
| 2 | OFF | A/B alternating | Baseline: only energy matters |
| 3 | ON | No cycling (constant mid-params) | Control for cycling effect |

## Parameters

```
CEM=50, freshness_max=255, mutation_rate=0.001
max_organisms=200, total_ticks=500_000
Phase A: food=20, R=8
Phase B: food=200, R=2
Cycle period: 50k ticks (25k per phase)
Constant (group 3): food=110, R=5

Initial: 20 Seed A + 20 Seed B
30 seeds per group, 12 threads
```

## Predictions

### Strong prediction
Group 1 EAT/REFRESH ratio flips between phases:
- Phase A: EAT% > REFRESH% (energy-limited)
- Phase B: REFRESH% > EAT% (freshness-limited)

### Weak prediction
Group 2 (no decay) shows NO flip: EAT% > REFRESH% in both phases.

### Falsification
If Group 1 EAT/REFRESH ratio does NOT change between phases, organisms are not responding to the current stressor.

## Required Code Changes

1. **Per-phase stats**: split snapshot intervals by phase (A vs B)
2. **Dynamic R**: change `config.refresh_radius` per tick (or pass as world state)
3. **Per-phase CSV output**: separate columns for Phase A and Phase B ratios

Estimated: ~40 lines to add phase tracking + ~20 lines for dynamic R.

## Alternative: Simpler version

If dynamic R is too complex, use a simpler proxy:
- Phase A: food=20, freshness_max=255 (energy stress, freshness easy)
- Phase B: food=200, freshness_max=50 (freshness stress, energy easy)

This avoids changing R dynamically but creates the same dual-pressure effect.

## Risk Assessment

Medium-high risk of negative result: organisms may not adapt fast enough within 25k-tick phases, especially since the seed programs have fixed instruction sequences. The "adaptation" would need to come through selective survival (organisms whose fixed ratio happens to match the current phase survive better), not individual behavioral change.

This is actually a strength: it tests whether **population-level** value gradients emerge from **individual-level** fixed programs under alternating selection pressures.
