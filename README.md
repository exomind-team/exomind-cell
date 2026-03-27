<h1 align="center">ExoMind Cell</h1>

<p align="center">
  <strong>Operational Closure Virtual Machine for Cognitive Life Science</strong>
</p>

<p align="center">
  <a href="https://github.com/exomind-team/exomind-cell/actions"><img src="https://github.com/exomind-team/exomind-cell/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-CCOPL--1.0-blue" alt="License: CCOPL-1.0"></a>
  <a href="https://github.com/exomind-team/exomind-cell/releases"><img src="https://img.shields.io/github/v/release/exomind-team/exomind-cell" alt="Release"></a>
  <a href="https://doi.org/10.5281/zenodo.19221442"><img src="https://zenodo.org/badge/DOI/10.5281/zenodo.19221442.svg" alt="DOI"></a>
</p>

<p align="center">
  A minimal artificial life virtual machine testing whether <strong>freshness decay</strong><br>
  (the constraint forcing organisms to actively maintain their own body)<br>
  drives the evolution of conditional survival-priority behavior.
</p>

<p align="center">
  English | <a href="README-zh.md">中文</a>
</p>

<p align="center">
  <img src="docs/screenshots/tui-stigmergy.png" width="800" alt="TUI Screenshot">
</p>

---

## Paper

> **Cognitive Life Science: Preliminary Experimental Validation of the Operational Closure Framework**
>
> Lin, JiaHao. (2026). *Preprint.* DOI: [10.5281/zenodo.19221442](https://doi.org/10.5281/zenodo.19221442)

## Citing This Work

If you use ExoMind Cell in your research, please cite:

```bibtex
@article{lin2026cognitive,
  title     = {Cognitive Life Science: Preliminary Experimental Validation of the Operational Closure Framework},
  author    = {Lin, JiaHao},
  year      = {2026},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19221442},
  url       = {https://doi.org/10.5281/zenodo.19221442}
}
```

APA format:

> Lin, J. (2026). Cognitive Life Science: Preliminary Experimental Validation of the Operational Closure Framework. *Zenodo*. https://doi.org/10.5281/zenodo.19221442

GB/T 7714 format:

> [1] LIN J H. Cognitive Life Science: Preliminary Experimental Validation of the Operational Closure Framework[R/OL]. Zenodo, 2026. https://doi.org/10.5281/zenodo.19221442.

---

## Quick Start

```bash
# Interactive TUI
cargo run --release -- --tui --cell          # Cell v3 mode (recommended)
cargo run --release -- --tui                 # Classic v2 mode

# GUI spike (egui on wgpu backend)
cargo run -- --gui                           # Native soup view prototype
cargo run -- --gui --gui-smoke-test 5        # Open + auto-close smoke test

# Headless experiments
cargo run --release -- --cell                # v3 cell experiments (5 seeds, 500k ticks)
cargo run --release -- --stats               # 100-seed parallel analysis (2M ticks)
cargo run --release -- --run-v2              # v2 global energy experiments

# Tests
cargo test                                   # 33 unit tests
```

## Features

- **Two VM architectures**: v2 (global energy) and v3 (per-cell freshness with Code/Energy/Stomach/Data cells)
- **14-instruction set**: NOP, INC, DEC, CMP, JMP, JNZ, LOAD, STORE, SENSE_SELF, EAT, REFRESH, DIVIDE, EMIT, SAMPLE
- **Stigmergy**: shared chemical medium for indirect organism communication
- **Interactive TUI**: real-time visualization with pause, step, speed control, organism inspector
- **GUI spike**: `egui + wgpu` soup view with GUI-only force-directed positions
- **Statistical analysis**: bootstrap CI, Mann-Whitney U, Kolmogorov-Smirnov test
- **Parallel execution**: rayon-powered multi-seed experiments (uses all CPU cores)
- **100-seed validation**: all behavioral metrics p<0.001

## GUI Spike

The current GUI prototype is a **non-semantic soup view**:

- `CellWorld` stays unchanged; no spatial coordinates are added to the VM
- organism positions live only in the GUI layout helper
- rendering uses `eframe` with the `wgpu` backend
- pause now freezes both VM ticks and the force-directed layout
- an in-scene performance overlay reports CPU frame/sim/layout/UI timings
- circles encode:
  - color = energy
  - radius = cell count
  - alpha = freshness

The current render path is `egui painter on wgpu`. The code also leaves an explicit seam for a later custom `wgpu` render pass and compute-based layout.

### GUI Usage Notes

- `Pause` stops simulation ticks and freezes the soup layout.
- `Step` advances a single simulation step while paused.
- `Ticks / frame` trades visual smoothness for faster evolution; high values can saturate the CPU.
- Red flicker mostly means low-energy / low-freshness organisms changing state quickly, not a rendering bug.
- Organisms drifting toward the border are following the GUI layout equilibrium, not real VM movement semantics.
- If CJK text still renders incorrectly, set `EXOMIND_CELL_UI_FONT` to a local CJK font file such as `C:\Windows\Fonts\NotoSansSC-VF.ttf`.

## TUI Controls

| Key | Action |
|-----|--------|
| `p` | Pause / Resume |
| `q` | Quit |
| `h` | Help overlay |
| `s` | Single step (when paused) |
| `i` | Inspect organism (cell-level detail) |
| `<` `>` | Navigate organisms in inspector |
| `+` `-` | Speed up / down (1-1000x) |

## Key Results

### 100-Seed Large-Scale (Cell v3, 2M ticks)

| Metric | Experimental | Control | p-value | Cohen's d |
|--------|-------------|---------|---------|----------|
| REFRESH ratio | 16.8% +/- 7.3% | 13.7% +/- 0.4% | <0.0001 | 0.59 |
| EAT ratio | 22.6% +/- 8.7% | 15.3% +/- 0.4% | <0.0001 | 1.19 |
| Population | 114 +/- 36 | 135 +/- 6 | 0.0001 | -0.78 |
| Avg Energy | 33.9 +/- 17.6 | 49.2 +/- 2.4 | <0.0001 | -1.22 |

REFRESH 95% CI [0.016, 0.044] excludes 0 -- the operational closure effect is statistically confirmed.

See [RESULTS.md](RESULTS.md) for full analysis. Experiment registry: [docs/experiments.md](docs/experiments.md).

## Architecture

```
src/
  instruction.rs  -- 14-instruction set + random/mutate/display
  organism.rs     -- v2 Organism, Config, seed programs (A/B/C)
  world.rs        -- v2 World simulation engine
  experiment.rs   -- v2 experiment runner, report generation
  cell_vm.rs      -- v3 Cell-based VM (Cell types, CellOrganism, CellWorld)
  gui.rs          -- egui/wgpu native GUI spike
  soup.rs         -- GUI-only soup layout + render snapshot helpers
  stats.rs        -- Statistical tests (bootstrap, Mann-Whitney, KS)
  tui.rs          -- ratatui terminal visualization (v2 + Cell v3)
  main.rs         -- CLI entry point
data/             -- experiment CSV files and genome dumps
docs/
  design.md       -- VM architecture
  experiments.md  -- experiment registry (EXP-001 to EXP-007)
  gui-design-proposal.md -- future GUI design (egui/wgpu)
```

## Documentation

- [VM Design](docs/design.md) -- instruction set, organism structure, cell types
- [Experiment Registry](docs/experiments.md) -- all 7 experiments with parameters and results
- [GUI Proposal](docs/gui-design-proposal.md) -- future graphical interface design
- [GUI Spike Plan](docs/plans/2026-03-26-egui-wgpu-soup-view.md) -- implementation plan for the current prototype

## Tech Stack

- **Language**: Rust 2021
- **Dependencies**: `rand 0.8`, `rayon 1.10`, `ratatui 0.29`, `crossterm 0.28`, `eframe 0.31`
- **Tests**: 33 unit tests
- **CI**: GitHub Actions (build + test + clippy)

## License

[CCOPL-1.0](LICENSE) (Collective Commons Open Public License)
