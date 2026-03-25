# ExoMind Cell

A minimal artificial life virtual machine for testing the **operational closure hypothesis**: that freshness decay (the constraint forcing organisms to actively maintain their own body) drives the evolution of conditional survival-priority behavior.

Part of the [Cognitive Life Science](https://github.com/exomind-team) research program. Repository: [exomind-team/exomind-cell](https://github.com/exomind-team/exomind-cell).

## Quick Start

```bash
# Headless experiments
cargo run --release                          # v2 global energy (5 seeds, 500k ticks)
cargo run --release -- --cell                # v3 cell-based (CEM=50, 5 seeds, 500k ticks)
cargo run --release -- --stats               # 100-seed statistical analysis (rayon parallel, 2M ticks)

# TUI: real-time terminal visualization
cargo run --release -- --tui                 # v2 mode
cargo run --release -- --tui --no-decay      # v2 control group
cargo run --release -- --tui --stigmergy     # v2 with EMIT/SAMPLE
cargo run --release -- --tui --cell          # v3 cell-based mode
cargo run --release -- --tui --cell --no-decay  # v3 control group

# Tests
cargo test                                    # 24 unit tests
```

<!-- To capture TUI screenshot: run `cargo run --release -- --tui`, wait a few seconds, then screenshot -->
<!-- ![TUI Screenshot](docs/screenshots/tui-running.png) -->

Results: [RESULTS.md](RESULTS.md) (consolidated, 7 experiments). CSV data in `data/`.
Experiment registry: [docs/experiments.md](docs/experiments.md).

## Two VM Modes

### v2: Global Energy Model

Organisms have a global `energy: i32` and global `freshness: u8`. Simple but effective for demonstrating the operational closure hypothesis.

- 14 instructions: NOP, INC, DEC, CMP, JMP, JNZ, LOAD, STORE, SENSE_SELF, EAT, REFRESH, DIVIDE, EMIT, SAMPLE
- Stigmergy: shared medium for indirect communication (EMIT/SAMPLE)

### v3: Cell-based Model (`--cell`)

Organisms are composed of heterogeneous **Cells** — each with independent freshness decay:

- **Code cells**: contain instructions (the program IS the body)
- **Energy cells**: store energy (total energy = sum of all Energy cells)
- **Stomach cells**: buffer for digestion (EAT fills Stomach, DIGEST moves to Energy)
- **Per-cell freshness**: each cell decays independently; dead cells are removed, not the whole organism
- **Local REFRESH**: only refreshes cells within radius R of current instruction pointer

This creates the core tension: bigger body = more energy storage = more REFRESH needed.

## Key Results

### v2 (500k ticks, 5 seeds)

| Metric | Experimental | Control | Interpretation |
|--------|-------------|---------|----------------|
| REFRESH ratio | 24.5% +/- 4.6% | 21.4% +/- 6.5% | REFRESH under positive selection |
| E_MAX effect | +9.8% delta | +1.9% delta | Energy cap amplifies REFRESH difference |

### v3 Cell-based

| Setup | Exp Pop | Ctrl Pop | Exp DIVIDE | Ctrl DIVIDE |
|-------|---------|----------|-----------|------------|
| CEM=20 | 3-26 | 27-29 | 0-9% | 4-5% |
| CEM=50 | 7 | — | 8% | — |

CEM (cell energy max) is the threshold parameter for reproduction under per-cell freshness decay.

## Architecture

```
src/
  instruction.rs  — 14-instruction set + random/mutate/display
  organism.rs     — v2 Organism, Config, seed programs (A/B/C)
  world.rs        — v2 World simulation engine
  experiment.rs   — v2 experiment runner, report generation
  cell_vm.rs      — v3 Cell-based VM (Cell types, CellOrganism, CellWorld)
  tui.rs          — ratatui terminal visualization (v2 mode)
  main.rs         — CLI entry point (--tui, --cell flags)
data/             — experiment CSV files and genome dumps
docs/
  design.md       — VM architecture details
  experiments.md  — experiment parameters and file index
```

## Tech Stack

- **Language**: Rust (2021 edition)
- **Dependencies**: `rand 0.8`, `ratatui 0.29`, `crossterm 0.28`
- **Tests**: 21 unit tests via `cargo test`

## License

[CCOPL-1.0](LICENSE) (Collective Commons Open Public License)
