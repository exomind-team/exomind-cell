# D0 Virtual Machine

A minimal artificial life virtual machine for testing the **operational closure hypothesis**: that freshness decay (the constraint forcing organisms to actively maintain their own body) drives the evolution of conditional survival-priority behavior.

Part of the [Cognitive Life Science](https://github.com/exomind-team) research program.

## Quick Start

```bash
# Run all experiments (5 seeds x 2 groups + 7 competition levels)
cargo run --release

# Run tests
cargo test
```

Results are written to `RESULTS.md` and per-experiment CSV files.

## What It Does

The D0 VM simulates a "soup" of digital organisms that must:
- **EAT** to gain energy (or starve to death)
- **REFRESH** to maintain body integrity (or disintegrate from freshness decay)
- **DIVIDE** to reproduce (optional — survival comes first)

The core experiment compares:
- **Experimental group**: freshness decays every tick (organisms must actively self-maintain)
- **Control group**: freshness never decays (organisms only die from energy depletion)

## Key Results

See [RESULTS.md](RESULTS.md) for full analysis. Summary across 5 random seeds:

| Metric | Experimental | Control | Interpretation |
|--------|-------------|---------|----------------|
| REFRESH ratio | 19.7% +/- 1.0% | 13.9% +/- 5.4% | REFRESH under positive selection vs genetic drift |
| Avg energy | 498 | 611 | Experimental organisms spend more energy on maintenance |
| DIVIDE ratio | 8.4% | 7.1% | Slightly more reproduction under mortality pressure |

**Competition experiment** (varying food scarcity): food_per_tick ~40 is the threshold where DIVIDE first appears. Below this, organisms focus entirely on survival (EAT + REFRESH).

## Architecture

```
src/
  instruction.rs  — 12-instruction set (NOP, INC, DEC, CMP, JMP, JNZ, LOAD, STORE, SENSE_SELF, EAT, REFRESH, DIVIDE)
  organism.rs     — Organism struct, Config, seed programs
  world.rs        — World simulation engine, statistics, genome dumps
  experiment.rs   — Experiment runner, report generation
  main.rs         — Entry point, multi-seed and competition experiments
```

See [docs/design.md](docs/design.md) for VM architecture details and [docs/experiments.md](docs/experiments.md) for experiment parameters.

## Tech Stack

- **Language**: Rust (2021 edition)
- **Dependencies**: `rand 0.8` (deterministic PRNG with seed control)
- **Tests**: 13 unit tests via `cargo test`

## License

[CCOPL-1.0](LICENSE) (Collective Commons Open Public License)
