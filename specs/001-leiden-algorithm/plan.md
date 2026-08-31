# Implementation Plan: Leiden Algorithm in Rust

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-leiden-algorithm/spec.md`

## Summary

Implement the Leiden algorithm for community detection (Traag, Waltman, van Eck, 2019) as a
panic-free, fully documented Rust library crate with a workspace-strict lint profile. The
crate is decomposed along its natural algorithm seams — graph representation (CSR-backed),
local moving, refinement, aggregation, and orchestration — and ships alongside two binaries:
a non-interactive CLI (`leiden`) for scripted use and an interactive Terminal User Interface
built on **Ratatui** for inspecting partitions, watching iterative progress, and exploring
community structure on a single TTY. Both binaries consume the same library API; no
domain logic leaks into the binary crates.

*Note: Graphs classified as `DegenerateInput` (zero nodes or zero total edges per `spec.md FR-003a`) short-circuit the outer loop and return `RunResult.iterations == 0`; no `local-moving` / `refinement` / `aggregation` phases execute on that path.*

## Technical Context

**Language/Version**: Rust stable, edition 2024, MSRV documented in `README.md` and pinned
via `rust-toolchain.toml`. `unsafe_code = deny` across the workspace.

**Primary Dependencies**:
- `thiserror` — domain `Error` enums (`#[derive(Debug, Error)]`) for every fallible
  operation; no `unwrap`/`expect`/`panic` in library or binary code.
- `tracing` + `tracing-subscriber` — structured observability mandated by Constitution
  Principle VI; the only sanctioned logging mechanism. CLI emits human progress to stderr
  via `tracing` macros; TUI emits per-iteration events to an in-process channel.
- `clap` (derive) — CLI argument parsing for the non-interactive binary.
- **`ratatui = "0.30.2"`** + `crossterm` backend — TUI rendering for the interactive
  binary (`leiden-tui`). The version is pinned to match `tasks.md:T006` (Constitution
  §VII). Ratatui is selected because it is the de facto Rust TUI framework, is
  pure-Rust on top of `crossterm`, plays well with `missing_docs = deny`, and is
  dependency-light enough to coexist with the strict lint profile.
- `serde` + `serde_json` — JSON partition output (FR-007b) and JSON graph input
  (FR-007a optional). Pinned via `cargo add`.
- `rand` — **omitted from v1.** The v1 algorithm is fully deterministic (no
  stochastic refinement) per `spec.md` Assumptions and Constitution Additional
  Constraints. The `--seed <U>` flag is accepted at the CLI for forward
  compatibility only and is forwarded to `RunResult.seed` without influencing
  the algorithm. If a stochastic variant is added later, it MUST be gated
  behind a Cargo feature flag and a Constitution amendment (per the
  Additional Constraints "Stochastic variants" clause).
- `proptest` — property-based tests for partition invariants (Constitution §V).
- `criterion` — micro-benchmarks for the inner loops (neighborhood scan, move
  computation, refinement merge) (Constitution §V).
- `cargo-deny` (CI-only advisory/license gate per constitution §VII).

**Storage**: Single-machine, in-memory (Constitution Assumptions). The graph is loaded
into a `CsrGraph` (compressed sparse row over dense `u32` indices). No on-disk cache;
no streaming/out-of-core. The user supplies node ids of any `Hash + Eq + Clone + Ord`
type (the canonical supertrait set per `spec.md` Clarifications 2026-08-30),
to dense `u32` indices at the input boundary.

**Testing**: `cargo test --workspace` for unit + integration; `proptest` for partition
invariants; `criterion` for inner-loop benchmarks; `cargo clippy --workspace --all-targets
-- -D warnings`; `cargo fmt --check`; `cargo doc --workspace --no-deps` (fails on
`missing_docs`); `cargo deny check`. TUI tests use `ratatui::backend::TestBackend` for
render assertions; no real-terminal dependency.

**Target Platform**: Linux/macOS/Windows where Rust stable runs. The library is portable
(no platform-specific syscalls); the CLI and TUI depend on `crossterm`, which supports
all three. Single-threaded by default. The `--threads N` CLI flag and
`Leiden::with_threads(NonZeroU32) -> Self` builder are accepted for forward
compatibility only and **do NOT consume the value in v1**: the algorithm runs
single-threaded, the stored value is recorded on `RunResult.threading` (which is
always `ThreadingPolicy::SingleThreaded` in v1 per `data-model.md §1.10`), and **no
`rayon` dependency is permitted in the workspace at v1**. The `parallel = []` Cargo
feature stub (declared in `tasks.md` T129) is a dormant forward-compatibility slot;
**activating it (default-on, dependencies-clause addition, or `cfg(feature =
"parallel")`-gated rayon import) requires a Constitution MAJOR amendment** per the
Additional Constraints "Stochastic variants" clause and the parallel-execution
assumption (`spec.md` §Assumptions). This keeps the deterministic SC-001 budget
valid (single modern CPU thread for the 100-node/500-edge fixture).

**Project Type**: Library crate (`leiden`) + two binaries (`leiden` CLI, `leiden-tui`).
Workspace layout: a single Cargo workspace with one library crate and two
binary crates (library-first per Constitution §I).

**Performance Goals**:
- SC-001: ≤ 5 s for a 100-node, 500-edge fixture on one modern CPU thread.
- Local-moving inner loop: neighborhood scan + delta computation cache-friendly via CSR
  storage.
- Quality computation: O(|E|) per partition, single `f64` accumulation, no per-iteration
  re-summation.
- Determinism: byte-identical partition output under fixed seed/γ.

**Constraints**:
- `unsafe_code = deny` workspace-wide.
- **MSRV floor (CRITICAL)**: ratatui 0.30.2 (pinned at T006) declares `rust_version = "1.88.0"` on crates.io. `rust-toolchain.toml` MUST pin a stable toolchain ≥ 1.88.0; the README's MSRV statement (T009) MUST reflect this floor. If a different ratatui version is substituted, its `rust_version` MUST be checked against the pinned toolchain before adoption. Per Constitution Additional Constraints "Language & edition", an MSRV bump requires a PATCH amendment to `.specify/memory/constitution.md`.
- No `unwrap`/`expect`/`panic`/`todo!`/`unimplemented!`/`dbg!` in production code.
- `missing_docs = deny` — every public item carries `///` docs; TUI widgets, key
  bindings, and color schemes documented for downstream maintenance.
- Numerical stability: `f64` only; modularity guards against `0/0` (empty graph) and
  `NaN` propagation. Quality outputs always finite or a typed error.
- Resolution γ ≤ 0 → typed error at the input boundary.
- TUI must not deadlock under `tracing` emission: the TUI consumes events from an
  `mpsc` channel rather than invoking the algorithm on the render thread.
  `tasks.md` resolves this dependency explicitly: **T111a** (`Leiden::with_event_sink`)
  is a Phase 6 (TUI) task, not Phase 7 — the implementation originally proposed as T125 has been moved to T111a to satisfy the T113 and T114
  dependencies; T125 has been removed from `tasks.md`; the renumbering to T111a is purely for the T113/T114 dependency resolution (T125's slot is left empty rather than marked DEPRECATED to avoid ghost task IDs). T113 and T114 list `(depends on T111a)`. **T125 status note**: T125 is intentionally absent from `tasks.md` (not marked DEPRECATED) to avoid a ghost task ID; implementers who encounter "T125" in older documents or external references should consult `tasks.md §Phase 6` for the current task ID (T111a) and the T113/T114 dependency rationale above.

**Scale/Scope**: ≤ ~100k nodes / ~1M edges in-memory per the spec Assumptions. Massive-
scale distributed Leiden is out of scope for v1.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Pre-design (before Phase 0)**: see columns below.

**Post-design (after Phase 1)**: see "Post-Phase-1" column. All gates continue
to pass; the design introduced no new principle violations.

| Principle | Pre-design | Post-Phase-1 | Notes |
|---|---|---|---|
| I. Library-First & Domain Modeling | PASS | PASS | Library-first verified: `data-model.md` defines all domain types in the `leiden` crate; both binaries consume them. No algorithm code in `leiden-cli` or `leiden-tui`. |
| II. Strict Lint Compliance | PASS | PASS | `[workspace.lints]` block from `rust-code-rigor.md` copied verbatim. Ratatui 0.30.2 is `#![forbid(unsafe_code)]`; our call paths are `unwrap`-free. |
| III. Panic-Free Error Propagation | PASS | PASS | `LeidenError` (library) and `CliError` (CLI) cover every fallible op. No `From<io::Error>` blanket at library level; CLI's `Io` variant provides the bridge. |
| IV. Documentation & Visibility Discipline | PASS | PASS | `data-model.md §1` and `contracts/library-api.md` document every public item. TUI widget builders, key bindings, and the `LogPaneLayer` custom tracing layer all carry `///` docs. |
| V. Test-First (NON-NEGOTIABLE) | PASS | PASS | `quickstart.md §3-§9` enumerate the failing-test commits that must precede each implementation commit: parse, single-node, empty-graph, two-cliques, fixture suite, determinism, resolution, library smoke, CLI round-trip, malformed inputs, TUI snapshots, proptest invariants, criterion budgets. |
| VI. Observability & I/O Discipline | PASS | PASS | FR-010 satisfied: every phase emits a `LeidenEvent` (T019, T124). `tracing` is the sole logging mechanism (T092–T094); `println!`/`eprintln!`/`dbg!` forbidden by `print_stdout = warn` + `dbg_macro = deny` in the lint block. CLI uses `writeln!(stdout.lock())` for partition output (T091). Bounded-channel overflow path covered by T113a (`LeidenEvent::Throttled`) and T113b (`Sender::send` failure logging) per Constitution §III. `contracts/tui-events.md §4` confirms a single tracing source of truth: stderr when not a TTY, file via `--log-file`, in-memory ring buffer for the TUI log pane. |
| VII. Dependency & Build Rigor | PASS | PASS | All deps pinned in `research.md §4`; tasks T004–T006 use `cargo add` to enforce semver ranges. No wildcards. `multiple_crate_versions = deny` enforced; no transitive duplicates expected from the listed deps. Ratatui pinned to `0.30.2` per T006 to match `Constitution §VII`; `cargo deny check` config is introduced in T127 (Phase 7) and exercised by T139's pre-merge gate, satisfying §VII at release-cut time rather than at design time. |
| Additional: Domain Accuracy | PASS | PASS | Algorithm follows Traag et al. 2019 (Algorithm A.2); deterministic variant per spec. |
| Additional: Determinism | PASS | PASS | Tie-breaking by lowest node id documented in `data-model.md §1.4` and `quickstart.md §4`. |
| Additional: Numerical Stability | PASS | PASS | `0/0` guard in modularity; property test in `quickstart.md §8` asserts no NaN over 1000 random graphs. |
| Additional: Unsafe Code | PASS | PASS | `unsafe_code = deny` workspace-wide. Ratatui is itself `#![forbid(unsafe_code)]`. |

No unjustified violations. Both pre-design and post-Phase-1 gates pass.

## Project Structure

### Documentation (this feature)

```text
specs/001-leiden-algorithm/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
│   ├── library-api.md   # Public library API contract
│   ├── cli-schema.md    # CLI flag and output schema contract
│   └── tui-events.md    # TUI event-channel and key-binding contract
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)

```text
leiden/                          # Workspace root (crate name: leiden)
├── Cargo.toml                   # [workspace] + [workspace.lints] verbatim from rust-code-rigor.md
├── rust-toolchain.toml          # Pin toolchain
├── README.md                    # MSRV, scope, citations
├── crates/
│   ├── leiden/                  # Library crate (the algorithm)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs           # Public re-exports, top-level docs
│   │   │   ├── graph/           # CsrGraph, NodeId trait, edge model
│   │   │   ├── partition/       # Partition, Community, refinement
│   │   │   ├── quality/         # Modularity, QualityFunction trait
│   │   │   ├── local_moving/    # Greedy local-moving phase
│   │   │   ├── refinement/      # Refinement phase (Traag et al. §3)
│   │   │   ├── aggregation/     # Aggregate graph construction
│   │   │   ├── orchestrator/    # Outer loop, termination, run-result
│   │   │   ├── events.rs        # LeidenEvent (for tracing + TUI channel)
│   │   │   └── error.rs         # thiserror enums
│   │   ├── tests/               # Integration tests, proptest, fixtures
│   │   └── benches/             # criterion benches
│   ├── leiden-cli/              # Non-interactive CLI binary
│   │   ├── Cargo.toml
│   │   ├── src/main.rs          # clap parsing + parse → run → serialize
│   │   ├── src/parse/           # Edge-list + JSON input parsers (FR-007a)
│   │   ├── src/format/          # JSON + tab-separated text output (FR-007b)
│   │   └── tests/
│   └── leiden-tui/              # Interactive Ratatui binary
│       ├── Cargo.toml
│       ├── src/main.rs          # Entry; sets up terminal, channel, worker
│       ├── src/app.rs           # App state, message enum
│       ├── src/ui/              # Ratatui widgets (graph view, community panel, log pane)
│       ├── src/event.rs         # mpsc::Receiver<LeidenEvent>, key map
│       ├── src/worker.rs        # Spawns the orchestrator, forwards events (canonical name per tasks.md T111)
│       └── tests/               # TestBackend snapshot tests
└── fixtures/                    # Curated reference graphs for tests/benches
```

**Structure Decision**: A single Cargo workspace with one library crate (`leiden`)
and two binary crates (`leiden-cli`, `leiden-tui`). The library-first decomposition
(Principle I) puts every algorithm seam behind a clean facade; both binaries are
thin adapters (parse → run → render/serialize). `Ratatui` is restricted to the
`leiden-tui` crate, which keeps the library free of TUI dependencies and prevents
the strict lint profile from being weakened by an interactive UI.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| (none) | — | — |

No constitution violations. The TUI is an additional binary, not an additional
principle. All constitution principles hold for both binaries.