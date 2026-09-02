# Implementation Tasks: TUI Visual Explanation

**Feature**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

---

## Dependencies

```text
Phase 1: Setup (Shared Infrastructure)
       │
       ▼
Phase 2: Foundational (Blocking Prerequisites: Geometry, Guard, Flesch-Kincaid, Simulation Engine)
       │
       ├───────────────────────────────┐
       ▼                               ▼
Phase 3: [US1] Initial State & Presets   Phase 4: [US2] Dynamic Force Clustering & Granularity
(Priority: P1 🎯 MVP)                   (Priority: P1)
       │                               │
       └───────────────┬───────────────┘
                       ▼
Phase 5: [US3] Final Communities & Summary
(Priority: P2)
       │
       ▼
Phase 6: Polish & Cross-Cutting Concerns (Dimension Guard, Help Modal, Benchmarks, Docs)
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, crate dependency configuration, module declarations, design system color tokens, and error handling structures.

- [X] T001 Define `TuiError` domain error enum in `leiden/crates/leiden-tui/src/error.rs`
- [X] T002 [P] Declare new modules (`simulation`, `explanation`, `presets`, `error`) and public re-exports in `leiden/crates/leiden-tui/src/lib.rs`
- [X] T003 [P] Define Tokyo Night palette tokens and 12-color CIELAB `COMMUNITY_COLORS` array in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T004 [P] Define rounded border helpers, title styles, and unicode symbols in `leiden/crates/leiden-tui/src/ui/styles.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core mathematical geometry, screen dimension guard, Flesch-Kincaid readability scoring, and 2D force simulation physics engine that MUST be complete before ANY user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 [P] Write unit tests for `Point2D` vector operations (distance, add_scaled, clamp) and `TerminalDimensionGuard` in `leiden/crates/leiden-tui/tests/test_geometry_and_guard.rs`
- [X] T006 [P] Write unit tests for Flesch-Kincaid readability scoring and word-wrapping in `leiden/crates/leiden-tui/tests/test_readability.rs`
- [X] T007 [P] Write unit tests for `ForceSimulation` relaxation, zero-division avoidance, and screen projection in `leiden/crates/leiden-tui/tests/test_simulation_math.rs`
- [X] T008 [P] Implement `Point2D` spatial geometry struct and vector math operations in `leiden/crates/leiden-tui/src/simulation.rs`
- [X] T009 [P] Implement `TerminalDimensionGuard` (80x24 min dimensions) and dimension check logic in `leiden/crates/leiden-tui/src/ui/mod.rs`
- [X] T010 [P] Implement Flesch-Kincaid grade level scoring calculation and text wrapping utility in `leiden/crates/leiden-tui/src/explanation.rs`
- [X] T011 Implement `ForceSimulation` spring-charge physics relaxation, damping, and `screen_coordinates` mapping in `leiden/crates/leiden-tui/src/simulation.rs`

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 - Understand the Initial State (Priority: P1) 🎯 MVP

**Goal**: Non-technical user launches the TUI (with default demo preset or custom CLI file argument) and sees an unclustered "messy" graph with introductory 8th-grade explanation text, and can select curated demo presets (Karate Club, Two Cliques, Random Mess) with keys `1`, `2`, `3`.

**Independent Test**: Run `cargo run -p leiden-tui` (or with a custom file arg) and verify the initial view renders unclustered monochromatic nodes (`FG_2`), displays the "A Messy Network Starting Point" explanation panel, and switches datasets on pressing `1`, `2`, `3` while auto-pausing.

### Tests for User Story 1

- [X] T012 [P] [US1] Write unit tests for curated presets (Karate Club, Two Cliques, Random Mess) and CLI dataset file loading in `leiden/crates/leiden-tui/tests/test_presets.rs`
- [X] T013 [P] [US1] Write unit tests for initial unclustered `ExplanationState` and unassigned monochromatic node styling in `leiden/crates/leiden-tui/tests/test_initial_state_ui.rs`

### Implementation for User Story 1

- [X] T014 [P] [US1] Implement `PresetId`, `PresetDataset` with built-in Karate Club, Two Cliques, Random Mess graphs, and `from_cli_path` loader in `leiden/crates/leiden-tui/src/presets.rs`
- [X] T015 [P] [US1] Implement `ExplanationState::initial_unclustered` and introductory narrative content in `leiden/crates/leiden-tui/src/explanation.rs`
- [X] T016 [US1] Implement `ExplanationPanel` widget rendering Step Headline, Analogy, and Live Badges in `leiden/crates/leiden-tui/src/ui/explanation_panel.rs`
- [X] T017 [US1] Implement `render_graph_canvas` painting unclustered nodes (`●` in `FG_2`) and edges on Ratatui Canvas in `leiden/crates/leiden-tui/src/ui/graph_canvas.rs`
- [X] T018 [US1] Update `App` and `main.rs` to parse CLI dataset file paths and initialize `AppState` with preset selection in `leiden/crates/leiden-tui/src/app.rs` and `leiden/crates/leiden-tui/src/main.rs`
- [X] T019 [US1] Wire preset switching keys (`1`, `2`, `3`) to reset explanation state and reload graph topology in `leiden/crates/leiden-tui/src/event.rs` and `leiden/crates/leiden-tui/src/app.rs`

**Checkpoint**: User Story 1 is fully functional — non-technical users can launch, inspect unclustered graph states, switch presets, and read plain-English introductory explanations.

---

## Phase 4: User Story 2 - Watch the Graph Transform (Priority: P1)

**Goal**: Non-technical user watches the graph transform via auto-play (`Space`) or manual stepping (`n` / Right Arrow), seeing nodes physically move across the canvas via force simulation to cluster into cohesive communities while adopting `COMMUNITY_COLORS`, with dual granularity (`t` for PhaseLevel vs MicroStep) and updating plain-English explanations.

**Independent Test**: Advance execution in the TUI (via `Space` or `n`) and verify nodes dynamically migrate across the 2D canvas toward assigned community centroids, adopt distinct community colors, intra-community edges colorize while inter-community edges dim, `t` toggles granularity, and the explanation panel dynamically updates headlines and analogies per phase.

### Tests for User Story 2

- [X] T020 [P] [US2] Write unit tests for `PlaybackController` state machine (play/pause, step request, granularity toggle, preset reset policy) in `leiden/crates/leiden-tui/tests/test_playback_controller.rs`
- [X] T021 [P] [US2] Write unit tests for `ExplanationState::from_leiden_event` phase transitions (Local Moving, Refinement, Aggregation) in `leiden/crates/leiden-tui/tests/test_explanation_transitions.rs`
- [X] T022 [P] [US2] Write integration test for dynamic force relaxation and color assignments during algorithmic events in `leiden/crates/leiden-tui/tests/test_clustering_canvas.rs`

### Implementation for User Story 2

- [X] T023 [P] [US2] Implement `GranularityMode` and `PlaybackController` state transitions in `leiden/crates/leiden-tui/src/app.rs`
- [X] T024 [P] [US2] Implement `ExplanationState::from_leiden_event` mapping Leiden events to analogies (grade level $\le 8.0$) in `leiden/crates/leiden-tui/src/explanation.rs`
- [X] T025 [US2] Update `ForceSimulation::tick` to compute community centroids and apply attractive forces to community members in `leiden/crates/leiden-tui/src/simulation.rs`
- [X] T026 [US2] Update `render_graph_canvas` to color nodes and intra-community edges with `COMMUNITY_COLORS` and dim inter-community edges with `FG_3` in `leiden/crates/leiden-tui/src/ui/graph_canvas.rs`
- [X] T027 [US2] Implement `render_status_bar` rendering Play/Pause status, Progress bar, Granularity mode badge, and key hints in `leiden/crates/leiden-tui/src/ui/status_bar.rs`
- [X] T028 [US2] Update event loop and crossterm key handling for `Space` (play/pause), `n`/`Right` (step), and `t` (granularity) in `leiden/crates/leiden-tui/src/event.rs` and `leiden/crates/leiden-tui/src/app.rs`

**Checkpoint**: User Stories 1 AND 2 are functional — users can watch dynamic force-directed clustering and step through the algorithm with live plain-English explanations.

---

## Phase 5: User Story 3 - View the Final Communities (Priority: P2)

**Goal**: User sees the final result where the messy graph has become a set of neat, color-coded communities, along with a completion summary headline, community count, quality metrics, and breakdown table.

**Independent Test**: Allow the algorithm to finish and verify the final screen displays tightly clustered communities, `ACCENT_SUCCESS` completion badge, final narrative summary (*"Neat Communities Discovered!"*), and community partition summary table.

### Tests for User Story 3

- [X] T029 [P] [US3] Write unit tests for `ExplanationState::completed` summary formatting and reading level score in `leiden/crates/leiden-tui/tests/test_completed_summary.rs`
- [X] T030 [P] [US3] Write integration test for completion state rendering in `leiden/crates/leiden-tui/tests/test_completion_ui.rs`

### Implementation for User Story 3

- [X] T031 [P] [US3] Implement `ExplanationState::completed` summary text generator in `leiden/crates/leiden-tui/src/explanation.rs`
- [X] T032 [US3] Update `render_explanation_panel` and `render_status_bar` to display completion status (`ACCENT_SUCCESS` badge and final metrics) in `leiden/crates/leiden-tui/src/ui/explanation_panel.rs` and `leiden/crates/leiden-tui/src/ui/status_bar.rs`
- [X] T033 [US3] Implement `render_community_summary_table` rendering final community membership breakdown table in `leiden/crates/leiden-tui/src/ui/community.rs`

**Checkpoint**: All three user stories are complete and independently verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements and edge cases affecting multiple user stories: minimum dimension guard overlay, help modal overlay (`?`), CPU throttling, benchmark gate, panic/signal cleanup, and strict lints/docs verification.

- [X] T034 [P] Write unit test for undersized terminal overlay rendering and resize restoration in `leiden/crates/leiden-tui/tests/test_dimension_overlay.rs`
- [X] T035 [P] Write unit test for keybinding help overlay toggle (`?`) in `leiden/crates/leiden-tui/tests/test_help_modal.rs`
- [X] T036 Implement centered "TERMINAL TOO SMALL" modal overlay when dimensions $< 80 \times 24$ in `leiden/crates/leiden-tui/src/ui/mod.rs`
- [X] T037 Implement centered $50 \times 14$ keybinding help modal overlay (`?`) in `leiden/crates/leiden-tui/src/ui/mod.rs`
- [X] T038 Implement CPU throttling when paused/idle ($< 0.1\%$ CPU) and signal/panic cleanup handler in `leiden/crates/leiden-tui/src/main.rs`
- [X] T039 [P] Implement Criterion/performance benchmark asserting physics tick $\le 5\text{ms}$ on 50 nodes in `leiden/crates/leiden-tui/benches/simulation_perf.rs`
- [X] T040 Run `quickstart.md` validation, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo doc --workspace --no-deps` verification across `leiden-tui`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phases 3–5)**: All depend on Foundational phase completion.
  - Phase 3 (US1) and Phase 4 (US2) can proceed concurrently once Phase 2 is complete.
  - Phase 5 (US3) depends on US1 & US2 components.
- **Polish (Phase 6)**: Depends on User Story phases being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — delivers unclustered initial state and preset selection (MVP).
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) — delivers dynamic force-directed clustering and dual granularity playback.
- **User Story 3 (P2)**: Integrates with US1 and US2 to display final completion summaries and partition metrics.

### Within Each User Story

1. Failing unit/integration tests written FIRST per Constitution Principle V (TDD).
2. Domain entities and data structures implemented.
3. Rendering widgets and UI components implemented.
4. App state machine, event routing, and keybindings wired together.
5. Story verified against its Independent Test criteria before advancing.

### Parallel Opportunities

- **Phase 1**: T002, T003, and T004 can execute in parallel.
- **Phase 2**: T005, T006, T007 (tests) and T008, T009, T010 (implementations) can execute in parallel.
- **Phase 3 (US1)**: T012 and T013 (tests), plus T014 and T015 (data/content) can execute in parallel.
- **Phase 4 (US2)**: T020, T021, and T022 (tests), plus T023 and T024 (controllers/mappers) can execute in parallel.
- **Phase 5 (US3)**: T029 and T030 (tests), plus T031 can execute in parallel.
- **Phase 6 (Polish)**: T034, T035, and T039 can execute in parallel.

---

## Parallel Execution Examples

### User Story 1 (Initial State & Presets)

```bash
# Launch test creation in parallel:
Task: "Write unit tests for curated presets (Karate Club, Two Cliques, Random Mess) and CLI dataset file loading in leiden/crates/leiden-tui/tests/test_presets.rs"
Task: "Write unit tests for initial unclustered ExplanationState and unassigned monochromatic node styling in leiden/crates/leiden-tui/tests/test_initial_state_ui.rs"

# Launch data and content generation in parallel:
Task: "Implement PresetId, PresetDataset with built-in Karate Club, Two Cliques, Random Mess graphs, and from_cli_path loader in leiden/crates/leiden-tui/src/presets.rs"
Task: "Implement ExplanationState::initial_unclustered and introductory narrative content in leiden/crates/leiden-tui/src/explanation.rs"
```

### User Story 2 (Watch Graph Transform)

```bash
# Launch test creation in parallel:
Task: "Write unit tests for PlaybackController state machine in leiden/crates/leiden-tui/tests/test_playback_controller.rs"
Task: "Write unit tests for ExplanationState::from_leiden_event phase transitions in leiden/crates/leiden-tui/tests/test_explanation_transitions.rs"
Task: "Write integration test for dynamic force relaxation and color assignments in leiden/crates/leiden-tui/tests/test_clustering_canvas.rs"

# Launch controller and narrative implementation in parallel:
Task: "Implement GranularityMode and PlaybackController state transitions in leiden/crates/leiden-tui/src/app.rs"
Task: "Implement ExplanationState::from_leiden_event mapping Leiden events to analogies in leiden/crates/leiden-tui/src/explanation.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (`T001`–`T004`).
2. Complete Phase 2: Foundational Prerequisites (`T005`–`T011`).
3. Complete Phase 3: User Story 1 (`T012`–`T019`).
4. **STOP and VALIDATE**: Launch `cargo run -p leiden-tui` and verify initial messy graph display, preset switching (`1`, `2`, `3`), and plain-English introductory explanation.

### Incremental Delivery

1. **Increment 1 (MVP)**: Setup + Foundation + User Story 1 $\to$ Visual unclustered network with preset picker and introductory explanation.
2. **Increment 2**: Add User Story 2 $\to$ Dynamic 2D force-directed animation, `COMMUNITY_COLORS`, `Space` play/pause, `n` stepping, `t` granularity toggle, and live phase analogies.
3. **Increment 3**: Add User Story 3 $\to$ Final completion summary panel, `ACCENT_SUCCESS` badge, and community partition summary table.
4. **Increment 4 (Polish)**: Add Phase 6 $\to$ Terminal $< 80 \times 24$ resize guard overlay, `?` keybinding modal, CPU throttling, and performance benchmark.

---

## Notes

- `[P]` tasks = separate files, zero mutual dependencies.
- `[Story]` labels (`[US1]`, `[US2]`, `[US3]`) map strictly to user stories from `spec.md`.
- All code and tests strictly adhere to the Leiden Algorithm Constitution v1.1.0 (zero `unwrap()`/`expect()` in non-test code, TDD test-first, `missing_docs = deny`, `pedantic = deny`).
