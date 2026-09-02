# Requirements Quality Review Checklist: TUI Visual Explanation

**Purpose**: Validate specification completeness, clarity, consistency, measurability, and edge-case coverage for the TUI Visual Explanation feature prior to final implementation gating.
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [contracts/tui-visual-explanation.md](../contracts/tui-visual-explanation.md) | [contracts/explanation-content.md](../contracts/explanation-content.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## 1. Requirement Completeness & Scenario Coverage

- [x] CHK001 Are acceptance scenarios explicitly documented for all initial startup modes, distinguishing preset loading from custom CLI dataset paths? [Completeness, Spec §User Story 1]
  > **REVIEW NOTE**: Satisfied. Acceptance scenarios 1 & 2 in Spec User Story 1 explicitly distinguish preset startup from custom CLI arguments.
- [x] CHK002 Does the specification define visual and narrative transitions across all five algorithm phases (Initial, Local Moving, Refinement, Aggregation, Completed)? [Completeness, Spec §User Story 2, Contract §2]
  > **REVIEW NOTE**: Satisfied. Narrative transitions are codified across all 5 phases in Contract §2 and Data Model §2.
- [x] CHK003 Are completion and summary state requirements defined for graphs with only 1 or 2 small communities? [Coverage, Spec §User Story 3]
  > **REVIEW NOTE**: Satisfied. Completion summary handles any community count $k \ge 1$ via dynamic headline & stats.
- [x] CHK004 Is the expected error behavior specified when a user launches the TUI with an invalid or non-existent file path? [Coverage, Exception Flow, Contract §2.4]
  > **REVIEW NOTE**: Satisfied. `PresetDataset::from_cli_path()` returns domain `TuiError::DatasetNotFound` with a clean stderr message and exit code.
- [x] CHK005 Are requirements fully documented for restarting or resetting an explanation run mid-execution without restarting the process? [Completeness, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Key `r` resets simulation physics and state machine back to Step 1 per Contract §2.1.

## 2. 2D Force-Directed Simulation & Canvas Rendering Requirements

- [x] CHK006 Is the initial unclustered spatial distribution of nodes explicitly specified with coordinate bounding constraints? [Clarity, Spec §FR-002, Data Model §1.1]
  > **REVIEW NOTE**: Satisfied. Bounded in $[0.05, 0.95]$ normalized canvas space (Contract §3.2, Data Model §2.1).
- [x] CHK007 Are the mathematical dynamics of attractive forces toward community centroids versus repulsive inter-node forces specified unambiguously? [Clarity, Spec §FR-003, Research §1]
  > **REVIEW NOTE**: Satisfied. Attractive springs ($F_{attr} = k_{attr} \cdot d$) and softened electrostatic repulsion ($F_{rep} = k_{rep}/\max(d^2, \epsilon^2)$) defined in Contract §3.2.
- [x] CHK008 Does the specification define velocity damping and convergence criteria per simulation tick to avoid endless visual oscillation? [Completeness, Spec §FR-003, Data Model §1.2]
  > **REVIEW NOTE**: Satisfied. Damping constant $\alpha = 0.85$ and 25-tick convergence budget per step ratified in Contract §3.2.
- [x] CHK009 Are node and edge rendering styles (Unicode symbols, continuous lines, dimmed inter-community edges) explicitly defined for all clustering phases? [Clarity, Contract §3, Spec §FR-001]
  > **REVIEW NOTE**: Satisfied. Node discs `●` (`U+25CF`), `Line` widgets, intra-community colors, and `FG_3` dimmed inter-cluster edges ratified in Contract §3.1.
- [x] CHK010 Is the maximum node count threshold for rendering adjacent node ID labels specified with measurable limits? [Clarity, Contract §3.3]
  > **REVIEW NOTE**: Satisfied. Strict $N \le 40$ threshold defined in Contract §3.1.

## 3. 3-Tier Explanatory Content & Readability Requirements

- [x] CHK011 Is the 3-tier structure (Step Headline, Plain-English Analogy, Live Badges) specified with exact layout height constraints? [Completeness, Spec §FR-004, Contract §1.1]
  > **REVIEW NOTE**: Satisfied. 3-tier layout allocated 35% of Main Content Area (min 8 rows at 80×24) per Contract §1.1.
- [x] CHK012 Is the 8th-grade reading level target (Flesch-Kincaid index ≤ 8.0) quantified with an objective scoring formula and validation method? [Measurability, Spec §SC-003, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Flesch-Kincaid index formula and $\le 8.0$ ceiling verified in Contract §4.1 and tested via unit tests.
- [x] CHK013 Does the specification explicitly blacklist technical jargon (e.g., modularity, gamma resolution, eigenvectors) from user-facing analogy texts? [Clarity, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. 9 prohibited technical terms enumerated in Contract §4.2.
- [x] CHK014 Are allowed everyday metaphors (friend groups, clubs, lunch tables) documented consistently across all algorithmic phases? [Consistency, Contract §2, Contract §3.3]
  > **REVIEW NOTE**: Satisfied. Whitelist of metaphors specified in Contract §4.3 and mapped across all phases in Contract §2.
- [x] CHK015 Are requirements defined for how the explanation panel handles multi-line text wrapping on narrow terminal widths? [Coverage, Edge Case, Data Model §3]
  > **REVIEW NOTE**: Satisfied. `wrapped_analogy_lines()` word-wraps at $\le 76$ columns with max 3 lines (Data Model §3).

## 4. Playback Controls & Dual Granularity Stepping Requirements

- [x] CHK016 Are keybindings and execution preconditions specified for all playback actions (`Space` for Play/Pause, `n`/Right Arrow for Step, `t` for Granularity)? [Completeness, Spec §FR-005, Contract §2]
  > **REVIEW NOTE**: Satisfied. Complete matrix for 9 key controls ratified in Contract §2.1.
- [x] CHK017 Is the behavioral difference between `PhaseLevel` stepping and `MicroStep` stepping defined with exact pausing trigger conditions? [Clarity, Spec §FR-005, Data Model §1.5]
  > **REVIEW NOTE**: Satisfied. `PhaseLevel` pauses at major phase completion; `MicroStep` pauses after each node migration (Data Model §2.5).
- [x] CHK018 Is the auto-play stepping tick speed (default interval and behavior under rapid input) specified in milliseconds? [Clarity, Data Model §1.5, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Auto-play tick speed fixed at 200ms per step in Data Model §2.5.
- [x] CHK019 Are keyboard navigation rules specified for when playback reaches the final completed phase? [Completeness, Spec §FR-005, Contract §2]
  > **REVIEW NOTE**: Satisfied. Reaching completion transitions status to `✔ Finished`; user can press `r` to restart or `1`–`3` to load another preset.
- [x] CHK020 Is a global help keybinding overlay (`?`) and its modal dismissal behavior fully specified? [Completeness, Contract §2]
  > **REVIEW NOTE**: Satisfied. Centered 50×14 help modal and dismissal keys specified in Contract §2.1.

## 5. Preset Datasets & Custom Input Requirements

- [x] CHK021 Are exact node counts, edge counts, and graph structures defined for all three curated presets (Karate Club, Two Cliques, Random Mess)? [Completeness, Spec §FR-006, Data Model §1.4]
  > **REVIEW NOTE**: Satisfied. Karate Club (34N/78E), Two Cliques (16N/56E), Random Mess (30N/60E) specified in Data Model §2.4.
- [x] CHK022 Are key shortcuts for instant preset switching (`1`, `2`, `3`) specified alongside their active state preconditions? [Consistency, Contract §2, Data Model §1.4]
  > **REVIEW NOTE**: Satisfied. Shortcuts `1`, `2`, `3` specified in Contract §2.1.
- [x] CHK023 Is the fallback behavior specified when a custom dataset supplied via CLI contains disconnected components or isolated nodes? [Coverage, Edge Case, Spec §FR-006]
  > **REVIEW NOTE**: Satisfied. Disconnected components relax into separated cluster centroids seamlessly per Contract §3.2.
- [x] CHK024 Does the specification define whether preset switching is permitted while auto-play is actively running? [Ambiguity, Contract §2.2]
  > **REVIEW NOTE**: Satisfied. Preset switching is permitted anytime; it automatically pauses auto-play and resets state to Step 1 (Contract §2.2).

## 6. Viewport Scaling, Resize & Terminal Dimension Guard

- [x] CHK025 Is the minimum supported terminal geometry (80 columns × 24 rows) quantified and aligned across spec, plan, and contracts? [Consistency, Spec §FR-007, Contract §4]
  > **REVIEW NOTE**: Satisfied. 80×24 strictly enforced in Spec FR-007, Plan, Contract §4.1, and Data Model §2.6.
- [x] CHK026 Are modal warning overlay contents, border styles, and interaction locking rules defined for undersized viewports? [Clarity, Spec §Edge Cases, Contract §4]
  > **REVIEW NOTE**: Satisfied. Centered 46×7 modal specifications ratified in Contract §4.1.
- [x] CHK027 Is dynamic canvas coordinate re-normalization specified for terminal resize events occurring during active playback? [Completeness, Spec §FR-007, Spec §Edge Cases]
  > **REVIEW NOTE**: Satisfied. Instant coordinate re-projection on render frame specified in Contract §4.2.
- [x] CHK028 Does the specification define whether simulation physics state and playback progress are preserved when resizing across the 80x24 boundary? [Clarity, Spec §Edge Cases, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Zero state or data loss across resize cycles guaranteed in Contract §4.2.

## 7. Accessibility, Visual Hierarchy & Non-Functional Requirements

- [x] CHK029 Are contrast ratio requirements (WCAG AA/AAA ≥ 4.5:1 against dark backgrounds) quantified for all text and node colors? [Measurability, Plan §Constraints, Research §2]
  > **REVIEW NOTE**: Satisfied. Verified contrast metrics tabulated for all tokens in Contract §5.1.
- [x] CHK030 Is the 12-color CIELAB-distinct community palette explicitly cross-referenced with `design-system.md`? [Traceability, Contract §3.1, Plan §Summary]
  > **REVIEW NOTE**: Satisfied. Cross-referenced in Contract §5.1 and Plan Summary with `design-system.md` §9.
- [x] CHK031 Is the prohibition of `Modifier::ITALIC` specified to guarantee universal terminal emulator compatibility? [Consistency, Research §2, Contract §5.2]
  > **REVIEW NOTE**: Satisfied. Banned across all widgets in Contract §5.2 and design-system.md §4.1.
- [x] CHK032 Is the animation frame rate performance requirement (20 FPS / 50ms tick loop on ≥50 nodes / 100 edges) objectively testable? [Measurability, Spec §SC-002, Plan §Technical Context]
  > **REVIEW NOTE**: Satisfied. Unified 20 FPS (50ms tick interval, $\le 16\text{ms}$ compute budget) and benchmark gate `benches/simulation_perf.rs` ratified in Spec SC-002 and Plan Technical Context.

## 8. Edge Cases, Failure Modes & Boundary Conditions

- [x] CHK033 Are requirements defined for graphs with zero edges (all isolated nodes)? [Coverage, Edge Case, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Zero-edge graphs relax under electrostatic repulsion to fill the canvas as distinct singletons.
- [x] CHK034 Are requirements defined for graphs that collapse into a single monolithic community? [Coverage, Edge Case, Contract §2]
  > **REVIEW NOTE**: Satisfied. Single community convergence summary specified in Contract §2.
- [x] CHK035 Is the behavior specified when a user enters unrecognized keystrokes during active animation? [Coverage, Edge Case, Contract §2.1]
  > **REVIEW NOTE**: Satisfied. Unrecognized keys are safely ignored without altering state (Contract §2.1).
- [x] CHK036 Are requirements defined for handling terminal signals (SIGINT / SIGTERM / SIGHUP) without corrupting terminal state? [Coverage, Non-Functional, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Dedicated panic hook and signal handler cleanup ratified in Contract §4.2.

## 9. Specification Consistency & Contract Traceability

- [x] CHK037 Do the status bar visual indicator requirements in `contracts/tui-visual-explanation.md` align with `spec.md §FR-005`? [Consistency, Spec §FR-005, Contract §1.1]
  > **REVIEW NOTE**: Satisfied. Status bar mockup in Contract §1.1 aligns directly with Spec FR-005 playback states.
- [x] CHK038 Are the phase names in `contracts/explanation-content.md` consistent with `data-model.md §ExplanationState`? [Consistency, Contract §1, Data Model §1.3]
  > **REVIEW NOTE**: Satisfied. Phase enum names match identically across Contract §1 and Data Model §2.3.
- [x] CHK039 Are all functional requirements (FR-001 through FR-007) traced to measurable acceptance scenarios and user stories? [Traceability, Spec §User Scenarios, Spec §Requirements]
  > **REVIEW NOTE**: Satisfied. 100% of functional requirements FR-001 through FR-007 trace to User Stories 1–3.
- [x] CHK040 Are domain error types and fallible layout calculations documented without allowing panics or unwrap shortcuts? [Consistency, Constitution Principle III, Plan §Constitution Check]
  > **REVIEW NOTE**: Satisfied. Constitution Principle III enforces zero panics/unwraps with `thiserror` domain error types.

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- Add comments or findings inline during PR review
- Items are numbered sequentially (CHK001–CHK040) for easy reference
