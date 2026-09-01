# Implementation Tasks: TUI Algorithm Stepping

**Feature**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

## Dependencies
Foundational Components (Leiden Core Sync) -> US1 (Interactive Stepping TUI integration) -> US2 (Canvas Rendering) -> Polish & Edge Cases

## Phase 1: Setup & Infrastructure
**Goal**: Verify workspace builds and passes existing tests.
**Independent Test**: `cargo test --workspace` succeeds without errors.

- [X] T001 Verify workspace builds and passes existing tests in Cargo.toml

## Phase 2: Foundational Components
**Goal**: Implement `ControlFlags` and intermediate state emission in the core `leiden` orchestrator. Blocks all user stories.
**Independent Test**: `cargo test -p leiden` passes with new orchestrator control tests.

- [X] T002 [P] Write failing test for LeidenEvent carrying partition state in leiden/crates/leiden/src/events.rs
- [X] T003 [P] Write failing test for orchestrator pausing and aborting via ControlFlags in leiden/crates/leiden/src/orchestrator/mod.rs
- [X] T004 Update LeidenEvent to include `partition: Option<Partition>` state in leiden/crates/leiden/src/events.rs
- [X] T005 Add ControlFlags struct and `with_control_flags` builder method to leiden/crates/leiden/src/orchestrator/mod.rs
- [X] T006 Implement wait loop checking paused, step, and abort flags during `run_outer_loop` in leiden/crates/leiden/src/orchestrator/mod.rs

## Phase 3: [US1] Interactive Stepping
**Goal**: Wire the TUI `app.rs` keys (`p`, `s`) to the worker thread's `ControlFlags` to allow manual step control.
**Independent Test**: Running the TUI allows pausing with `p` and executing exactly one iteration with `s`.

### Tests for [US1]
- [ ] T007 [P] [US1] Write failing tests for App key bindings (`s` and `p`) and state transitions in leiden/crates/leiden-tui/src/app.rs

### Implementation for [US1]
- [ ] T008 [US1] Update `App` and `ControlState` to include `step` and `abort` Atomics in leiden/crates/leiden-tui/src/app.rs
- [ ] T009 [US1] Handle `s` keypress to trigger `step` atomic and switch to paused mode if running in leiden/crates/leiden-tui/src/app.rs
- [ ] T010 [US1] Update `spawn_leiden_worker` to initialize and pass `ControlFlags` to the orchestrator in leiden/crates/leiden-tui/src/worker.rs
- [ ] T011 [US1] Update `App::push` to extract the emitted partition state from `IterationFinished` in leiden/crates/leiden-tui/src/app.rs

## Phase 4: [US2] Visual Observation
**Goal**: Upgrade the Graph Topology panel to render nodes in a spatial canvas layout clustered by their community.
**Independent Test**: The Graph view visually updates node block positions automatically as they change communities.

### Tests for [US2]
- [ ] T012 [P] [US2] Write unit tests for CommunityGrid spatial coordinate calculation in leiden/crates/leiden-tui/src/ui/graph.rs

### Implementation for [US2]
- [ ] T013 [US2] Implement `CommunityGrid` structure to track bounding boxes and spatial positions in leiden/crates/leiden-tui/src/ui/graph.rs
- [ ] T014 [US2] Upgrade `render_graph_panel` to use `ratatui::widgets::canvas::Canvas` for community-clustered spatial nodes in leiden/crates/leiden-tui/src/ui/graph.rs

## Phase 5: Polish & Cross-Cutting Concerns
**Goal**: Finalize edge cases, such as gracefully aborting when quitting, and performance validation.

- [ ] T015 Handle `q` (quit) keypress to trigger abort signal gracefully in leiden/crates/leiden-tui/src/app.rs
- [ ] T016 Run quickstart.md validation manually to ensure <10% overhead on large graphs via leiden/crates/leiden-tui/Cargo.toml

## Parallel Execution Examples

```bash
# Launch Foundational Test TDD writing in parallel:
Task: "Write failing test for LeidenEvent carrying partition state in leiden/crates/leiden/src/events.rs"
Task: "Write failing test for orchestrator pausing and aborting via ControlFlags in leiden/crates/leiden/src/orchestrator/mod.rs"

# In [US1] and [US2], tests can be written independently before their implementations:
Task: "Write failing tests for App key bindings (s and p) and state transitions in leiden/crates/leiden-tui/src/app.rs"
Task: "Write unit tests for CommunityGrid spatial coordinate calculation in leiden/crates/leiden-tui/src/ui/graph.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Setup and Foundational tasks (Core library updates for `ControlFlags`).
2. Complete Phase 3 [US1] to allow step-by-step halting and continuing. 
3. Validate by observing the text output of the graph view update one step at a time.

### Incremental Delivery
1. Deploy MVP [US1] (The TUI now effectively pauses and resumes).
2. Complete Phase 4 [US2] to replace the text output with the spatial `ratatui` canvas.
3. Validate continuous layout updates.
4. Complete Phase 5 (Polish) to ensure graceful shutdown on quit while paused.
