# ExoMind Cell — GUI Design Proposal

## Overview

A graphical interface for real-time visualization and parameter control of the D0 VM operational closure experiments. Inspired by ALIEN (https://github.com/chrxh/alien) and Lenia.

## Layout

```
+---------------------------------------------------------------+
|  Toolbar: [Play/Pause] [Speed: 1x/10x/100x] [Seed: ___]     |
|           [Mode: v2/v3] [Decay: ON/OFF] [Reset]              |
+----------------------------+----------------------------------+
|                            |                                  |
|     Soup View              |     Statistics Panel             |
|     (main canvas)          |                                  |
|                            |     Population: ████████ 42      |
|     ○ ○  ○                 |     Avg Energy: ████ 85          |
|       ○    ○               |     Food Pool:  ██████████ 500   |
|     ○   ○                  |     Max Gen:    127               |
|          ○  ○              |                                  |
|       ○                    |     --- Instruction Mix ---      |
|     ○    ○   ○             |     EAT:     ████████ 28%       |
|                            |     REFRESH: ██████ 21%         |
|     Each circle = organism |     DIVIDE:  ███ 9%             |
|     Size = cell count      |     DIGEST:  █████ 15%          |
|     Color = energy level   |                                  |
|     Opacity = freshness    |     --- Time Series ---          |
|                            |     [Population graph]           |
|                            |     [Energy graph]               |
|                            |     [REFRESH% graph]             |
+----------------------------+----------------------------------+
|  Timeline: [|<] [<] [>] [>|]  Tick: 42000/500000  [Export]   |
+---------------------------------------------------------------+
```

## Organism Detail View (click to inspect)

```
+------------------------------------------+
|  Organism #42 | Gen 15 | Age 3200       |
|                                          |
|  Cells: [C][C][C][C][S][E][E]           |
|          ^ip                             |
|  Code:  EAT DIGEST REFRESH JMP          |
|  Energy: 35/100  Freshness: 243/255     |
|                                          |
|  Instruction History (last 20):          |
|  EAT EAT DIGEST REFRESH JMP EAT ...    |
+------------------------------------------+
```

## Features

### Core
1. **Soup visualization**: organisms as colored circles, size proportional to cell count, color maps to energy level (red=low, green=high), opacity maps to min freshness
2. **Real-time statistics**: population, energy, instruction mix, food pool
3. **Time series graphs**: population, REFRESH%, energy over time
4. **Play/Pause/Speed control**: 1x, 10x, 100x simulation speed
5. **Parameter panel**: adjust CEM, R, food_per_tick, mutation_rate at runtime

### Advanced
6. **Organism inspector**: click an organism to see its cell composition, code, registers
7. **Lineage view**: track a lineage from parent to children, show mutation events
8. **Heatmap mode**: color organisms by REFRESH frequency, DIVIDE rate, or energy
9. **Export**: save current state, export time series to CSV

### Cell v3 Specific
10. **Cell bar**: each organism shows its cell composition as a colored bar (Code=blue, Energy=yellow, Stomach=green, Data=purple)
11. **Freshness overlay**: per-cell freshness shown as brightness gradient within each organism
12. **REFRESH coverage**: highlight which cells are within current REFRESH radius

## Technology Options

### Option A: egui + eframe (Recommended for MVP)
- **Pros**: Pure Rust, immediate mode GUI, easy to prototype, works on all platforms, built-in plotting (egui_plot)
- **Cons**: Not GPU-accelerated rendering for soup view, limited visual polish
- **Effort**: ~1-2 weeks for MVP
- **Dependencies**: `eframe`, `egui`, `egui_plot`

### Option B: wgpu + winit + egui
- **Pros**: GPU-accelerated soup rendering (can handle 10k+ organisms), egui for panels
- **Cons**: More complex setup, custom rendering code for soup view
- **Effort**: ~3-4 weeks for MVP
- **Dependencies**: `wgpu`, `winit`, `egui-wgpu`

### Option C: Bevy
- **Pros**: Full game engine, ECS architecture fits well with organisms, hot reload
- **Cons**: Heavy dependency, steep learning curve, overkill for data visualization
- **Effort**: ~2-3 weeks but more maintenance

### Recommendation

**Start with Option A (egui)** for rapid iteration. The TUI already validates the core data pipeline. egui can reuse the same data collection code. If performance becomes an issue with >1000 organisms, upgrade the soup view to wgpu while keeping egui for panels (Option B).

## References

- **ALIEN**: Particle-based ALife with GPU acceleration. Key UI elements: spatial view + property inspector + parameter editor
- **Lenia**: Continuous cellular automata. Key UI: heatmap visualization + parameter sweep controls
- **NetLogo**: Classic ALife visualization. Key UI: patch view + monitor widgets + sliders
- **Avida-ED**: Educational Avida interface. Key UI: petri dish + organism viewer + population stats

## Implementation Priority

1. Statistics panel with real-time graphs (reuse TUI data)
2. Soup view with simple circle rendering
3. Play/Pause/Speed controls
4. Organism click-to-inspect
5. Parameter adjustment sliders
6. Cell composition bars
7. Lineage tracking (future)
