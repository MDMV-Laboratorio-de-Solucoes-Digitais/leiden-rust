# Implementation Plan: TUI Design System

**Branch**: `[002-tui-design-system]` | **Date**: 2026-09-01 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-tui-design-system/spec.md`

## Summary

Implement a comprehensive design system for `leiden-tui` as two Rust modules (`src/ui/colors.rs` and `src/ui/styles.rs`) within the existing `leiden-tui` crate. The design system replaces the current placeholder color constants with a full Tokyo Night–inspired dark palette (5 background tones, 4 text tones, 5 semantic accents, 12 community-hash colors), `const fn` style presets for all recurring visual patterns, state-driven theming mapping each `AppState` variant to a unique color + symbol + label triple, a responsive layout engine with four terminal-width breakpoints, a focus/border system using `BorderType::Rounded`, log severity styling, progress visualization helpers, and a 16-color ANSI fallback strategy with true-color detection. All constants and presets are defined as `pub const` or `pub const fn` items with `///` doc comments, satisfying the strict lint profile (`missing_docs = deny`, `const fn` preference).

The feature builds on the existing `design-system.md` reference document at the repository root, which contains the canonical color values, layout specifications, widget styling tables, and implementation-ready Rust code snippets. The existing `colors.rs` (32 lines, 8-color placeholder palette) will be replaced with the full design system; a new `styles.rs` module will be added for style presets and helper functions that don't fit in the color module.

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV 1.88.0. `unsafe_code = deny` across the workspace. Pinned via `rust-toolchain.toml`.

**Primary Dependencies**:
- **`ratatui = "0.30.2"`** — TUI framework providing `Color::Rgb`, `Style`, `Modifier`, `BorderType::Rounded`, `Gauge`, `Sparkline`, `Block`, `Borders`. Pinned per Constitution §VII and `001-leiden-algorithm` plan.
- **`crossterm = "0.29.0"`** — terminal backend for Ratatui. No direct interaction from the design system modules (Ratatui abstracts the backend).
- **`thiserror`** — not directly consumed by the design system modules (no fallible operations in color/style constants), but available in the crate for error types.
- **`tracing`** — the design system does not emit logs, but the log severity styling references tracing level semantics.

**Storage**: N/A. The design system is purely in-memory constants and pure functions. No I/O, no persistent state.

**Testing**: `cargo test --workspace` for unit tests. Design system tests verify:
- `const fn` style presets compile and produce expected `Style` values.
- `community_color()` returns deterministic colors and wraps at index 12.
- `supports_truecolor()` returns correct results for known `COLORTERM`/`TERM` values.
- `state_color()`, `state_indicator()`, `state_label()` cover all `AppState` variants.
- `layout_mode()` returns correct `LayoutMode` for all breakpoints.
- Contrast ratio validation (computed against documented pairs from `design-system.md §2.3`).
- `ratatui::backend::TestBackend` snapshot tests for widget rendering with the new style presets.

**Target Platform**: Linux/macOS/Windows. The design system is platform-independent (pure color/style constants). True-color detection uses `std::env::var()` for `COLORTERM` and `TERM`, which works on all platforms.

**Project Type**: Module within the `leiden-tui` binary crate. Two files: `src/ui/colors.rs` (color constants, community hash, true-color detection, fallback palette) and `src/ui/styles.rs` (style presets, state theming, layout mode, border/focus helpers).

**Performance Goals**:
- SC-007: Render loop completes each frame in under 50 ms. The design system contributes zero runtime overhead — all style presets are `const fn` resolved at compile time.
- `community_color()` is `const fn` with O(1) modular arithmetic.
- `supports_truecolor()` is called once at startup, not per frame.

**Constraints**:
- `unsafe_code = deny` workspace-wide.
- `missing_docs = deny` — every `pub` color constant, style preset, and helper function carries `///` docs (Constitution §IV).
- All style presets must be `const fn` to satisfy the strict lint profile (Constitution §II). Non-`const fn` helpers (e.g., `supports_truecolor()`, `focused_block()`) are acceptable only when they require runtime evaluation or borrow non-`'static` data.
- `Modifier::ITALIC` is forbidden — use `Modifier::DIM` or `Modifier::UNDERLINED` instead (FR-016).
- `Modifier::REVERSED` is forbidden for selection — use explicit `BG_3` + `FG_0` (FR-015).
- No emoji characters — only Unicode box-drawing, symbol, and Greek characters (SC-009).
- WCAG AA contrast ratio ≥ 4.5:1 for all documented foreground-background pairs (FR-012).

**Scale/Scope**: 26 color constants, ≥9 style presets, 12 community colors, 4 layout modes, 4 state themes, 5 log severity styles, and a true-color detection function. Approximately 300–400 lines of Rust across the two modules.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Pre-design (before Phase 0)**: see columns below.

**Post-design (after Phase 1)**: see "Post-Phase-1" column. All gates continue to pass; the design introduced no new principle violations.

| Principle | Pre-design | Post-Phase-1 | Notes |
|---|---|---|---|
| I. Library-First & Domain Modeling | PASS | PASS | Design system modules live inside `leiden-tui` crate's `src/ui/` directory. They define pure data constants and `const fn` presets consumed by the TUI widgets. No algorithm logic in the design system; the `AppState` type is imported from `app.rs`, not redefined. |
| II. Strict Lint Compliance | PASS | PASS | All style presets are `const fn`. No `allow` attributes without `#[expect(..., reason = "...")]`. The full `[workspace.lints]` block from `rust-code-rigor.md` applies unmodified. |
| III. Panic-Free Error Propagation | PASS | PASS | No fallible operations in color/style constants. `supports_truecolor()` uses `std::env::var()` which returns `Result` — handled with `if let Ok(...)` pattern, never `unwrap()`. `community_color()` uses modular arithmetic, no division by zero possible (array length is const 12). |
| IV. Documentation & Visibility Discipline | PASS | PASS | Every `pub const`, `pub const fn`, and `pub fn` carries `///` doc comments per the reference implementation in `design-system.md §9`. Module-level `//!` docs explain the color layer taxonomy. `pub(crate)` used for internal helpers not exported from the crate. |
| V. Test-First (NON-NEGOTIABLE) | PASS | PASS | `quickstart.md` defines the test scenarios that must be written before each implementation commit: community color determinism, contrast ratio verification, true-color detection, state theme completeness, layout breakpoint correctness, and snapshot tests. |
| VI. Observability & I/O Discipline | PASS | PASS | The design system modules do not emit logs or perform I/O. The log severity styling maps (`ACCENT_ERROR` for `[ERROR]`, etc.) reference `tracing` semantics but do not invoke `tracing` macros. |
| VII. Dependency & Build Rigor | PASS | PASS | No new dependencies introduced. The design system uses only `ratatui::style::{Color, Modifier, Style}` and `ratatui::widgets::{BorderType}` — already in the dependency tree via `ratatui = "0.30.2"`. `std::env::var` from the standard library for true-color detection. |
| Additional: Unsafe Code | PASS | PASS | No `unsafe` code. All operations are safe arithmetic, `const fn` style construction, and environment variable reads. |

No unjustified violations. Both pre-design and post-Phase-1 gates pass.

## Project Structure

### Documentation (this feature)

```text
specs/002-tui-design-system/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
│   └── design-system-api.md  # Public API contract for the design system modules
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)

```text
leiden/                              # Workspace root
├── Cargo.toml                       # [workspace] + [workspace.lints]
├── crates/
│   └── leiden-tui/                  # Interactive Ratatui binary
│       ├── Cargo.toml               # ratatui = "0.30.2", crossterm = "0.29.0"
│       ├── src/
│       │   ├── lib.rs               # Re-exports
│       │   ├── main.rs              # Entry point
│       │   ├── app.rs               # AppState, App, FocusPanel, PanelVisibility
│       │   ├── event.rs             # Key mapping
│       │   ├── logging.rs           # LogRing, LogPaneLayer
│       │   ├── worker.rs            # Background worker thread
│       │   └── ui/
│       │       ├── mod.rs           # render() dispatcher + layout engine (MODIFIED)
│       │       ├── colors.rs        # ← REPLACED: Full design system color palette
│       │       ├── styles.rs        # ← NEW: Style presets, state theming, border helpers
│       │       ├── community.rs     # Community panel widget (UPDATED to use design system)
│       │       ├── graph.rs         # Graph view widget (UPDATED to use design system)
│       │       ├── log_pane.rs      # Log pane widget (UPDATED to use design system)
│       │       └── status_bar.rs    # Status bar widget (UPDATED to use design system)
│       └── tests/                   # Snapshot and integration tests
└── design-system.md                 # Reference design document (READ-ONLY, not modified)
```

**Structure Decision**: The design system is implemented as two modules within the existing `leiden-tui` crate's `src/ui/` directory. `colors.rs` is expanded from its current 32-line placeholder to the full palette (~120 lines). `styles.rs` is a new module (~180 lines) for style presets, state theming helpers, layout mode detection, and border/focus constructors. This is the simplest structure that satisfies the spec's FR-001 and FR-002 while keeping all visual constants in one importable location. No separate crate is needed because the design system has exactly one consumer (`leiden-tui`).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| (none) | — | — |

No constitution violations. The design system is pure data + `const fn` functions, the simplest possible form factor for a Rust module.
