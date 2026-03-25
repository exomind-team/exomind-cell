# EXP-015: GATE + Multi-Food Selection (Value-Based Choice)

## Hypothesis

Organisms with GATE mechanism can develop food-type preferences based on experience, while organisms without GATE cannot. This tests D1 value evaluation: the ability to choose between options based on their assessed value to survival.

## Mechanism

Two food types in the pool:
- **Food A** (rich): energy=20, requires 2x DIGEST to extract (thick shell)
- **Food B** (fast): energy=5, requires 1x DIGEST (thin shell)

The food type eaten is written to a special medium channel, so organisms can SAMPLE it. Organisms with GATE + evaluation module can:
1. SAMPLE to detect which food type was just eaten
2. STORE the food type to Data cell
3. GATE controls whether to continue eating (repeat EAT for more of same type) or switch strategy

## Groups (4)

| Group | GATE | Food Types | Purpose |
|-------|------|-----------|---------|
| 1 | ON | A + B mixed | Can organisms develop food preference? |
| 2 | OFF | A + B mixed | Baseline without GATE (random eating) |
| 3 | ON | A only | Control: no choice needed |
| 4 | ON | B only | Control: no choice needed |

## Parameters (based on frozen baseline)

```
CEM=50, R=5, freshness_max=255, mutation_rate=0.001
max_organisms=200, total_ticks=1_000_000
data_cell_gating=true (groups 1,3,4) / false (group 2)

Food injection:
  Food A: 10 per tick (high energy, slow digest)
  Food B: 30 per tick (low energy, fast digest)
  Total effective energy: 10*20 + 30*5 = 350/tick

Seed mix: 20 Seed G (GATE) + 10 Seed A + 10 Seed B

30 seeds per group, rayon parallel, --threads 12
```

## Seed H (new, food-aware with GATE)

```
EAT                    // eat whatever is available
SAMPLE(10)             // r0 = medium[10] (food type indicator: 0=A, >0=B)
STORE r0 → Data        // record food type
DIGEST                 // first digest
LOAD Data → r1         // r1 = recorded food type
CMP r1, r3             // r3 = threshold (if food type A: r1=0, so r0=0)
GATE                   // check adjacent Data cell
  DIGEST               // second DIGEST (only for Food A — gated by Data)
REFRESH
JMP loop
```

When eating Food A (type=0): Data cell = 0 → GATE skips second DIGEST → organism misses half the energy but saves a tick. Evolution pressure: organisms that learn "Food A needs 2 DIGEST" will evolve to NOT skip the second DIGEST (by modifying the GATE/Data cell relationship).

Actually, better design: invert the logic.

## Revised Seed H

```
EAT                    // eat
DIGEST                 // first digest (always)
SAMPLE(10)             // r0 = food type indicator
STORE r0               // save to Data cell
GATE                   // if Data cell = 0 (Food A eaten): execute next
  DIGEST               // SECOND digest (only needed for Food A)
SENSE_SELF r1          // check energy
CMP r1, r4             // enough energy?
GATE                   // Data cell controls DIVIDE
  DIVIDE
REFRESH
JMP loop
```

This way:
- Food A (type=0 in medium): Data cell = 0 → GATE does NOT skip → second DIGEST runs → full 20 energy extracted
- Food B (type>0 in medium): Data cell > 0 → GATE skips second DIGEST → saves 1 instruction cycle

Wait — GATE skips when Data=0, not when Data>0. So:
- Food A: Data=0 → GATE skips next → second DIGEST skipped → only 10/20 energy extracted (BAD)
- Food B: Data>0 → GATE doesn't skip → second DIGEST runs unnecessarily (wastes 1 cycle but no harm)

This is backwards. The organism needs to evolve to INVERT the GATE behavior for food efficiency. That's actually a harder but more interesting test of adaptivity.

## Alternative: Use two Data cells

```
[Data cell 1 (food type)] [GATE] [Code: DIGEST]  ← second DIGEST gated by food type
```

If food type = A (Data=0): GATE skips DIGEST → bad (misses energy)
If food type = B (Data>0): GATE doesn't skip DIGEST → wasteful but not fatal

Evolution pressure: organisms that STORE a non-zero value for Food A (overriding the raw food type signal) will extract full energy. This requires the evaluation module to learn "always write non-zero after eating Food A."

## Required Code Changes

1. **Food type tracking**: When EAT succeeds, write food type to medium[10]
   - Food A: medium[10] = 0
   - Food B: medium[10] = value > 0 (e.g., the food energy amount)

2. **Multi-food with type indicator**: Extend the existing multi_food system
   - Already have simple/complex food pools
   - Add: after EAT, write to medium which type was eaten

3. **Seed H**: New seed with food-aware GATE logic (see above)

4. **Statistics**: Track Food A vs Food B consumption ratios per group

Estimated code changes: ~50 lines (food type indicator in EAT + Seed H + stats)

## Expected Results

### Optimistic
Group 1 (GATE + mixed food) develops food preference: organisms evolve to STORE appropriate values after eating Food A, enabling full energy extraction. Group 2 (no GATE) shows no preference.

### Realistic
Food type preference may not emerge in 1M ticks. The evaluation module needs to simultaneously: (1) detect food type via SAMPLE, (2) STORE correct value, (3) have GATE positioned correctly relative to DIGEST. This is a 3-step coordination that mutation alone may not find.

### Pessimistic
No difference between groups — GATE mechanism not powerful enough for food-type discrimination at this mutation rate and timescale.

## Alternative Approaches

### A. Simpler: food scarcity cycling
Instead of food types, cycle food availability (feast/famine). GATE organisms can learn "feast phase → DIVIDE, famine phase → conserve." Simpler mechanism, more likely to show results.

### B. Harder: food quality sensing
Add a SENSE_FOOD instruction that reads the energy value of food in the Stomach cell. Organisms can then CMP and decide whether to DIGEST once or twice. Requires new instruction but is more direct.

### C. Population-level: specialist vs generalist
Seed some organisms optimized for Food A and others for Food B. See if GATE organisms can switch between strategies while non-GATE organisms stay fixed.

## Recommendation

Start with **Alternative A** (feast/famine cycling with GATE) as EXP-015. It's closest to the EXP-014 design (which worked), just with periodic food switching instead of one-time switching. If that works, graduate to food-type discrimination as EXP-016.
