---

description: "Task list for TUI Design System implementation"
---

# Tasks: TUI Design System

**Input**: Design documents from `/specs/002-tui-design-system/`
**Branch**: `[002-tui-design-system]`
**Source crate**: `leiden/crates/leiden-tui/`

**Prerequisites**: plan.md (tech stack, constraints, structure), spec.md (7 user stories P1–P3), data-model.md (color constants, style presets, state theming, layout), contracts/design-system-api.md (API surface), research.md (const fn / fallback decisions), quickstart.md (11 runnable validation scenarios), constitution.md (§V TDD non-negotiable).

**Tests**: Tests ARE required. Constitution §V mandates test-first (red-green-refactor); plan.md §Testing lists verification targets; quickstart.md defines 11 concrete scenarios. Tests must be written and observed failing before implementation commits.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1–US7)
- Setup phase: no story label. Foundational phase: no story label. User story phases: MUST label `[US1]`...`[US7]`.
- Each implementation task below includes an exact file path.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify toolchain, lint profile, and module wiring before any design system code is written.

- [X] T001 Verify `rust-toolchain.toml` pins stable toolchain, edition 2024, MSRV floor 1.88.0 (Constitution §VII, plan.md §Constraints)
- [X] T002 [P] Verify `[workspace.lints]` in workspace `Cargo.toml` enforces `unsafe_code = deny`, `missing_docs = deny`, `missing_debug_implementations`, `panic`/`unwrap_used`/`expect_used` denied, `clippy::pedantic = deny` (Constitution §II, guide-to-strict-rust.md)
- [X] T003 Create `pub mod colors;` and `pub mod styles;` declarations in `leiden/crates/leiden-tui/src/ui/mod.rs`

**Checkpoint**: Toolchain and lint gate ready; module declarations in place.

---

## Phase 2: Foundational (Blocking Prerequisites — Color Constants)

**Purpose**: Implement the `colors.rs` data layer — all color constants, ANSI fallbacks, and pure helper functions. **No user story may begin until this phase is complete.**

**⚠️ CRITICAL**: All widget and style-preset tasks depend on these constants.

- [X] T004 [P] Write failing tests: all 26 color constants compile as `const` items (`const _: Color = BG_0;` pattern from quickstart §2) in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T005 Implement color constants in `leiden/crates/leiden-tui/src/ui/colors.rs`: `BG_0`–`BG_4`, `FG_0`–`FG_3`, `ACCENT_*` (5), `COMMUNITY_COLORS: [Color; 12]` — all `pub const` with `///` docs (FR-001, data-model §1, design-system.md §9)
- [X] T006 Write failing test: ANSI fallback completeness — every `*_ANSI` constant compiles and `COMMUNITY_COLORS_ANSI.len() == 6` (quickstart §11) in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T007 Add ANSI fallback constants to `colors.rs`: `BG_*_ANSI`, `FG_*_ANSI`, `ACCENT_*_ANSI`, `COMMUNITY_COLORS_ANSI: [Color; 6]` (FR-014, contracts §1.2, design-system.md §10.1)
- [X] T008 Write failing tests: `community_color()` determinism + wrap at 12, `supports_truecolor()` detection for COLORTERM/TERM/conservative default (quickstart §4, §5) in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T009 Implement `community_color(community_id: u32) -> Color` and `supports_truecolor() -> bool` in `leiden/crates/leiden-tui/src/ui/colors.rs` (FR-007, FR-013, data-model §1.5)
- [X] T010 [P] Write contrast ratio validation test computing WCAG ratios for all documented pairs (data-model §6, design-system.md §2.3) in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T046 [P] Write failing test: `resolve_color()` selects RGB when `supports_truecolor()` returns true, ANSI fallback when false; verifies all 15 color pairs have a corresponding `_ANSI` constant (data-model §2.1, contracts §1.4) in `leiden/crates/leiden-tui/src/ui/colors.rs`
- [X] T047 Implement `resolve_color(color: Color, ansi: Color) -> Color` in `leiden/crates/leiden-tui/src/ui/colors.rs`, gating on `supports_truecolor()` (FR-013/FR-014, B1)

**Checkpoint**: `colors.rs` complete with all constants, ANSI fallbacks, `resolve_color()`, and passing unit tests.

---

## Phase 3: User Story 1 — Visually Distinguishable Application States (Priority: P1) 🎯 MVP

**Goal**: Map each `AppState` variant (`Idle`, `Running`, `Done`, `Error`) to a unique color + symbol + label triple so state is distinguishable in under 1 second.

**Independent Test**: Launch `leiden-tui`, transition through `Idle → Running → Done`, `Running → Error`, `Error → Idle`, and verify each state uses a distinct color + symbol combination in the status bar.

- [X] T011 [US1] Write failing tests: `state_theme_covers_all_variants` (all 4 states return non-empty color/symbol/label) + `state_indicators_are_unique` (quickstart §6, contracts §2.2)
- [X] T012 [US1] Implement `state_color()`, `state_indicator()`, `state_label()` + `INDICATOR_*` symbol constants in `leiden/crates/leiden-tui/src/ui/styles.rs` (FR-003, data-model §§4.3, 5)
- [X] T013 [US1] Update `leiden/crates/leiden-tui/src/ui/status_bar.rs` to render state indicator symbol + label using state theme functions, with progress gauge placeholder for Running
- [X] T014 [US1] Write snapshot test: status bar renders distinct `○`/`●`/`✓`/`✗` + label per state using `TestBackend` (quickstart §10)

**Checkpoint**: All 4 `AppState` variants produce a unique color + symbol + label. Error → Idle preserves log ring.

---

## Phase 3.5: Help Overlay (FR-017)

**Goal**: Render a keyboard shortcut help overlay triggered by `?`, with key bindings organized into labeled groups ("Navigation", "Panels", "General"), `BG_1` background, `ACCENT_PRIMARY` rounded border. Dismissable by any key press. Does not dismiss underlying panels.

**Independent Test**: Press `?`, verify overlay renders over all panels with correct styling; press any key, verify overlay closes and panels are preserved.

- [X] T048 Write failing test: `help_overlay()` returns a `Block` with `BG_1` background and `ACCENT_PRIMARY` `BorderType::Rounded` border; key bindings are grouped into labeled categories (quickstart §10, FR-017, design-system.md §5.5)
- [X] T049 Implement `help_overlay()` widget in `leiden/crates/leiden-tui/src/ui/mod.rs` (or new `src/ui/help.rs`) with grouped key bindings table, `BG_1` background, `ACCENT_PRIMARY` border (FR-017, data-model §8)
- [X] T050 Write snapshot test: help overlay renders over all panels with `?`, dismisses on any key, preserves panel state (TestBackend, SC-001, SC-009)

**Checkpoint**: Help overlay renders with correct styling, all key bindings visible in labeled groups, dismissable by any key, no emoji in output.

---

## Phase 4: User Story 2 — Community Identification Across Panels (Priority: P1)

**Goal**: Each community gets a consistent, visually distinct color across the community panel and graph view via `COMMUNITY_COLORS[id % COMMUNITY_COLORS.len()]`. Selection uses explicit `BG_3` + `FG_0`, not `Modifier::REVERSED`.

**Independent Test**: Load a graph with ≥ 12 communities, verify that community `N`'s color block in the community panel matches the node circle color in the graph view, and confirm all 12 colors are distinguishable.

- [X] T015 [US2] Write failing tests: `selected_row_style()` uses `BG_3` + `FG_0` + `BOLD` (not `REVERSED`); `no_italic_in_style_presets` covers all presets (quickstart §8, FR-015)
- [X] T016 [US2] Implement `header_style()`, `selected_row_style()`, `normal_row_style()` as `const fn` in `leiden/crates/leiden-tui/src/ui/styles.rs` (FR-015, data-model §3.2)
- [X] T017 [US2] Update `leiden/crates/leiden-tui/src/ui/community.rs` to use `community_color()` for color blocks and `selected_row_style()` for row selection (US-2, design-system.md §5.1)
- [X] T018 [US2] Update `leiden/crates/leiden-tui/src/ui/graph.rs` to use `community_color()` for node circle colors (US-2, design-system.md §5.2)
- [X] T019 [P] [US2] Write snapshot test: verifying community color block matches graph node color for the same community ID (TestBackend)

**Checkpoint**: Cross-panel color parity confirmed; color block preserved on row selection; no `Modifier::REVERSED`.

---

## Phase 5: User Story 3 — Responsive Multi-Panel Layout (Priority: P2)

**Goal**: Four terminal-width breakpoints (`Full` ≥120, `Compact` 80–119, `Stacked` 60–79, `Minimal` <60) with exhaustive, non-overlapping coverage. Panel toggles (`g`/`l`) redistribute space per the toggle matrix. Community panel is permanent.

**Independent Test**: Resize terminal to each breakpoint (≥120, 80–119, 60–79, <60 columns) and verify panels rearrange with no overflow or content hidden behind the status bar.

- [X] T020 [US3] Write failing test: `layout_mode()` returns correct `LayoutMode` at all breakpoints and interior values (quickstart §7, data-model §4.2)
- [X] T021 [US3] Implement `LayoutMode` enum + `layout_mode(width: u16) -> LayoutMode` as `const fn` in `leiden/crates/leiden-tui/src/ui/styles.rs` (FR-004, data-model §4)
- [X] T022 [US3] Implement responsive layout engine in `leiden/crates/leiden-tui/src/ui/mod.rs` dispatching on `LayoutMode`, plus panel toggle redistribution per the toggle matrix (FR-004, FR-005, design-system.md §§3.3–3.5)
- [X] T023 [US3] Write snapshot tests: all 4 breakpoints + all `g`/`l` toggle combinations render correctly (TestBackend, SC-003)

**Checkpoint**: All 4 breakpoints produce correct layout; zero unhandled column widths; toggles redistribute per documented matrix.

---

## Phase 6: User Story 4 — Log Event Severity Differentiation (Priority: P2)

**Goal**: Log events color-coded by severity: `ERROR` = `ACCENT_ERROR` bold, `WARN` = `ACCENT_WARNING` bold, `INFO` = `ACCENT_INFO` normal, `DEBUG` = `FG_2` normal, `TRACE` = `FG_3` dim.

**Independent Test**: Feed the log pane a mix of `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` events and verify each level uses its designated color and style.

- [X] T024 [US4] Write failing tests: each `log_*_style()` returns correct color + modifier, no `ITALIC` (quickstart §8, FR-008)
- [X] T025 [US4] Implement `log_error_style()` through `log_trace_style()` as `const fn` in `leiden/crates/leiden-tui/src/ui/styles.rs` (FR-008, data-model §4.4)
- [X] T026 [US4] Update `leiden/crates/leiden-tui/src/ui/log_pane.rs` to apply severity styles to `[LEVEL]` prefixes and implement 500-entry FIFO ring buffer (US-4, design-system.md §5.3)
- [X] T027 [US4] Write snapshot test: log pane renders `[ERROR]` = bold red, `[WARN]` = bold amber, `[INFO]` = teal, `[DEBUG]` = FG_2, `[TRACE]` = FG_3 dim (TestBackend)

**Checkpoint**: All 5 severity levels use designated color + modifier; FIFO eviction non-blocking.

---

## Phase 7: User Story 5 — Keyboard-Only Navigation with Focus Indication (Priority: P2)

**Goal**: `Tab` cycles focus across visible panels; focused panel border = `ACCENT_PRIMARY`, unfocused = `FG_3`; hidden panels are skipped; single visible panel makes `Tab` a no-op.

**Independent Test**: Press `Tab` to cycle focus, verify focused panel border changes from `FG_3` to `ACCENT_PRIMARY`, and that hidden panels are skipped in the cycle.

- [X] T028 [US5] Write failing tests: `focused_border_style()` = `ACCENT_PRIMARY`, `unfocused_border_style()` = `FG_3`, `key_hint_style()` = `FG_3` + `DIM` (quickstart §8, FR-006)
- [X] T029 [US5] Implement `focused_border_style()`, `unfocused_border_style()`, `title_style_focused()`, `title_style_unfocused()` as `const fn` in `leiden/crates/leiden-tui/src/ui/styles.rs` (FR-006, data-model §3.1)
- [X] T030 [P] [US5] Implement `key_hint_style()`, `key_letter_style()` as `const fn` in `leiden/crates/leiden-tui/src/ui/styles.rs` (data-model §3.3)
- [X] T031 [US5] Implement `focused_block()`, `unfocused_block()`, `panel_block()` in `leiden/crates/leiden-tui/src/ui/styles.rs` using `BorderType::Rounded` (FR-010, data-model §4.5)
- [X] T032 [US5] Wire `Tab` focus cycle with hidden-panel skipping and single-panel no-op in `leiden/crates/leiden-tui/src/event.rs` + `leiden/crates/leiden-tui/src/ui/mod.rs` (FR-006, US-5)
- [X] T033 [US5] Write snapshot test: `Tab` changes border color to `ACCENT_PRIMARY`; hidden panel skipped; `Tab` no-op when single panel visible (TestBackend)

**Checkpoint**: Tab cycles visible panels only; borders change on focus; single-panel no-op confirmed.

---

## Phase 8: User Story 6 — Accessible Color Contrast and Fallback (Priority: P3)

**Goal**: All fg/bg pairs ≥ 4.5:1 WCAG AA. Graceful degradation to 16-color ANSI when true-color is unsupported. Symbols reinforce color for color-vision deficiency.

**Independent Test**: Set `COLORTERM=` (unset) and `TERM=xterm` to force 16-color mode, launch the TUI, and verify all text remains legible and state indicators are distinguishable.

- [X] T034 [US6] Run contrast ratio validation for all documented pairs — `FG_0`/`FG_1`/`FG_2`/`ACCENT_*` on `BG_0`/`BG_3` (data-model §6, design-system.md §2.3, FR-012, SC-004)
- [X] T035 [US6] Run `supports_truecolor()` + ANSI fallback completeness tests in 16-color mode (quickstart §5, §11)
- [X] T036 [US6] Write integration test: forcing `COLORTERM` unset + `TERM=xterm`, verify state indicators (`○`/`●`/`✓`/`✗`) remain distinguishable by symbol + color at 16-color depth (US-6 scenario 3, SC-005)

**Checkpoint**: All pairs ≥ 4.5:1; 16-color fallback renders; symbols reinforce color for CVD.

---

## Phase 9: User Story 7 — Progress Visualization During Execution (Priority: P3)

**Goal**: Progress gauge shows `iteration / iteration_cap` proportion; quality sparkline displays a fixed window of the 20 most recent iterations with auto-scaling; ΔQ signed and colored (`ACCENT_SUCCESS` for ΔQ > 0, `ACCENT_ERROR` for ΔQ < 0).

**Independent Test**: Run the algorithm on a fixture, verify the progress gauge fills proportionally to `iteration / iteration_cap`, and the sparkline's bar heights reflect the latest 20 convergence values.

- [X] T037 [US7] Write failing snapshot test: Running state status bar shows progress gauge `5/10` + quality value `Q=0.4231` (quickstart §10, FR-009)
- [X] T038 [US7] Implement progress gauge rendering in `leiden/crates/leiden-tui/src/ui/status_bar.rs` using `BG_4` (empty) + `ACCENT_INFO` (fill) with ratio `iteration / iteration_cap` (FR-009, design-system.md §7.1)
- [X] T039 [US7] Implement quality sparkline with 20-iteration fixed window + auto-scaling to observed range in `leiden/crates/leiden-tui/src/ui/status_bar.rs` (US-7, design-system.md §7.2)
- [X] T040 [US7] Write test: sparkline shows only 20 most recent iterations (oldest scroll off left); ΔQ < 0 renders in `ACCENT_ERROR` with negative sign; γ to 2 decimal places (US-7 scenarios 2–4, FR-011)
- [X] T051 Write test: all FR-011 number formats verified — modularity `Q=0.4231` (4dp), `ΔQ=+0.0033` (4dp signed), node/edge counts with thousands separators (e.g. `12,345`), iteration `current/cap`, `γ=1.50` (2dp) (FR-011, SC-008, design-system.md §4.2)

**Checkpoint**: Gauge fills proportionally; sparkline fixed-window 20; ΔQ signed and color-coded; γ 2dp.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Verification that the full design system passes lint, tests, and validation scenarios.

- [X] T041 [P] Run lint compliance gate: `cargo clippy -p leiden-tui --all-targets -- -D warnings` + `cargo doc -p leiden-tui --no-deps` (fails on missing_docs) (Constitution §II, §IV, quickstart §9)
- [X] T042 Run full workspace test suite: `cargo test --workspace` + `cargo nextest run`
- [X] T043 Run all quickstart.md validation scenarios (§2–§11) end-to-end and confirm pass
- [X] T044 [P] Verify all 14 const fn style presets + 10 Unicode symbol constants + 15 color constants + 15 ANSI fallback constants + 3 non-const state theme fns + 1 LayoutMode enum + 1 layout_mode fn + 3 block builder fns + 1 resolve_color fn match `contracts/design-system-api.md` API surface exactly (FR-002 through FR-017, resolve_color invariant)
- [X] T045 Run `cargo fmt --check` on the `leiden-tui` crate
- [X] T052 Write test: render loop completes each frame in under 50 ms, including channel drain and widget rendering, at 20 FPS poll rate (SC-007, design-system.md §7)
- [X] T053 [P] Run performance contracts under `--release`: `ct --workspace --release` to exercise `#[cfg(not(debug_assertions))]`-gated perf tests (Constitution §DevWorkflow, SC-007)
- [X] T054 Write test: no emoji characters appear in rendered TUI output; all symbols drawn from documented Unicode BMP set (SC-009, FR-016, design-system.md §4.3)
- [X] T055 Write test: partitions with ≤100 communities render without scrolling; >100 communities scroll with deterministic color cycling (SC-010, FR-018)
- [X] T056 Run `cargo deny check` for dependency advisories and licenses (Constitution §DevWorkflow CI pipeline)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Phase 1. BLOCKS all user story phases.
- **US1 (Phase 3)**: Depends on Phase 2 (color constants for state colors). Can start in parallel with US2 after Phase 2.
- **US2 (Phase 4)**: Depends on Phase 2 (community colors). Can start in parallel with US1 after Phase 2.
- **US3 (Phase 5)**: Depends on Phase 2. Independent of US1/US2.
- **US4 (Phase 6)**: Depends on Phase 2 (accent colors). Independent of US1–US3.
- **US5 (Phase 7)**: Depends on Phase 2 (border styles). Independent of US3–US4; shares `styles.rs` with US1/US2.
- **US6 (Phase 8)**: Depends on Phase 2 (ANSI fallbacks, supports_truecolor). Independent of US1–US5.
- **US7 (Phase 9)**: Depends on Phase 2 + US1 (status bar rendering, state colors for Running state).
- **Help Overlay (Phase 3.5)**: Depends on Phase 2 (color constants + ANSI fallbacks for `BG_1`/`ACCENT_PRIMARY`). Independent of US1–US7.
- **Polish (Phase 10)**: Depends on completion of all user story phases + Phase 3.5.

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories (MVP focus)
- **US2 (P1)**: Can start after Phase 2 — independent, but shares `styles.rs` with US1
- **US3 (P2)**: Can start after Phase 2 — independent
- **US4 (P2)**: Can start after Phase 2 — independent
- **US5 (P2)**: Can start after Phase 2 — independent (focus/border styles don't depend on other stories)
- **US6 (P3)**: Can start after Phase 2 — independent (fallback + contrast validation)
- **US7 (P3)**: Depends on US1 (shared `status_bar.rs` edits) for Running state status bar rendering

### Within Each User Story

- Tests MUST be written and observed failing before implementation
- Style preset tests precede preset implementation (TDD)
- Widget updates follow style preset implementation
- Snapshot tests validate integration last

### Parallel Opportunities

- Phase 1: T002 and T003 can run in parallel (different files: Cargo.toml vs mod.rs)
- Phase 2: T004 (color const test) and T010 (contrast test) can run in parallel (both test files, no impl dependency)
- Phases 3–9: US1–US6 can all start in parallel after Phase 2 (different widget files: status_bar.rs, community.rs+graph.rs, mod.rs, log_pane.rs, event.rs+mod.rs, colors.rs tests respectively)
- US1 and US2 share `styles.rs` — order T012 (US1 state theme) before T016 (US2 table styles) to avoid merge conflicts
- Phase 10: T041 and T044 can run in parallel (lint vs API verification)
- **IMPORTANT**: T031 (`panel_block`, `focused_block`, `unfocused_block`) is in Phase 7, but data-model §8 shows `community.rs`, `graph.rs`, and `log_pane.rs` import `panel_block`. T031 MUST be completed before T017/T018/T026 use `panel_block()` in those widget files — or these widget updates must use inline `Block` construction and defer `panel_block` adoption to Phase 7.

---

## Parallel Example: Foundational Phase

```bash
# T004 writes color-constant tests, T010 writes contrast-ratio tests — both test sections in colors.rs, no dependency:
Task: "Write failing tests: all 26 color constants compile as const items"
Task: "Write contrast ratio validation test computing WCAG ratios for all documented pairs"
```

---

## Implementation Strategy

### MVP First (User Story 1 + Phase 2)

1. Complete Phase 1: Setup (toolchain + lint + module declarations)
2. Complete Phase 2: Foundational (colors.rs — all color constants, ANSI fallbacks, `resolve_color()`, tests)
3. Complete Phase 3: US1 (state theming — color + symbol + label per AppState)
4. Complete Phase 3.5: Help Overlay (FR-017 — key bindings grouped overlay, dismissable)
5. **STOP and VALIDATE**: Test US1 + Help Overlay independently — launch TUI, verify 4 states visually distinct, verify `?` opens help overlay

### Incremental Delivery

1. Complete Phases 1–2 → Foundation ready (including `resolve_color()`)
2. Add US1 → Test independently → Validated (MVP)
3. Add Phase 3.5 (Help Overlay) → Test independently → Validated (FR-017)
4. Add US2 → Test independently → Validated (community colors + selection)
5. Add US3 → Test independently → Validated (responsive layout)
6. Add US4 → Test independently → Validated (log severity)
7. Add US5 → Test independently → Validated (focus navigation)
8. Add US6 → Test independently → Validated (accessibility + fallback)
9. Add US7 → Test independently → Validated (progress visualization)
10. Phase 10: Full lint + test + quickstart + deny validation

### Parallel Team Strategy

With multiple developers:

1. Team completes Phase 1 + Phase 2 (colors.rs foundation) together
2. Once Phase 2 is done (parallel-safe, different widget files):
   - Developer A: US1 (status_bar.rs state theming)
   - Developer B: US2 (community.rs + graph.rs color parity)
   - Developer C: US3 (mod.rs layout engine)
   - Developer D: US4 (log_pane.rs severity styling)
- Notes: US1/US2/US5 share `styles.rs` with US3 — coordinate merge order. US5 (event.rs + mod.rs) and US3 (mod.rs) share `mod.rs` — sequence appropriately. US7 depends on US1 (shared status_bar.rs). US1/US2 must complete before Phase 3.5 (Help Overlay) uses `BG_1`/`ACCENT_PRIMARY` constants.
3. Phase 3.5 (Help Overlay) can start after US1 completes (needs state theme constants for overlay border)
4. US6 can run independently (pure test + colors.rs validation)
5. All stories complete and integrate independently

---

## Done When

- [ ] tasks.md generated with all phases, task IDs (T001–T056), and file paths
- [ ] Extension hooks dispatched or skipped (no `.specify/extensions.yml` registered — skipped silently)
- [ ] Completion reported with task count, story breakdown, and MVP scope

---

## Notes

- **[P]** tasks = different files, no dependencies on incomplete tasks
- **[Story]** label maps task to specific user story for traceability (US1–US7)
- TDD is NON-NEGOTIABLE (Constitution §V) — every test task precedes its implementation task. New functions (resolve_color, help_overlay, render budget, etc.) MUST follow the same pattern: T046 → T047, T048 → T049, T052 → T053.
- Tests mutate `std::env` variables — use `--test-threads=1` for `supports_truecolor` tests to avoid races
- All `pub` items MUST carry `///` doc comments (`missing_docs = deny`, Constitution §IV)
- All style presets MUST be `const fn` (Constitution §II, plan.md §Constraints)
- Verify tests fail before implementing — commit after each task or logical group
- Stop at each checkpoint to validate story independently before proceeding
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Panel toggle redistribution rules live in `design-system.md §3.5` (Full/Compact, Stacked, Minimal) — T022 implements these
- 56 tasks total across 11 phases; 7 user stories (2 P1 MVP, 3 P2, 2 P3) + 1 cross-cutting phase (Phase 3.5: Help Overlay)
