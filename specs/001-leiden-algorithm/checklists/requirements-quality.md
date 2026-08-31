# Requirements-Quality Checklist: Leiden Algorithm in Rust

**Purpose**: Reviewer-owned requirements-quality review for the 001-leiden-algorithm
spec, plan, research, data model, and contracts. This is "unit tests for English" —
it validates whether the *requirements themselves* are well-written, complete,
unambiguous, and ready for implementation. It does NOT verify implementation.
**Created**: 2026-08-30
**Feature**: [spec.md](../spec.md), [plan.md](../plan.md), [research.md](../research.md),
[data-model.md](../data-model.md), [contracts/](../contracts/), [quickstart.md](../quickstart.md)

**Review Ownership**: Mark an item `[x]` only when the reviewer determines the
requirements-quality criterion is satisfied. `[x]` does not mean implementation
work is complete.

**Marker Semantics**: `[x]` = reviewer-approved requirements quality. `$speckit-implement`
reads checkbox state as a gate and MUST NOT modify markers.

---

## Requirement Completeness

- [ ] CHK001 — Are all three phases of Leiden (local moving, refinement, aggregation) explicitly required as separate algorithmic stages? [Completeness, Spec §FR-009, Research §1.5]
- [ ] CHK002 — Is the modularity ΔQ formula explicitly required to match Traag 2019 Eq. (A5) including the `+ kᵢ` self-loop compensation term? [Completeness, Spec §FR-009, Research §1.3]
- [ ] CHK003 — Is the refinement merge predicate `Σ_in(D, C\D) ≥ k_D · (k_C − k_D)` (Traag Algorithm A.2 line 37) explicitly required? [Completeness, Spec §FR-009, Research §1.3]
- [ ] CHK004 — Is the aggregation rule (aggregate node = non-empty refined community; aggregate edge = sum of cross-community edges) explicitly required? [Completeness, Spec §FR-009, Research §1.3]
- [ ] CHK005 — Are both termination conditions (convergence and iteration cap) required, with the cap defaulting to 10? [Completeness, Spec §FR-003a, Quickstart §3]
- [ ] CHK006 — Is the convergence definition (no node moved in local moving AND refinement didn't split anything) explicitly required? [Clarity, Spec §FR-003a, Research §1.3]
- [ ] CHK007 — Are all four edge-case classes (empty graph, single-node, disconnected, self-loop) explicitly required to be handled at the input boundary? [Completeness, Spec §Edge Cases]
- [ ] CHK008 — Is the iteration-cap behavior (return best partition seen so far, set `termination_reason = iteration_cap`, report iteration count) explicitly required? [Completeness, Spec §FR-003a]
- [ ] CHK009 — Are tie-breaking rules (lowest node id wins) explicitly required for both local-moving and refinement? [Completeness, Spec §Edge Cases, Data Model §1.4]
- [ ] CHK010 — Is the requirement that the library maps user-supplied `NodeId` types to dense internal `u32` indices and stores adjacency in CSR form explicitly stated in the spec? [Completeness, Spec §FR-001, Data Model §1.3]

## Requirement Clarity

- [ ] CHK011 — Is "well-connected community" (FR-002) defined with a measurable criterion (internally connected after refinement)? [Clarity, Spec §FR-002]
- [ ] CHK012 — Is "maximizes modularity under the Leiden algorithm's guarantee" (US-1) clarified as "non-decreasing modularity under the local-moving ΔQ formula"? [Clarity, Spec §US-1, Research §1.3]
- [ ] CHK013 — Is "deterministic — identical inputs yield byte-identical output" (FR-004) clarified with a tie-breaking rule that covers both intra-community and inter-community ties? [Clarity, Spec §FR-004, Edge Cases]
- [ ] CHK014 — Is "default deterministic but unspecified" seed clarified as "no RNG involved at all when seed is None"? [Clarity, Spec §Assumptions, Library API §4]
- [ ] CHK015 — Is the JSON partition output shape (FR-007b) defined with exact field names, types, and value ranges for `termination_reason`? [Clarity, Spec §FR-007b, CLI Schema §1.4.1]
- [ ] CHK016 — Is the edge-list separator rule (tab or comma, auto-detected) defined with a deterministic tie-break when both appear? [Clarity, Spec §FR-007a, CLI Schema §1.3.1]
- [ ] CHK017 — Is the format dispatch rule (extension hint vs. first-non-whitespace-byte sniff) defined with precedence? [Clarity, CLI Schema §1.3.2]
- [ ] CHK018 — Is the resolution parameter γ behavior at `0.0` defined (rejected with typed error) vs. at very small positive values (accepted, may produce many singleton communities)? [Clarity, Spec §FR-003, Edge Cases]
- [ ] CHK019 — Is the default TUI key for quit (`q` AND `Ctrl+C`) clarified with the precedence rule when both are bound? [Clarity, CLI Schema §2.3]
- [ ] CHK020 — Is the TUI log pane retention rule (500-entry ring buffer, eviction policy) specified? [Clarity, TUI Events §5]

## Requirement Consistency

- [ ] CHK021 — Do FR-001 (CSR + dense `u32` indices) and FR-006 (public API without internal mutable state) align — i.e., is the public `CsrGraph` opaque so internal CSR fields are not leaked? [Consistency, Spec §FR-001, §FR-006, Data Model §1.3]
- [ ] CHK022 — Do FR-002 (well-connected communities) and FR-009 (follow published algorithm) align — i.e., is "well-connected" defined as a post-condition of refinement, not local moving? [Consistency, Spec §FR-002, §FR-009, Research §1.3]
- [ ] CHK023 — Do FR-003a (terminate on convergence OR cap) and the edge-case "iteration cap reached before convergence" align on which termination reason is recorded? [Consistency, Spec §FR-003a, §Edge Cases]
- [ ] CHK024 — Do the determinism requirement (FR-004) and the stochastic-variant out-of-scope assumption align — i.e., is the algorithm required to use lowest-id tie-break in ALL phases, not just local moving? [Consistency, Spec §FR-004, §Assumptions, Research §1.4]
- [ ] CHK025 — Do FR-007a (edge-list + JSON input) and FR-008 (malformed input rejected at boundary) align on whether JSON dangling-node errors reference the JSON field path or the parsed edge index? [Consistency, Spec §FR-007a, §FR-008, CLI Schema §3.3]
- [ ] CHK026 — Do FR-007b (JSON default, `--format text` switch) and FR-008 (rejected formats produce typed error) align on whether the CLI defaults to JSON when `--format` is absent in both interactive and non-interactive modes? [Consistency, Spec §FR-007b, §FR-008]
- [ ] CHK027 — Do FR-010 (structured observability events) and Constitution §VI (tracing only) align on the exact list of phases that emit events? [Consistency, Spec §FR-010, Constitution §VI, Data Model §1.8]
- [ ] CHK028 — Does the SC-003 success criterion ("1,000 random weighted graphs … algorithm completes without panic") align with the constitution's `panic = deny` lint and `LeidenError::EmptyGraph` early-exit path? [Consistency, Spec §SC-003, Constitution §III]

## Acceptance Criteria Quality

- [ ] CHK029 — Is SC-001 (≤ 5 s for 100-node, 500-edge fixture) quantified with a specific CPU profile and a specific criterion (wall-clock on one thread, not CPU-time)? [Measurability, Spec §SC-001, Quickstart §7]
- [ ] CHK030 — Is SC-002 (≥ 90% match on reference fixtures) defined with a clear "match" criterion (same community count? same co-assignments? tolerance on modularity delta?)? [Measurability, Spec §SC-002, Quickstart §2]
- [ ] CHK031 — Is SC-003 (no NaN, no panic, no disconnected community across 1000 random graphs) defined with the random graph generator's parameters (edge distribution, weight distribution)? [Measurability, Spec §SC-003, Quickstart §8]
- [ ] CHK032 — Is SC-004 ("public library API exposes exactly the capabilities described in FR-006") measurable via `cargo doc` (every public item has docs) and a static check that no public method exposes `&mut self` on a `CsrGraph`? [Measurability, Spec §SC-004, Library API §1]
- [ ] CHK033 — Is SC-005 (every malformed input scenario produces a structured, line-referencing error) defined with the exact required line-number format (`<path>:<line>: <message>`)? [Measurability, Spec §SC-005, CLI Schema §3.3]

## Scenario Coverage

- [ ] CHK034 — Are requirements specified for the "best partition seen so far" return value when iteration cap is reached — i.e., must the library retain partition state across iterations? [Coverage, Spec §FR-003a, §Edge Cases]
- [ ] CHK035 — Are requirements specified for the TUI when the user supplies a `GRAPH_FILE` that fails to load (e.g., path doesn't exist, parse error)? [Coverage, CLI Schema §2.5]
- [ ] CHK036 — Are requirements specified for the TUI when `--gamma` is edited mid-run to a value ≤ 0 (validation on apply, not just on start)? [Coverage, CLI Schema §2.3]
- [ ] CHK037 — Are requirements specified for what happens when `crossterm` fails to enter raw mode (TTY already in use, container without TTY) in the TUI? [Coverage, CLI Schema §2.5, Gap]
- [ ] CHK038 — Are requirements specified for the CLI when stdout is closed by the consumer (broken pipe) before the partition is fully written? [Coverage, Gap]
- [ ] CHK039 — Are requirements specified for the orchestrator when the channel to the TUI is full (back-pressure / `LeidenEvent::Throttled`)? [Coverage, TUI Events §2]
- [ ] CHK040 — Are requirements specified for parallel execution — i.e., is single-threaded execution the ONLY supported mode, or is a `rayon`-based parallel mode required as a v1 feature? [Coverage, Spec §Assumptions, Plan §Performance Goals]
- [ ] CHK041 — Are requirements specified for the `--log-file` mode when the file cannot be created (permission denied, disk full) — fall back to stderr or hard error? [Coverage, Gap]

## Edge Case Coverage

- [ ] CHK042 — Is the behavior defined for a graph with `n` nodes but a single self-loop edge (the rejection of self-loops contradicts the US-1 single-node acceptance scenario if interpreted strictly)? [Edge Case, Spec §US-1, §Edge Cases, Conflict]
- [ ] CHK043 — Is the behavior defined for a graph with one node and a single self-loop (rejected as malformed, or accepted as degenerate)? [Edge Case, Gap]
- [ ] CHK044 — Is the behavior defined for a graph with very large node counts where `Σ_tot` could exceed `f64`'s integer-exact range (> 2⁵³)? [Edge Case, Research §1.4, Spec §Edge Cases]
- [ ] CHK045 — Is the behavior defined for parallel edges (same source, target, weight repeated) — silently deduplicated, treated as separate, or rejected? [Edge Case, Spec §Edge Cases, Gap]
- [ ] CHK046 — Is the behavior defined for the TUI when the terminal is resized mid-run (panels should reflow)? [Edge Case, Gap]
- [ ] CHK047 — Is the behavior defined for the CLI when multiple `--format` flags are provided (last wins, error, ignore)? [Edge Case, Gap]
- [ ] CHK048 — Is the behavior defined for the TUI when `RUST_LOG` is set vs. `--log-level` (env override vs. flag precedence)? [Edge Case, Gap]

## Non-Functional Requirements

- [ ] CHK049 — Are performance requirements quantified for each phase independently (local moving, refinement, aggregation) so a regression in one phase is attributable? [NFR, Plan §Performance Goals, Quickstart §7]
- [ ] CHK050 — Are memory requirements specified (peak RSS for the 100k-node / 1M-edge worst case)? [NFR, Gap]
- [ ] CHK051 — Are accessibility requirements specified for the TUI (high-contrast color theme, screen-reader-friendly status messages, no reliance on color alone for community distinction)? [NFR, Gap]
- [ ] CHK052 — Are observability requirements specified for the library when the TUI channel is closed mid-run (does the library keep emitting `tracing` events to stderr/file)? [NFR, TUI Events §7]
- [ ] CHK053 — Are portability requirements specified for the TUI on Windows Terminal vs. legacy cmd.exe vs. WSL (crossterm behavior may differ)? [NFR, Gap]
- [ ] CHK054 — Are reproducibility requirements specified for property tests (fixed seed for the random graph generator so failures are reproducible)? [NFR, Quickstart §8]

## Dependencies & Assumptions

- [ ] CHK055 — Is the assumption "input scale: in-memory only" validated against FR-001 (CSR) — i.e., is there a documented memory bound for the largest expected graph? [Assumption, Spec §Assumptions, Plan §Scale/Scope]
- [ ] CHK056 — Is the assumption "MSRV documented in README" (Constitution Additional Constraints) covered by a concrete README section requirement? [Assumption, Constitution Additional Constraints, Gap]
- [ ] CHK057 — Is the dependency on `rand` justified given the deterministic variant is the only v1 mode — i.e., is `rand` required at all in v1? [Dependency, Plan §Primary Dependencies, Research §1.4]
- [ ] CHK058 — Is the dependency on `crossterm` (vs. `termion` or `termwiz`) documented with the platform-coverage rationale? [Dependency, Research §2.5]
- [ ] CHK059 — Is the dependency on `color-eyre` in the TUI's `main` justified vs. `anyhow` — i.e., is there a documented reason for the choice? [Dependency, Plan §Primary Dependencies, Gap]
- [ ] CHK060 — Is the dependency on `insta` for TUI snapshot tests covered by a CI step (`cargo insta review` on PR)? [Dependency, Quickstart §6, TUI Events §6]

## Ambiguities & Conflicts

- [ ] CHK061 — Is the conflict between "deterministic — identical inputs yield byte-identical output" (FR-004) and "deterministic but unspecified seed" (Assumptions) resolved — i.e., is the default seed `None` (no RNG, fully deterministic by lowest-id tie-break) or `Some(0)`? [Ambiguity, Spec §FR-004, §Assumptions]
- [ ] CHK062 — Is the conflict between "self-loops rejected at boundary" (FR-008) and "self-loops contribute once to kᵢ and e_c" (research §1.4) clarified — i.e., is the rejection at the input parser or at `CsrGraph::from_edges`? [Ambiguity, Spec §FR-008, Research §1.4]
- [ ] CHK063 — Is the conflict between "FR-006 library API without exposing internal mutable state" and "FR-001 user-supplied id ↔ internal u32 mapping preserved on output" clarified — i.e., does "without exposing internal mutable state" forbid `pub` accessor methods like `internal_index_of(&self, id: &Id) -> Option<u32>`? [Ambiguity, Spec §FR-001, §FR-006]
- [ ] CHK064 — Is the conflict between "FR-010 emit structured observability events for every major phase" and the constitution's ban on `println!`/`eprintln!` resolved — i.e., are observability events emitted exclusively via `tracing`, with no parallel `eprintln!` path for "progress to stderr"? [Ambiguity, Spec §FR-010, Constitution §VI]
- [ ] CHK065 — Is the term "binary" in the spec (CLI binary vs. TUI binary) clarified — i.e., does the spec refer to one binary that supports a `--tui` flag, or two separate binaries (`leiden` and `leiden-tui`)? [Ambiguity, Spec §US-3, Plan §Project Structure]
- [ ] CHK066 — Is the term "stable iteration" (used in research §1.3 and data-model §1.9) consistent across all artifacts — i.e., is the spec's "partition is stable across a full local-moving pass with no further modularity improvement" (FR-003a) the SAME condition as research's `local_moving_changed == false ∧ refined_partition == local_moved_partition`? [Conflict, Spec §FR-003a, Research §1.3]

## Traceability

- [ ] CHK067 — Is the requirement ID scheme consistent across spec.md, plan.md, data-model.md, and contracts/ (FR-001..FR-010 referenced uniformly, or are there renumbered duplicates)? [Traceability, All Docs]
- [ ] CHK068 — Are all FR-001..FR-010 IDs traceable to at least one acceptance scenario in a User Story, or are any FRs orphans? [Traceability, Spec §FR-001..§FR-010, §US-1..§US-3]
- [ ] CHK069 — Are all SC-001..SC-005 success criteria traceable to at least one FR or US, or are any SCs orphans? [Traceability, Spec §SC-001..§SC-005, §FR-001..§FR-010]
- [ ] CHK070 — Is every LeidenEvent variant in data-model.md §1.8 traceable to a phase in the spec (graph load, local moving, refinement, aggregation, termination)? [Traceability, Spec §FR-010, Data Model §1.8]
- [ ] CHK071 — Is every LeidenError variant in data-model.md §1.11 traceable to a specific FR (FR-001, FR-003, FR-003a, FR-008)? [Traceability, Spec §FR-001, §FR-003, §FR-003a, §FR-008]
- [ ] CHK072 — Is every TUI key binding in CLI Schema §2.3 traceable to a User Story acceptance scenario or documented as a "navigation" requirement? [Traceability, CLI Schema §2.3, §US-1..§US-3]

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied.
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation.
- `$speckit-implement` reads checklist checkbox state as a gate and must not modify markers.
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `$speckit-specify` and `$speckit-clarify`.
- Items are numbered sequentially (CHK001..CHK072) for easy reference.
- Add reviewer comments inline above the relevant item; do not delete items.