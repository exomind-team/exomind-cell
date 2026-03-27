# Egui WGPU Soup View Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a runnable `egui + wgpu` spike that visualizes the existing `CellWorld` as a non-semantic soup view with a CPU force-directed layout, without changing VM logic.

**Architecture:** Keep `CellWorld` as the simulation core and add a separate GUI entrypoint that advances the world on the CPU, derives a render snapshot, and renders circles in an `egui` canvas first. Keep the rendering boundary explicit so the soup view can later swap to a custom `wgpu` callback/instanced pipeline without touching simulation code.

**Tech Stack:** Rust 2021, `eframe`/`egui` with `wgpu` renderer, existing `CellWorld`, unit tests via `cargo test`

---

### Task 1: Add the failing layout tests

**Files:**
- Modify: `src/cell_vm.rs`
- Test: `src/cell_vm.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_soup_layout_initializes_one_particle_per_alive_organism() {
    // Build a small world, collect a render snapshot, and assert counts match.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test soup_layout -- --nocapture`
Expected: FAIL because the layout/render helper types do not exist yet.

**Step 3: Write minimal implementation**

```rust
pub struct SoupParticle {
    pub organism_id: u64,
    pub x: f32,
    pub y: f32,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test soup_layout -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cell_vm.rs
git commit -m "test: cover soup layout particle initialization"
```

### Task 2: Add a stable CPU force layout helper

**Files:**
- Modify: `src/cell_vm.rs`
- Test: `src/cell_vm.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_soup_layout_step_keeps_particles_inside_view_bounds() {
    // Step the layout and assert all coordinates stay within 0..1 space.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test soup_layout_step -- --nocapture`
Expected: FAIL because the step/update function is missing.

**Step 3: Write minimal implementation**

```rust
pub fn step_force_layout(particles: &mut [SoupParticle], dt: f32) {
    // Apply simple repulsion + center attraction + clamp to normalized bounds.
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test soup_layout_step -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cell_vm.rs
git commit -m "feat: add cpu force layout for soup view"
```

### Task 3: Add a GUI entrypoint with egui and wgpu backend

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/gui.rs`

**Step 1: Write the failing test or build gate**

```rust
// Add a CLI flag expectation by invoking the new GUI module from main.
```

**Step 2: Run build to verify it fails**

Run: `cargo build`
Expected: FAIL because `gui` module and GUI dependencies are not wired yet.

**Step 3: Write minimal implementation**

```rust
if args.iter().any(|a| a == "--gui") {
    gui::run_cell_gui()?;
}
```

**Step 4: Run build to verify it passes**

Run: `cargo build`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/gui.rs
git commit -m "feat: add egui wgpu gui entrypoint"
```

### Task 4: Render the soup view and control panel

**Files:**
- Create: `src/gui.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing behavior check**

```bash
cargo run -- --gui
```

Expected: app starts but soup panel is empty or panics before rendering.

**Step 2: Write minimal implementation**

```rust
// Build a left control panel and central soup panel.
// Advance CellWorld on update and draw circles with egui painter.
```

**Step 3: Run the behavior check**

Run: `cargo run -- --gui`
Expected: a native window opens, organisms are visible as circles, and pause/step controls work.

**Step 4: Commit**

```bash
git add src/gui.rs src/main.rs
git commit -m "feat: render soup view prototype with egui"
```

### Task 5: Prepare the path to custom wgpu rendering

**Files:**
- Modify: `src/gui.rs`
- Modify: `README.md`
- Modify: `README-zh.md`

**Step 1: Write the failing documentation/build check**

Run: `cargo build`
Expected: PASS, but no explicit render-boundary abstraction or docs yet.

**Step 2: Write minimal implementation**

```rust
enum SoupRendererMode {
    EguiPainter,
    WgpuPlaceholder,
}
```

**Step 3: Run verification**

Run: `cargo test && cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git add src/gui.rs README.md README-zh.md
git commit -m "docs: document soup gui spike and wgpu upgrade path"
```
