# Implementation Plan: TUI Visual Explanation

**Branch**: `004-tui-visual-explanation` | **Date**: 2026-09-01 | **Spec**: [specs/004-tui-visual-explanation/spec.md](file:///home/luis/development/leiden/specs/004-tui-visual-explanation/spec.md)

**Input**: Feature specification for TUI Visual Explanation, aligned with `design-system.md` and Leiden Constitution v1.1.0.

---

## Summary

Deliver an intuitive, interactive visual explanation experience in `leiden-tui` for non-technical users. The implementation introduces:
1. A **2D Force-Directed Layout Simulation** (`ForceSimulation`) on Ratatui's `Canvas` widget, allowing users to watch a messy unclustered network physically glide into neat, color-coded community clusters (FR-001, FR-002, FR-003).
2. A **3-Tier Explanatory Text Panel** (`ExplanationPanel`) presenting a Step Headline, Plain-English Intuitive Analogy ($\le$ 8th-grade reading level, SC-003), and Live Stat Badges (FR-004, SC-001).
3. **Interactive Playback & Dual-Granularity Stepping** (`PlaybackController`) supporting Play/Pause (`Space`), Step (`n`), and Granularity toggling (`t` for Phase-level vs. Micro-step) (FR-005).
4. **Built-in Curated Demo Presets** (Karate Club, Two Cliques, Random Mess) with key shortcuts `1`, `2`, `3` and custom dataset support (FR-006).
5. **Screen Readability & Responsive Dimension Guard** enforcing minimum $80 \times 24$ geometry with a dedicated warning overlay (FR-007) and full WCAG contrast alignment with `design-system.md`.

---

## Technical Context

**Language/Version**: Rust stable 1.88.0+ (Edition 2024)  
**Primary Dependencies**: `ratatui` 0.30.2 (pinned), `crossterm` 0.29.0, `tracing` 0.1.41, `thiserror` 2.0.12  
**Storage**: N/A (In-memory graph adjacency & CSR partition data structures)  
**Testing**: `cargo test -p leiden-tui`, unit tests for physics & readability, Ratatui `TestBackend` mock rendering  
**Target Platform**: Linux / POSIX terminal (True-color / 256-color / 16-color ANSI fallbacks)  
**Project Type**: Interactive Terminal User Interface (TUI)  
**Performance Goals**: 20 FPS standard (50ms event tick loop, frame compute budget ≤ 16ms) per design-system §11 and SC-002, scaling to 30 FPS on high-refresh environments for ≥ 50 nodes / 100 edges. Benchmark gate: `benches/simulation_perf.rs` asserts physics tick ≤ 5ms.  
**Constraints**: Minimum 80×24 terminal, WCAG $\ge 4.5:1$ contrast against dark background, $\le 8.0$ Flesch-Kincaid reading level, zero panics in production code  
**Scale/Scope**: 3 curated presets + custom file input, 2D force simulation engine, 3-tier explanation widget, responsive viewport guards  

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-evaluated post-design: ALL GATES PASS.*

| Principle / Gate | Requirement | Compliance Status |
|---|---|---|
| **I. Library-First & Domain Modeling** | Self-contained domain types with owned data across boundaries; clear modular seams. | **PASS**: Clean separation between `physics`, `explanation`, `presets`, and `ui` rendering modules. |
| **II. Strict Lint Compliance** | Workspace `Cargo.toml` lints (`pedantic = deny`, `unsafe_code = deny`, `allow_attributes = deny`). | **PASS**: Zero warnings/denials; explicit `#[expect(..., reason = "...")]` used where necessary. |
| **III. Panic-Free Error Propagation** | `unwrap()`, `expect()`, `panic!()` forbidden in production code; domain errors with `thiserror`. | **PASS**: Fallible layout calculations return `Result` or graceful defaults; no unwraps. |
| **IV. Documentation & Visibility** | `///` docs on all public items; `pub(crate)` for internal items; `#[derive(Debug)]` on public types. | **PASS**: Complete documentation and debug derives across all entities. |
| **V. Test-First (NON-NEGOTIABLE)** | Unit tests, property checks, and regression tests written before implementation. | **PASS**: Comprehensive test plan covering force math, reading-level scoring, preset loading, and UI rendering. |
| **VI. Observability & I/O Discipline** | `tracing` structured logging; TUI handles rendering cleanly without stdout pollution. | **PASS**: Uses existing `LogRing` and structured tracing events. |
| **VII. Dependency & Build Rigor** | Pinned dependencies, semver ranges, MSRV 1.88.0 floor, atomic commits. | **PASS**: MSRV 1.88.0 respected; dependencies unchanged. |
| **VIII. Knowledge Graph Context** | `/graphify` queries leveraged for architectural context and cohesion verification. | **PASS**: Queried graphify knowledge graph for `leiden-tui` component structure and callflow relationships. |

---

## Project Structure

### Documentation (this feature)

```text
specs/004-tui-visual-explanation/
├── spec.md                          # Feature specification & user stories
├── plan.md                          # This implementation plan
├── research.md                      # Phase 0: Technical choices & Flesch-Kincaid mapping
├── data-model.md                    # Phase 1: Entities, force simulation, explanation state
├── quickstart.md                    # Phase 1: Validation scenarios & test guide
└── contracts/
    ├── tui-visual-explanation.md     # Phase 1: UI layout, canvas styling & playback contract
    └── explanation-content.md       # Phase 1: 3-tier explanation payload & readability schema
```

### Source Code (`leiden-tui`)

```text
leiden/crates/leiden-tui/src/
├── app.rs                           # App state, playback controller, preset switching
├── event.rs                         # Crossterm key event handling (Space, n, t, 1-3, ?)
├── lib.rs                           # Crate root & module exports
├── logging.rs                       # Log ring & tracing layer
├── main.rs                          # Entrypoint & CLI dataset argument handling
├── presets.rs                       # Curated demo datasets (Karate Club, Two Cliques, Mess)
├── simulation.rs                    # 2D Force-directed physics relaxation engine
├── explanation.rs                   # 3-tier plain-English explanation engine & reading-level checks
└── ui/
    ├── mod.rs                       # Root layout dispatcher & dimension guard overlay
    ├── colors.rs                    # Tokyo Night palette & COMMUNITY_COLORS tokens (design-system.md §9)
    ├── styles.rs                    # Border styles, title styles, unicode symbols (design-system.md §8)
    ├── explanation_panel.rs         # 3-tier Step Headline, Analogy & Live Stats widget
    ├── graph_canvas.rs              # 2D Canvas force-directed graph painter (nodes & edges)
    ├── community.rs                 # Community partition summary table
    ├── log_pane.rs                  # Log pane widget with FIFO eviction
    └── status_bar.rs                # Playback status bar & key hints
```

**Structure Decision**: Modular extension of `leiden-tui` isolating the 2D physics simulation (`simulation.rs`), explanation content generation (`explanation.rs`), and curated demo presets (`presets.rs`) from the UI rendering layer (`ui/`), strictly following `design-system.md` standards.

---

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| *None* | No constitutional or design system rules violated. | N/A |
