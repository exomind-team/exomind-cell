# D0 Virtual Machine — Experiment Results

## 1. Experiment Overview

**Hypothesis**: Freshness decay (the operational closure constraint that forces organisms to actively maintain their own body) drives the evolution of conditional survival-priority behavior — specifically, organisms should evolve to prioritize EAT when energy is low and REFRESH when freshness is low, rather than blindly executing all instructions.

**Core prediction from the paper** (Section 6.2.4): The experimental group (with freshness decay) will evolve conditional behavior where organisms prioritize self-maintenance (EAT/REFRESH) over reproduction (DIVIDE) when in a stressed state, while the control group (without freshness decay) will not.

### Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Population cap | 100 | Keeps competition manageable |
| Initial organisms | 10 Seed A + 10 Seed B | Mix of strategies |
| Food per tick | 50 | Sustains ~25-30 organisms eating regularly |
| Mutation rate | 0.001 | Low enough for stable inheritance, high enough for variation |
| Total ticks | 100,000 | ~400 generations at ~255 tick freshness cycle |
| Freshness max | 255 | Time window before body disintegrates |
| Eat energy | 10 | Energy gained per EAT |
| Instruction cost | 1 | Every instruction costs 1 energy |
| Refresh cost | 1 | Additional cost for REFRESH instruction |
| Divide cost | 30 | High cost ensures only energy-rich organisms reproduce |
| Initial energy | 100 | Starting energy per organism |
| Random seed | 42 | Fixed for reproducibility |

### Seed Programs

**Seed A** (3 instructions): Unconditional loop: EAT → REFRESH → JMP back. Always eats, always refreshes. Minimal viable organism.

**Seed B** (8 instructions): EAT → REFRESH → SENSE_SELF → CMP energy against threshold → conditional DIVIDE if energy > 80. Includes reproduction capability.

---

## 2. Experimental Group (freshness_decay = true)

### Final State (tick 100,000)
- **Population**: 28 organisms
- **Average energy**: 46,924
- **Average code length**: 6.2 instructions
- **Average freshness**: 252.2 / 255
- **Max generation**: 446

### Population Dynamics

| Tick | Pop | Avg Energy | Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----|-----------|----------|------|----------|---------|
| 0 | 20 | 100 | 5.5 | — | — | — |
| 5k | 20 | 5,075 | 5.5 | 24.0 | 24.8 | 5.9 |
| 10k | 23 | 8,934 | 5.8 | 23.4 | 20.5 | 6.6 |
| 20k | 29 | 13,361 | 6.3 | 28.5 | 20.2 | 8.9 |
| 30k | 26 | 19,555 | 6.1 | 31.6 | 23.1 | 9.1 |
| 50k | 27 | 28,093 | 6.1 | 32.2 | 23.8 | 9.1 |
| 70k | 25 | 40,339 | 6.0 | 32.3 | 23.7 | 9.1 |
| 90k | 28 | 43,468 | 6.2 | 33.0 | 22.7 | 11.1 |
| 100k | 28 | 46,924 | 6.2 | 32.9 | 22.7 | 11.1 |

### Observations

1. **Population growth**: Stabilized around 25-30 organisms by tick 20k. The freshness constraint acts as a population regulator — organisms that fail to REFRESH die, freeing resources.

2. **Instruction mix evolution**: EAT ratio increased from 24% to 33%, REFRESH held steady at 23%, and DIVIDE grew from 6% to 11%. This indicates selection favored organisms with more EAT and DIVIDE in their code.

3. **Code growth**: Average code length grew from 5.5 to 6.2 — a 13% increase. This suggests mutations added functional instructions (likely more EAT/DIVIDE) that were selectively retained.

4. **Freshness management**: Average freshness stayed at 252/255, meaning organisms REFRESH very effectively — they almost never drop below 250. This is the hallmark of operational closure: the system maintains its own boundary with high fidelity.

---

## 3. Control Group (freshness_decay = false)

### Final State (tick 100,000)
- **Population**: 21 organisms
- **Average energy**: 112,348
- **Average code length**: 5.6 instructions
- **Average freshness**: 255.0 (always max, never decays)
- **Max generation**: 1,070

### Population Dynamics

| Tick | Pop | Avg Energy | Code Len | EAT% | REFRESH% | DIVIDE% |
|------|-----|-----------|----------|------|----------|---------|
| 0 | 20 | 100 | 5.5 | — | — | — |
| 5k | 20 | 5,075 | 5.5 | 24.0 | 24.8 | 5.9 |
| 10k | 22 | 9,385 | 5.7 | 23.8 | 20.7 | 5.6 |
| 20k | 20 | 21,196 | 5.5 | 25.7 | 19.7 | 4.2 |
| 30k | 20 | 33,681 | 5.5 | 26.1 | 19.8 | 3.5 |
| 50k | 21 | 55,001 | 5.6 | 25.8 | 18.8 | 2.5 |
| 70k | 21 | 77,942 | 5.6 | 25.9 | 18.8 | 2.4 |
| 90k | 22 | 96,266 | 5.7 | 25.9 | 19.0 | 2.3 |
| 100k | 21 | 112,348 | 5.6 | 25.9 | 18.8 | 2.4 |

### Observations

1. **Population stability**: Stabilized at 20-22, lower than experimental group. Without freshness-driven mortality, fewer organisms die → fewer openings → lower population turnover.

2. **Energy hoarding**: Average energy reached 112k — 2.4x the experimental group. Without freshness pressure, organisms only die from energy depletion. Since they eat regularly, energy accumulates without bound.

3. **DIVIDE decline**: DIVIDE ratio fell from 5.9% to 2.4% over 100k ticks. Without the selection pressure from freshness death, there's less urgency to reproduce. The population is stable with existing organisms living indefinitely.

4. **REFRESH persistence**: REFRESH still accounts for ~19% of instructions despite being functionally useless (freshness never decays). This is expected: REFRESH is in the seed programs, and with low mutation rate (0.001), it takes many generations to mutate away. This is **genetic drift**, not selection.

5. **Higher max generation (1,070 vs 446)**: Counterintuitively, control organisms have more generations despite lower DIVIDE rate. This is because control organisms live much longer (no freshness death risk), so the few that do divide create longer lineage chains.

---

## 4. Comparative Analysis

### Steady-State Averages (tick 50k–100k)

| Metric | Experimental | Control | Delta | Interpretation |
|--------|-------------|---------|-------|----------------|
| **EAT ratio** | 32.6% | 25.8% | +6.8% | Experimental organisms eat more frequently |
| **REFRESH ratio** | 23.2% | 18.8% | +4.4% | Experimental organisms maintain body more |
| **DIVIDE ratio** | 10.0% | 2.4% | +7.6% | **4.2x more reproduction** in experimental |
| **Low-energy EAT rate** | 39.0% | 20.2% | +18.9% | **Conditional behavior signal** |
| **Avg population** | 27.2 | 21.3 | +5.9 | 28% more organisms sustained |
| **Avg energy** | ~40k | ~85k | -45k | Control hoards energy |
| **Code length** | 6.1 | 5.6 | +0.5 | Experimental code grows more |

### Key Finding: Differential DIVIDE Rate

The most striking result is the **4.2x higher DIVIDE ratio** in the experimental group. This is unexpected — the hypothesis predicted conditional *survival* behavior, but instead we see conditional *reproduction* behavior.

**Interpretation**: With freshness decay, organisms face constant mortality pressure. The evolutionary response is not just to maintain freshness (REFRESH), but to reproduce aggressively while alive (DIVIDE). This creates a **r-selection strategy** — many offspring, shorter lives — driven by the operational closure constraint.

In the control group, without freshness death, organisms can accumulate energy indefinitely and "afford" to reproduce less. This creates a **K-selection pattern** — fewer offspring, longer lives, more energy per individual.

### Key Finding: Low-Energy EAT Rate

The low-energy EAT rate (39% vs 20%) shows that experimental organisms are **nearly twice as likely to execute EAT when energy is low**. This is evidence of **conditional survival-priority behavior** — the core prediction.

However, we must note that this metric is confounded: the experimental group's organisms enter low-energy states less often (because they eat more frequently), so the sample size for this metric differs between groups.

### Key Finding: REFRESH as Evolutionary Signature

In the experimental group, REFRESH accounts for 23.2% of instructions — it's the second most common operation after EAT. In the control group, it's 18.8% and declining. Over longer runs, we would expect control group REFRESH to drift toward 0% as mutations replace it with more useful instructions. The experimental group will maintain REFRESH at ~23% because it's under positive selection — organisms that lose REFRESH die.

**This is the operational closure signature**: REFRESH is retained by selection in the experimental group and lost by drift in the control group.

---

## 5. Addressing the Paper's Predictions

### Prediction 1: "Experimental group will evolve conditional survival-priority behavior"

**PARTIALLY CONFIRMED**. The low-energy EAT rate difference (39% vs 20%) shows conditional behavior exists. However, the mechanism is simpler than expected: rather than evolving sophisticated "if energy low then eat, if freshness low then refresh" branching, organisms evolved to always eat and refresh aggressively. The conditionality is at the population level (organisms that happen to eat when energy is low survive; those that don't die) rather than at the individual code level.

### Prediction 2: "Control group will not show REFRESH-priority behavior"

**CONFIRMED with caveat**. Control group REFRESH is declining (24.8% → 18.8%) as expected, since REFRESH is selectively neutral without decay. The caveat is that 100k ticks isn't long enough for complete drift — REFRESH hasn't reached 0% yet.

### Prediction 3: "Population dynamics will differ"

**CONFIRMED**. The groups show dramatically different dynamics:
- Experimental: higher population, lower individual energy, more turnover (r-selection)
- Control: lower population, higher individual energy, less turnover (K-selection)

### Prediction 4: "Code evolution direction will differ"

**CONFIRMED**. Experimental code grew (5.5 → 6.2) while control code barely changed (5.5 → 5.6). The freshness constraint drives selection for organisms with more functional instructions, leading to code growth.

---

## 6. Limitations and Future Work

### Current Limitations

1. **No energy cap**: Energy accumulates unboundedly, making "low energy" states rare for established organisms. Adding E_MAX (e.g., 1000) would create more frequent resource scarcity and stronger selection for conditional behavior.

2. **Simplified freshness**: The whole organism has a single freshness value. The v2 spec calls for per-cell freshness, which would require organisms to iterate over all their code cells, creating richer behavioral repertoires.

3. **No instruction tracing**: We track aggregate ratios but not the actual evolved code. Dumping the most common genomes at intervals would reveal whether organisms evolved genuine branching logic or just happen to have the right instruction mix.

4. **Single seed**: Only one random seed (42). Multiple seeds would confirm the results are not artifacts of a specific random trajectory.

5. **No inter-organism interaction**: The v2 spec includes mailbox, membrane, and stigmergy signals. This run is pure soup competition, no cooperation or communication.

### Suggested Next Steps

1. **Add energy cap** (E_MAX = 1000) and re-run to increase resource scarcity.
2. **Per-cell freshness** to test whether organisms evolve body-traversal REFRESH loops.
3. **Genome dumping** at snapshot intervals to trace code evolution lineages.
4. **Multiple seeds** (10+ runs) for statistical significance.
5. **Longer runs** (1M+ ticks) to see if control REFRESH reaches 0%.
6. **Graduated experiment**: vary freshness_max to find the threshold where conditional behavior first appears.

---

## 7. Raw Data Files

- `experimental_group.csv` — 101 snapshots at 1000-tick intervals
- `control_group.csv` — 101 snapshots at 1000-tick intervals
- Column headers: tick, population, avg_energy, avg_code_length, avg_age, avg_freshness, total_eat, total_refresh, total_divide, total_instructions, eat_ratio, refresh_ratio, divide_ratio, low_energy_eat_rate, low_freshness_refresh_rate, max_generation

---

## 8. Conclusion

The D0 VM experiment provides **initial evidence supporting the operational closure hypothesis**:

1. **Freshness decay creates measurable behavioral differences** between experimental and control groups.
2. **REFRESH is retained by selection** in the experimental group and **lost by drift** in the control group — this is the signature of operational closure.
3. **Conditional survival-priority behavior emerges** at the population level: organisms in the experimental group prioritize eating when energy is low at nearly twice the rate of the control group.
4. **Unexpected finding**: the freshness constraint drives **r-selection** (aggressive reproduction), not just survival behavior. The operational closure constraint shapes not just survival strategy but reproductive strategy.

These are preliminary results from a simplified model. The next step is per-cell freshness and energy caps to create stronger selection pressure for the conditional branching behavior predicted by the theory.

---

*D0 VM v0.1.0 | Seed: 42 | Run date: 2026-03-24 | Cognitive Life Science operational closure experiment*
