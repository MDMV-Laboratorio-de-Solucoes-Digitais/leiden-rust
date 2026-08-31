# Observability & Tracing Checklist: Leiden Algorithm in Rust

**Purpose**: Reviewer-owned requirements-quality review for the observability and tracing
surface of the 001-leiden-algorithm feature. This is "unit tests for English" — it validates
whether the *observability requirements themselves* are well-written, complete, unambiguous,
and ready for implementation. It does NOT verify implementation.
**Created**: 2026-08-30
**Feature**: [spec.md](../spec.md) §FR-010, [plan.md](../plan.md) §Technical Context / §VI,
[research.md](../research.md) §2.3, [data-model.md](../data-model.md) §1.8, §1.9,
[contracts/tui-events.md](../contracts/tui-events.md), [contracts/cli-schema.md](../contracts/cli-schema.md) §1.5,
[contracts/library-api.md](../contracts/library-api.md) §6

**Review Ownership**: Mark an item `[x]` only when the reviewer determines the
requirements-quality criterion is satisfied. `[x]` does not mean implementation work is
complete.

**Marker Semantics**: `[x]` = reviewer-approved requirements quality. `$speckit-implement`
reads checkbox state as a gate and MUST NOT modify markers.

**Scope**: This checklist complements `requirements-quality.md` (which covers general
requirements quality). It drills into observability, tracing, and event-channel concerns
that overlap multiple artifacts (library, CLI, TUI). Each item is an *English*-level test
of the spec/plan/contracts/data-model — not an implementation test.

---

## LeidenEvent Taxonomy — Completeness

- [ ] CHK073 — Are every LeidenEvent variant's required fields (per data-model §1.8) explicitly enumerated in the spec/contracts, with types and value ranges? [Completeness, Spec §FR-010, Data Model §1.8, TUI Events §3]
- [ ] CHK074 — Is the spec explicit about which LeidenEvent variants carry a `phase: Phase` field vs. which carry an `iteration: u32` field (note: `IterationStarted` uses `Phase` but `LocalMovingProgress` uses bare `u32` per data-model §1.8)? [Clarity, Data Model §1.8, Gap]
- [ ] CHK075 — Is the spec explicit about the `LocalMovingDelta { iteration, delta_q }` vs. `QualityComputed { iteration, quality }` distinction — i.e., is `LocalMovingDelta` per-move and `QualityComputed` per-iteration-total? [Clarity, Spec §FR-010, Gap]
- [ ] CHK076 — Is the spec explicit about the `RefinementMerged { from, to }` field naming (community ids, node ids, or internal indices)? [Ambiguity, Data Model §1.8, Gap]
- [ ] CHK077 — Is the spec explicit about whether `Throttled { dropped: u64 }` (TUI Events §2) is part of the public library event taxonomy or only an internal channel-side effect? [Completeness, TUI Events §2, Data Model §1.8, Gap]
- [ ] CHK078 — Is there a defined `Error` / `Failed { error: LeidenError }` LeidenEvent variant for propagating errors to the TUI, or is error visibility only via `tracing::error!` + terminal exit? [Gap, Spec §FR-010, Gap]
- [ ] CHK079 — Are every variant's payload values (`nodes: usize`, `quality: f64`, etc.) quantified with allowed ranges (e.g., is `quality: f64` guaranteed finite; can `total_weight: f64` be 0 for empty graphs)? [Clarity, Data Model §1.8, Spec §FR-005]
- [ ] CHK080 — Is the spec explicit that `LeidenEvent` variants must be `#[derive(Debug, Clone)]` per Constitution §IV and library-api §6? [Consistency, Library API §6, Constitution §IV]
- [ ] CHK081 — Is the spec explicit about the ordering guarantee of `LeidenEvent` emissions — i.e., must `GraphLoaded` precede the first `IterationStarted`, and must the final `IterationFinished` precede `Terminated`? [Completeness, Spec §FR-010, Gap]

## LeidenEvent — Phase Coverage & Traceability

- [ ] CHK082 — Is every algorithmic phase (graph load, local moving, refinement, aggregation, termination) covered by at least one named LeidenEvent variant? [Coverage, Spec §FR-010, Data Model §1.8]
- [ ] CHK083 — Is there a LeidenEvent variant emitted at phase boundaries (`IterationStarted` / `IterationFinished`) so consumers can correlate phase progress? [Coverage, Spec §FR-010, Data Model §1.8]
- [ ] CHK084 — Is there a LeidenEvent variant for the `DegenerateInput` early-exit path so the TUI can transition to `Error` state without polling? [Coverage, Data Model §1.9 §2, Gap]
- [ ] CHK085 — Is every LeidenEvent variant traceable to a specific FR (FR-009 algorithm phase, FR-010 observability) with no orphan variants? [Traceability, Spec §FR-009, §FR-010, Data Model §1.8]
- [ ] CHK086 — Is the Phase enum (`LocalMoving` / `Refinement` / `Aggregation`) exhaustive over the algorithm's phases, or is `GraphLoad` missing? [Coverage, Data Model §1.8, Gap]
- [ ] CHK087 — Is the spec explicit about whether `LeidenEvent::Aggregation` fires once per iteration (after the aggregation step) or once per outer loop (only at termination)? [Clarity, Spec §FR-010, Data Model §1.8, Gap]

## Tracing Subscriber Setup — Requirements

- [ ] CHK088 — Is the `tracing_subscriber::registry().with(env_filter).with(fmt_layer).with(log_pane_layer).init()` composition (TUI Events §4) explicitly required, with a documented precedence order for env filter vs. `--log-level`? [Completeness, TUI Events §4, CLI Schema §1.2]
- [ ] CHK089 — Is the `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))` precedence (env wins, flag fallback) documented as a requirement rather than an implementation choice? [Clarity, TUI Events §4, Spec §Edge Cases, Gap]
- [ ] CHK090 — Is the requirement that `fmt_layer` is gated on `is_terminal() == false` so ratatui's terminal isn't disturbed, documented as a hard constraint? [Clarity, TUI Events §4, Spec §FR-010]
- [ ] CHK091 — Is the requirement that `tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_ansi(false)` is the only sanctioned stderr writer — i.e., no parallel `eprintln!`/`println!` per Constitution §VI — stated in the spec or contracts? [Consistency, TUI Events §4, Constitution §VI]
- [ ] CHK092 — Is the requirement that the library owns the global `tracing-subscriber` (TUI Events §1) and the TUI only attaches an additional `Layer<S>` documented as a hard contract? [Clarity, TUI Events §1, Gap]
- [ ] CHK093 — Is the requirement that `tracing` events are emitted BEFORE `LeidenEvent` is sent over the channel (so tracing is the source of truth, not the channel) made explicit? [Clarity, Library API §6, TUI Events §1, Gap]
- [ ] CHK094 — Is the requirement that `Sender::send` returns `Result` and the worker logs `tracing::warn!` and continues on channel error documented as a hard contract (TUI Events §2)? [Completeness, TUI Events §2, Library API §6]

## Tracing Levels & Structured Fields

- [ ] CHK095 — Is the spec explicit about which tracing level (`trace`/`debug`/`info`/`warn`/`error`) is used for each phase event in FR-010? [Clarity, Spec §FR-010, Gap]
- [ ] CHK096 — Is the spec explicit that tracing events use **structured fields** (e.g., `info!(node_id = id, "moved node")`) rather than stringly-typed log lines, per Constitution §VI? [Clarity, Spec §FR-010, Constitution §VI]
- [ ] CHK097 — Are the structured-field keys for each LeidenEvent variant standardized across artifacts — e.g., does `LocalMovingProgress` use `iteration` (u32) and `moved_nodes` (u32) consistently in spec, data-model, and contracts? [Consistency, Spec §FR-010, Data Model §1.8]
- [ ] CHK098 — Is the spec explicit that `f64` fields in tracing events are emitted with sufficient precision (no `{:?}` truncation that would hide numerical-stability bugs)? [Clarity, Gap, Constitution §VI]
- [ ] CHK099 — Is the spec explicit about the default `--log-level` (CLI Schema §1.2 says `info`) and the rationale (`info` for human-readable progress, `debug`/`trace` reserved for diagnostics)? [Clarity, CLI Schema §1.2, Gap]

## Channel Topology & Back-Pressure

- [ ] CHK100 — Is the channel topology diagram (TUI Events §1: orchestrator → mpsc → render thread) documented as a hard contract for v1, with no other topology (e.g., `crossbeam`, `tokio::mpsc`) allowed without a contract amendment? [Completeness, TUI Events §1]
- [ ] CHK101 — Is the bounded channel size (1024 per TUI Events §2) justified — i.e., is 1024 proven sufficient for the 100-node/500-edge SC-001 fixture, or is the bound ad-hoc? [Measurability, TUI Events §2, Spec §SC-001, Gap]
- [ ] CHK102 — Is the requirement that a full channel results in `LeidenEvent::Throttled { dropped: u64 }` being sent, then further events dropped (TUI Events §2), documented as the exact back-pressure policy — i.e., no retry, no blocking, no panic? [Completeness, TUI Events §2]
- [ ] CHK103 — Is the requirement that the worker thread NEVER blocks on `Sender::send` (i.e., never uses the synchronous `send` directly, must use `try_send` if the bound is exceeded) made explicit? [Completeness, TUI Events §2, Constitution §III]
- [ ] CHK104 — Is the requirement that `Throttled { dropped: u64 }` is rate-limited (not emitted on every dropped event, which would itself cause back-pressure) specified? [Clarity, TUI Events §2, Gap]
- [ ] CHK105 — Is the requirement that the drain loop uses `rx.try_recv()` not `rx.recv()` (TUI Events §2) documented as a hard contract — i.e., the render thread MUST NOT block on the worker? [Completeness, TUI Events §2]
- [ ] CHK106 — Is the requirement that the TUI consumes events on every tick (not at arbitrary time points) documented to prevent event-queue growth? [Completeness, TUI Events §2, Gap]

## Log Pane — Ring Buffer & Display

- [ ] CHK107 — Is the ring-buffer size (500 entries per TUI Events §1, §5) quantified as a hard contract with a documented eviction policy (FIFO)? [Clarity, TUI Events §1, §5, CLI Schema §2.4]
- [ ] CHK108 — Is the requirement that the ring buffer holds `String` formatted lines (not raw `tracing::Event`s) documented as a contract — i.e., the consumer never sees a raw event? [Clarity, TUI Events §5, Gap]
- [ ] CHK109 — Is the format string `"[{}] {}: {}"` (TUI Events §5) specified as the canonical log-pane line format, with the field-syntax (`{field:?}={value:?}`) documented? [Clarity, TUI Events §5, Gap]
- [ ] CHK110 — Is the requirement that `LogPaneLayer::on_event` is panic-free (the `let _ = write!` patterns in TUI Events §5) stated as a hard contract under Constitution §III? [Consistency, TUI Events §5, Constitution §III]
- [ ] CHK111 — Is the requirement that the log pane uses an `Arc<Mutex<RingBuffer<String>>>` (single shared sink) vs. per-consumer sinks documented as the threading model? [Completeness, TUI Events §5, Gap]
- [ ] CHK112 — Is the requirement that `LogPaneLayer` is `pub` for testability (insta snapshot tests per TUI Events §6) but not exposed in the public library API documented? [Consistency, TUI Events §5, §6, Gap]

## CLI Observability — stderr / file

- [ ] CHK113 — Are the CLI's three stderr lines (start, per-iteration, final per CLI Schema §1.5) explicitly required and format-quantified? [Completeness, CLI Schema §1.5, Spec §FR-007]
- [ ] CHK114 — Is the requirement that CLI stderr output uses `tracing` (not `eprintln!`/`println!`) per Constitution §VI documented as a hard contract? [Consistency, CLI Schema §1.5, Constitution §VI]
- [ ] CHK115 — Is the requirement that `--log-file <PATH>` produces JSON-lines (one tracing event per line, machine-parseable) rather than the default `fmt::layer()` text format documented? [Clarity, TUI Events §4, CLI Schema §1.2, Gap]
- [ ] CHK116 — Is the behavior defined when `--log-file` cannot be created (permission denied, disk full, path is a directory) — typed error, fallback to stderr, or silent skip? [Coverage, CLI Schema §1.2, Gap]
- [ ] CHK117 — Is the behavior defined when stderr is closed by the consumer (broken pipe) mid-run — does the library keep emitting `tracing` events to the closed handle? [Coverage, Gap, Constitution §VI]
- [ ] CHK118 — Is the behavior defined when stdout is closed by the consumer (broken pipe) before the partition is fully written — exit code, error message, partial write? [Coverage, Spec §FR-007, Gap]
- [ ] CHK119 — Is the requirement that the CLI does NOT register an `mpsc::Sender<LeidenEvent>` (it observes events via global subscriber per Library API §6) documented as a hard contract? [Consistency, Library API §6, TUI Events §1]

## TUI Event Handling

- [ ] CHK120 — Is the requirement that the worker thread is spawned via `std::thread::spawn` (not `tokio::spawn` or `rayon::spawn`) documented as a hard contract? [Clarity, TUI Events §2, Gap]
- [ ] CHK121 — Is the requirement that `Leiden::with_event_sink` is the canonical way for a binary to register as a consumer, and that this is the ONLY public hook for event delivery, documented? [Completeness, Library API §6, Gap]
- [ ] CHK122 — Is the requirement that the worker thread terminates when the orchestrator returns (no manual `JoinHandle::join` in the TUI) specified? [Clarity, TUI Events §7, Gap]
- [ ] CHK123 — Is the requirement that `Terminated` is the LAST event sent before the worker thread exits specified as a contract — i.e., no events after `Terminated`? [Completeness, TUI Events §7, Spec §FR-010]
- [ ] CHK124 — Is the requirement that `AppState` transitions are driven by `LeidenEvent` variants (`IterationFinished` → progress, `Terminated` → Done, etc.) enumerated for every variant? [Coverage, CLI Schema §2.5, TUI Events §3, Gap]
- [ ] CHK125 — Is the requirement that the TUI's main thread receives `Terminated` and transitions to `AppState::Done` (or `AppState::Error` on `LeidenError`) documented as the exact transition rule? [Completeness, TUI Events §7, CLI Schema §2.5]

## Snapshot & Test Contracts

- [ ] CHK126 — Is the `ratatui::backend::TestBackend` snapshot test contract (TUI Events §6) explicitly required as the test strategy for the TUI, with `insta::assert_snapshot!` as the assertion macro? [Completeness, TUI Events §6, Spec §FR-010]
- [ ] CHK127 — Is the requirement that snapshot tests live in `crates/leiden-tui/tests/snapshots/` and are updated via `cargo insta review` documented? [Completeness, TUI Events §6]
- [ ] CHK128 — Is the requirement that at least one snapshot test exercises the `Running` state with a populated `events` buffer (TUI Events §6 example) documented as a coverage gate? [Coverage, TUI Events §6]
- [ ] CHK129 — Is the requirement that tracing events are tested via `tracing-test` or `tracing_subscriber::fmt::TestWriter` (i.e., library events can be captured in tests without a real subscriber) documented? [Gap, Spec §FR-010]
- [ ] CHK130 — Is the requirement that `cargo deny check` and `cargo doc --workspace --no-deps` fail on missing docs for the `LogPaneLayer` and event types documented as part of the CI gate (Constitution §VII, Development Workflow)? [Consistency, TUI Events §5, Constitution §VII]

## Error Visibility & Tracing-error Integration

- [ ] CHK131 — Is the requirement that `LeidenError` variants are emitted via `tracing::error!` with structured fields (e.g., `error = ?err, path = %path`) documented as a hard contract under Constitution §VI? [Consistency, Data Model §1.11, Constitution §VI, Gap]
- [ ] CHK132 — Is the requirement that `CliError::Leiden(#[from] LeidenError)` (Data Model §3.3) preserves the underlying `LeidenError` context for `tracing` emission documented? [Consistency, Data Model §3.3, Gap]
- [ ] CHK133 — Is the requirement that on a `LeidenError`, the library emits `tracing::error!` BEFORE returning the error (so the structured log line appears before the process exits) documented? [Clarity, Gap]
- [ ] CHK134 — Is the behavior defined when `tracing` emission itself fails (e.g., the `LogPaneLayer`'s `Arc<Mutex<...>>` is poisoned) — silent skip, fallback to stderr, or hard error? [Coverage, Gap]

## End-of-Run & Cleanup

- [ ] CHK135 — Is the four-step end-of-run sequence (worker JoinHandle dropped, TUI receives `Terminated`, `ratatui::restore()` called, subscriber dropped) documented as the exact cleanup order in TUI Events §7? [Completeness, TUI Events §7]
- [ ] CHK136 — Is the requirement that `fmt_layer` resumes writing to stderr after `ratatui::restore()` (TUI Events §4: "when the TUI exits, fmt_layer resumes") explicitly documented with the rationale? [Clarity, TUI Events §4, §7]
- [ ] CHK137 — Is the requirement that `ratatui::init()` installs a panic hook that restores the terminal (Research §2.2) documented as a recovery path, and is the requirement that the library does NOT depend on this panic hook (library is `panic`-free) made explicit? [Consistency, Research §2.2, Constitution §III]
- [ ] CHK138 — Is the behavior defined when `ratatui::init()` fails (terminal already in use, no TTY available) — does the TUI emit a structured error before exiting? [Coverage, Gap, CLI Schema §2.5]

## Cross-Artifact Consistency

- [ ] CHK139 — Does the spec's FR-010 ("structured observability events for every major phase") align with data-model §1.8's LeidenEvent variant list — i.e., is every FR-010 phase covered by exactly one variant? [Consistency, Spec §FR-010, Data Model §1.8]
- [ ] CHK140 — Does the spec's FR-010 align with TUI Events §3's LeidenEvent variant list — i.e., no orphan variants in the contract that aren't in the data model? [Consistency, Spec §FR-010, TUI Events §3, Data Model §1.8]
- [ ] CHK141 — Does the CLI's stderr progress format (CLI Schema §1.5) align with the library's `tracing` events — i.e., is every CLI progress line traceable to a specific LeidenEvent variant? [Traceability, CLI Schema §1.5, Data Model §1.8]
- [ ] CHK142 — Does the TUI's panel rendering (community panel from `IterationFinished`, status bar from `Terminated`, TUI Events §3) align with the data-model §1.8 variant payloads — i.e., does every panel-rendered field exist on the corresponding variant? [Traceability, TUI Events §3, Data Model §1.8]

## Non-Functional Observability Requirements

- [ ] CHK143 — Is observability overhead quantified — i.e., does emitting all LeidenEvent variants in SC-001 (5 s budget for 100-node/500-edge) keep the run within budget? [Measurability, Spec §SC-001, Spec §FR-010, Gap]
- [ ] CHK144 — Is the requirement that observability is off by default in release builds (i.e., no `#[cfg(debug_assertions)]` gates that flip tracing off in release) clarified — i.e., does `tracing` use compile-time filtering to keep release overhead low? [Clarity, Constitution §VI, Gap]
- [ ] CHK145 — Is the requirement that the library does NOT depend on a specific `tracing-subscriber` version (only `tracing` itself) documented to keep the consumer's subscriber choice free? [Dependency, Library API §6, Research §2.3, Gap]
- [ ] CHK146 — Is the requirement that the library never panics on a closed or full `mpsc::Sender` (Library API §6: "the library never panics on a closed or full channel") documented as a hard contract? [Completeness, Library API §6, Constitution §III]

## Ambiguities & Open Questions

- [ ] CHK147 — Is the term "phase" (used in spec §FR-010, data-model §1.8 Phase enum, and TUI Events §3) consistent across all artifacts — i.e., does "phase" mean (a) the three Leiden algorithm phases (local moving / refinement / aggregation), (b) the outer-loop iteration, or (c) graph load + per-iteration phases? [Ambiguity, Spec §FR-010, Data Model §1.8, TUI Events §3]
- [ ] CHK148 — Is the conflict between "the library owns the global `tracing-subscriber`" (TUI Events §1) and the binary's need to install its own subscriber (Library API §6, "the CLI does not [register an event sink]") resolved — i.e., who owns the subscriber, and can a downstream library user override it? [Conflict, TUI Events §1, Library API §6, Gap]
- [ ] CHK149 — Is the term "structured observability events" (FR-010) clarified with concrete examples — e.g., is `info!(iteration = 3, phase = "local_moving", "started")` the canonical pattern, or is a richer schema required? [Ambiguity, Spec §FR-010, Gap]
- [ ] CHK150 — Is the conflict between "ring buffer holds 500 entries" (TUI Events §1, §5) and the spec's silence on buffer size resolved — i.e., is 500 a hard contract or an implementation default that the spec leaves to the data model? [Conflict, TUI Events §1, §5, Spec §FR-010, Gap]

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied.
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation.
- `$speckit-implement` reads checklist checkbox state as a gate and must not modify markers.
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `$speckit-specify` and `$speckit-clarify`.
- Items are numbered sequentially, continuing from `requirements-quality.md` (CHK073..CHK150 here).
- This checklist is reviewer-owned and complements — does not replace — `requirements-quality.md`.
- Add reviewer comments inline above the relevant item; do not delete items.