# Non-Functional, Accessibility & Performance Quality Checklist: TUI Visual Explanation

**Purpose**: Validate specification quality for performance budgets, WCAG accessibility, terminal geometry constraints, color fallbacks, and numerical stability for the TUI Visual Explanation feature.
**Created**: 2026-09-01
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [contracts/tui-visual-explanation.md](../contracts/tui-visual-explanation.md) | [research.md](../research.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

---

## 1. Frame Rate & Simulation Performance Requirements

- [x] CHK001 Is the target animation frame rate (20 FPS / 50ms tick interval) quantified with measurable frame-time budgets (≤ 16ms compute budget)? [Measurability, Spec §SC-002, Plan §Technical Context]
  > **REVIEW NOTE**: Satisfied. Unified across spec.md SC-002, plan.md, and contract §3.2 at 20 FPS (50ms event tick loop) with a frame computation budget ≤ 16ms, leaving > 34ms idle headroom per tick.
- [x] CHK002 Are graph scaling benchmarks defined for performance verification with the minimum benchmark fixture (≥ 50 nodes, 100 edges)? [Completeness, Spec §SC-002, Plan §Technical Context]
  > **REVIEW NOTE**: Satisfied. Benchmark gate `benches/simulation_perf.rs` is explicitly mandated in plan.md, asserting a physics tick budget of ≤ 5ms on a 50-node/100-edge fixture.
- [x] CHK003 Is memory allocation discipline specified for the physics simulation loop (avoiding heap reallocations during active ticks)? [Clarity, Plan §Technical Context, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Contract §3.2 mandates pre-allocation of node positions and velocities in flat pre-sized buffers at initialization; `ForceSimulation::tick()` executes with zero heap reallocations.
- [x] CHK004 Does the specification define CPU throttling or tick pausing behavior when the TUI is idle or in a paused playback state? [Coverage, Non-Functional, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Contract §4.2 explicitly mandates that when playback is paused or idle, physics simulation ticks are halted and the event poll blocks up to 200ms (CPU utilization < 0.1%).

## 2. Color Contrast & WCAG Accessibility Requirements

- [x] CHK005 Are contrast ratios for all foreground text tokens (`FG_0`, `FG_1`, `FG_2`) quantified against `BG_0` / `BG_1` with explicit WCAG AA/AAA thresholds (≥ 4.5:1 / ≥ 7.0:1)? [Measurability, Research §2, Contract §5.1]
  > **REVIEW NOTE**: Satisfied. Contract §5.1 table provides verified contrast ratios for all text tokens against both `BG_0` and `BG_1` (`FG_0`: 11.1:1/10.3:1 AAA, `FG_1`: 8.1:1/7.4:1 AAA, `FG_2`: 4.8:1/4.5:1 AA).
- [x] CHK006 Is the 12-color community palette (`COMMUNITY_COLORS`) verified for minimum 4.5:1 contrast and CIELAB perceptual distinctness? [Traceability, Research §2, Contract §5.1]
  > **REVIEW NOTE**: Satisfied. Contract §5.1 verifies that all 12 community colors achieve ≥ 4.5:1 contrast against `#1a1b26`/`#1f202e` with pairwise perceptual distance ΔE* ≥ 25.0 in CIELAB space.
- [x] CHK007 Is color alone prevented from being the sole carrier of critical information (e.g., node labels and community IDs provided alongside colors)? [Coverage, Accessibility, Contract §3.1]
  > **REVIEW NOTE**: Satisfied. Contract §3.1 requires node ID labels (for N ≤ 40), textual explanation tiers, and community centroid cluster proximity as redundant non-color indicators.
- [x] CHK008 Is the explicit prohibition of `Modifier::ITALIC` documented to prevent broken text artifacts on legacy terminal emulators? [Consistency, Research §2, Contract §5.2]
  > **REVIEW NOTE**: Satisfied. Documented in contract §5.2, research §2, and design-system.md §4.1.

## 3. Terminal Geometry & Viewport Dimension Guards

- [x] CHK009 Is the minimum geometry threshold (80 columns × 24 rows) strictly enforced and unified across all contract and spec documents? [Consistency, Spec §FR-007, Contract §4.1]
  > **REVIEW NOTE**: Satisfied. Uniformly specified at 80×24 across spec FR-007, design-system §0.2, contract §4.1, and data-model §2.6.
- [x] CHK010 Are layout proportions dynamically bounded so that no panel collapses below readable height during window resizing? [Clarity, Contract §1.1, Spec §Edge Cases]
  > **REVIEW NOTE**: Satisfied. Contract §1.1 bounds the Explanation Panel to min 8 rows and Graph Canvas to min 15 rows at 80×24.
- [x] CHK011 Is the warning overlay modal layout, text alignment, and border rendering specified for viewports smaller than 80x24? [Completeness, Contract §4.1]
  > **REVIEW NOTE**: Satisfied. Contract §4.1 specifies a centered 46×7 modal dialog with exact text, rounded borders, and interaction blocking.
- [x] CHK012 Does the spec require that simulation state, playback position, and active presets remain intact throughout resize pause-and-resume cycles? [Completeness, Spec §Edge Cases, Contract §4.2]
  > **REVIEW NOTE**: Satisfied. Contract §4.2 and spec §Edge Cases explicitly require zero loss of physics coordinates, playback state, or community assignments across resize cycles.

## 4. ANSI & Terminal Compatibility Fallbacks

- [x] CHK013 Are color degradation and fallback strategies specified for terminals supporting only 256-color or 16-color ANSI palettes? [Coverage, Non-Functional, Contract §5.1]
  > **REVIEW NOTE**: Satisfied. Contract §5.1 and design-system §10 provide complete fallback mapping tables to 256-color indices and 16-color ANSI equivalents.
- [x] CHK014 Are Unicode glyph fallback requirements (e.g., ASCII fallback for discs `●` and rounded border characters) documented for non-UTF-8 terminal environments? [Coverage, Edge Case, Contract §5.2]
  > **REVIEW NOTE**: Satisfied. Contract §5.2 and design-system §4.3 specify fallback to `BorderType::Plain` (`+--+`, `|  |`) and ASCII symbols (`*`, `o`) in non-UTF-8 environments.
- [x] CHK015 Is terminal raw mode and alternate screen buffer cleanup guaranteed upon exit or signal interruption? [Completeness, Contract §4.2, Constitution Principle VI]
  > **REVIEW NOTE**: Satisfied. Contract §4.2 mandates a panic hook and signal handler for `SIGINT`, `SIGTERM`, and `SIGHUP` guaranteeing raw mode restoration and `LeaveAlternateScreen`.

## 5. Robustness, Panic Prevention & Resource Limits

- [x] CHK016 Are all mathematical operations in the force simulation (Euclidean distance, division by distance squared) guarded against division-by-zero and NaN propagation? [Clarity, Constitution §Numerical Stability, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Contract §3.2 specifies softening factor ε=0.03 for repulsion and guarded displacement vector `δ⃗ = (⃗u - ⃗v)/max(d, ε) · (d_min - d) × 0.5`, preventing 0/0 → NaN.
- [x] CHK017 Are upper bound dataset size limits documented where force simulation performance may degrade gracefully? [Coverage, Edge Case, Contract §3.2]
  > **REVIEW NOTE**: Satisfied. Contract §3.2 sets N ≤ 200 nodes / E ≤ 1000 edges for full force simulation, gracefully transitioning to radial cluster centroids for N > 200.
- [x] CHK018 Is error propagation strictly data-driven without `unwrap()`, `expect()`, or panics in all physics, parsing, and rendering modules? [Consistency, Constitution Principle III, Plan §Constitution Check]
  > **REVIEW NOTE**: Satisfied. Constitution Principle III and Plan §Constitution Check strictly enforce panic-free error propagation via `thiserror` domain error types.
- [x] CHK019 Are MSRV compatibility requirements (Rust 1.88.0+ / Ratatui 0.30.2) documented and verified across all dependency manifests? [Traceability, Plan §Technical Context, Constitution §Additional Constraints]
  > **REVIEW NOTE**: Satisfied. Ratatui 0.30.2 pinned with MSRV floor 1.88.0 ratified in Constitution Additional Constraints and Plan Technical Context.

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `/speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`
- Add comments or findings inline during PR review
- Items are numbered sequentially (CHK001–CHK019) for easy reference
