# Quickstart & Validation Guide: TUI Visual Explanation

**Feature**: TUI Visual Explanation (`specs/004-tui-visual-explanation`)  
**Status**: Ready for Validation  
**Aligned With**: `design-system.md`, `.specify/memory/constitution.md` (v1.1.0)  

---

## 1. Prerequisites

- Rust stable toolchain ($\ge 1.88.0$, Edition 2024 per Constitution Additional Constraints).
- Modern terminal emulator supporting Unicode box-drawing (kitty, alacritty, iTerm2, WezTerm, GNOME Terminal, Windows Terminal).
- Terminal window dimensions expanded to at least **80 columns × 24 rows**.

---

## 2. Running the Interactive Visual Explanation

### Launch with Default Demo Preset (Karate Club)
```bash
cargo run -p leiden-tui
```

### Launch with a Custom Dataset
```bash
cargo run -p leiden-tui -- path/to/my-graph.edg
```

---

## 3. Step-by-Step Validation Scenarios

### Scenario 1: Initial State & Preset Selection (User Story 1 / FR-002, FR-006)
1. Launch `cargo run -p leiden-tui`.
2. **Observe**:
   - The top **Explanation Panel** displays *"A Messy Network Starting Point"* and plain-English text explaining that no groups have formed yet.
   - The **Graph Visualization** shows nodes in unclustered monochromatic color (`FG_2` neutral gray).
   - Pressing `1`, `2`, or `3` instantly switches between built-in presets (Karate Club, Two Cliques, Random Mess).

### Scenario 2: Dynamic Force-Directed Clustering (User Story 2 / FR-003, FR-004, FR-005)
1. Press `Space` to start Auto-Play (or press `n` / `Right Arrow` to step manually).
2. **Observe**:
   - Nodes physically glide across the 2D canvas towards their assigned community clusters.
   - Nodes and intra-community edges update with vibrant `COMMUNITY_COLORS` (blue, green, amber, coral, teal, lavender).
   - The **Explanation Panel** dynamically updates its headline and analogy text (e.g., *"Finding Best Friend Circles"* during Local Moving).
   - Pressing `t` toggles between `PhaseLevel` (major phase jumps) and `MicroStep` (fine-grained node moves).

### Scenario 3: Final Communities Result (User Story 3 / SC-001)
1. Allow the algorithm to run to convergence.
2. **Observe**:
   - The status bar and explanation panel show the final success state (*"Neat Communities Discovered!"*).
   - The graph clearly displays visually separated, tightly clustered communities.
   - The explanation summarizes the final community count.

### Scenario 4: Terminal Dimension Guard Overlay (FR-007 / Edge Cases)
1. Resize the terminal window below 80 columns or 24 rows.
2. **Observe**:
   - Normal panel rendering pauses immediately.
   - A centered warning modal appears: *"TERMINAL TOO SMALL — Minimum required: 80 × 24"*.
   - Resizing back $\ge 80 \times 24$ immediately restores the interactive UI without losing state.

---

## 4. Automated Testing Suite

Run workspace unit and integration tests:
```bash
cargo test -p leiden-tui
```

Run pedantic clippy lint check:
```bash
cargo clippy --workspace --all-targets -- -D warnings
```
