# Implementation Plan: TUI Algorithm Stepping

**Branch**: `003-tui-algorithm-stepping` | **Date**: 2026-09-01 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-tui-algorithm-stepping/spec.md`

## Summary

Implement interactive visual stepping for the Leiden algorithm within the TUI. This requires extending the core algorithm orchestrator with thread synchronization primitives (`Arc<AtomicBool>`) to support pausing, stepping, and aborting. It also involves upgrading the `LeidenEvent` to emit the intermediate `Partition` state so that `ratatui` can render a spatial, community-clustered canvas visualization of the graph mid-run.

## Technical Context

**Language/Version**: Rust 1.88.0 (pinned per constitution)

**Primary Dependencies**: `ratatui` 0.30.2, `crossterm` 0.29.0

**Storage**: N/A

**Testing**: `cargo test`

**Target Platform**: Terminal (Linux, macOS)

**Project Type**: CLI / TUI application + Library

**Performance Goals**: < 10% overhead on large graphs for state cloning

**Constraints**: Terminal must support 80x24 layout and basic Unicode blocks. Adhere strictly to the design system (Data Observatory pattern).

**Scale/Scope**: Supports typical graph sizes evaluated by Leiden (thousands of nodes).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Tracing I/O Discipline:** `println!` is forbidden. We will use `tracing::info!` via the existing `LeidenEvent` system.
- **Determinism:** The stepping logic must not alter the determinism of the underlying Leiden algorithm.
- **TDD / Micro-verification:** Tests will be written for the `ControlFlags` synchronization behavior.
- **MSRV:** Code must compile on Rust 1.88.0.
- **No Unsafe Code:** The `ControlFlags` use standard safe atomics.

## Project Structure

### Documentation (this feature)

```text
specs/003-tui-algorithm-stepping/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
leiden/crates/
├── leiden/                  # Core algorithm library
│   └── src/
│       ├── events.rs        # Add Partition to LeidenEvent
│       └── orchestrator/    # Add ControlFlags wait loop
└── leiden-tui/              # Terminal UI binary
    └── src/
        ├── app.rs           # Add step/pause/abort key bindings
        ├── worker.rs        # Pass ControlFlags to Leiden orchestrator
        └── ui/
            └── graph.rs     # Implement community-clustered Canvas layout
```

**Structure Decision**: The project is a split library (`leiden`) and binary (`leiden-tui`). We will modify the boundary interfaces (`orchestrator` and `events.rs`) in the library, and implement the interactive keybindings and rendering in the TUI binary.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
