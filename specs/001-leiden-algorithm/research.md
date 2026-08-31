# Research: Leiden Algorithm in Rust

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30
**Inputs**: `spec.md`, `constitution.md`, `guide-to-strict-rust.md`, `rust-code-rigor.md`
**Output of**: `$speckit-plan` Phase 0 (Outline & Research)

This document captures the technical decisions made before design. It is the
authoritative reference for the data model, contracts, and implementation tasks.

---

## 1. Algorithm — Deterministic Leiden

### 1.1 Decision

Implement Leiden per Traag, Waltman, van Eck (2019) — *From Louvain to Leiden:
guaranteeing well-connected communities* (Sci. Rep. 9:5233, arXiv:1810.08473).
Deterministic variant only (Constitution Additional Constraints). Stochastic
refinement is bypassed by deterministic tie-breaking (lowest node id).

### 1.2 Rationale

The spec mandates (FR-009) faithful adherence to the published algorithm. The
constitution (Additional Constraints) requires deterministic tie-breaking and
bans stochastic variants. The three-phase structure (local moving → refinement →
aggregation) is the paper's Algorithm A.2 (lines 33–48) and is preserved verbatim.

### 1.3 Key Formulas (cited)

| Concept | Expression | Source |
|---|---|---|
| Modularity | `Q(P) = (1 / 2m) · Σ_c [ e_c − γ · (Σ_tot(c))² / (2m) ]` | Traag 2019, Eq. (1) |
| Local-moving ΔQ (move node `i` from `C` to `T`) | `ΔQ = [ Σ_in(T, i) − Σ_in(C, i) ] / m − γ · kᵢ · [ Σ_tot(T) − Σ_tot(C) + kᵢ ] / (2 m²)` | Traag 2019, Eq. (A5); cross-validated with `leiden-rs` `quality.rs` L41–L51 |
| Refinement merge predicate | `Σ_in(D, C \ D) ≥ k_D · (k_C − k_D)` | Traag 2019, Algorithm A.2 line 37 |
| Aggregate edge weight | `w(A, B) = Σ_{u∈A, v∈B} A_uv` | Traag 2019, Algorithm A.2 lines 44–48 |
| Termination (stable iteration) | `local_moving_changed == false ∧ refined_partition == local_moved_partition` | Traag 2019, §III.A; matches `leiden-rs::run_core` |

### 1.4 Numerical stability

- `m = 0` (empty graph): guard returns `0.0` for both total quality and delta.
  No division by `m` or `2m²` ever occurs.
- `Σ_in = 0`: handled by the formula (no `1/Σ_in` term).
- `Σ_tot` accumulation: `f64` only; capacity easily handles ≤ 100k nodes / 1M edges
  within `f64`'s 2⁵³ integer-exact range. Internal indices stay `u32`.
- `NaN` / `±∞`: validate `γ.is_finite()` and `m.is_finite()` at boundaries;
  defensively coerce any NaN result to `0.0`. Property test asserts no NaN
  in returned quality.
- Self-loops: contribute once to `kᵢ` and once to `e_c`; the `+ kᵢ` term in the
  delta compensates.
- Tie epsilon: two candidates within `f64::EPSILON` are treated as equal;
  pick the lower community id.

### 1.5 Module decomposition

Mirrors the algorithm's natural seams and `leiden-rs`'s module structure:

```
graph/         CsrGraph, NodeId trait, edge model
partition/     Partition, Community, refinement predicates
quality/       Modularity, QualityFunction trait
local_moving/  Greedy move with Σ_tot incremental update
refinement/    MergeNodesSubset, deterministic refinement
aggregation/   Build aggregate graph from refined partition
orchestrator/  Outer loop, termination, run-result
events.rs      LeidenEvent (tracing + TUI channel)
error.rs       LeidenError (thiserror enum)
```

A shared `MoveComponents { k_i, sigma_in_to_target, sigma_tot_target, sigma_in_from_current, sigma_tot_current }`
struct lets every phase compute deltas without recomputing per-phase sums
(paper's Section III fast-local-move optimisation).

### 1.6 Alternatives considered

| Alternative | Rejected because |
|---|---|
| Stochastic refinement (paper default) | Spec mandates determinism; bypass via lowest-id tie-break. |
| CPM quality default | Spec says modularity. Trait designed so CPM is a one-impl addition later. |
| HashMap-per-node adjacency | 10–100× slower than CSR; cache-unfriendly. |
| Parallel local-moving (coloring/rayon) | Non-deterministic across runs; SC-001 budget already met sequentially. |
| Re-running `total_quality` per move (O(n²)) | `leiden-rs` shows O(degree) is achievable. |
| Iteration-cap-only termination | Spec wants both convergence and cap (FR-003a). |
| Per-iteration community renumbering | Adopted — keeps community ids dense. |

---

## 2. Ratatui — Interactive TUI

### 2.1 Decision

Adopt **Ratatui 0.30.2** (released 2026-06-19) with the default `crossterm 0.29`
backend, in a dedicated `leiden-tui` binary crate. Use `ratatui::init()` /
`ratatui::restore()` (not `ratatui::run()`) for explicit error propagation; the
worker thread owns the `mpsc::Sender<LeidenEvent>` and the main thread drains
the receiver on each tick via `try_recv()`.

### 2.2 Rationale

- Ratatui 0.30.2 is current stable; `#![forbid(unsafe_code)]` since 0.23; the
  public API we touch is panic-free and `unwrap`-free on our call paths.
- `ratatui::init()` installs a panic hook that restores the terminal on panic
  — satisfies the constitution's panic-free mandate without manual unsafe.
- `ratatui::backend::TestBackend` enables snapshot-based TUI tests without
  touching a real terminal.
- `crossterm 0.29` is the documented default backend and supports Linux/macOS/Windows.
- Restricting Ratatui to the `leiden-tui` crate keeps the library (`leiden`)
  free of TUI dependencies and prevents the strict lint profile from being
  weakened by an interactive UI.

### 2.3 Tracing coexistence

The constitution forbids `println!`/`eprintln!` (Principle VI). Ratatui
diagnostics come from `tracing` only:

1. **File log** — `tracing-subscriber::fmt::layer().with_writer(file)` writes
   full structured events to a file (`--log-file` flag) for post-mortem.
2. **In-TUI log pane** — a 30-line custom `tracing_subscriber::Layer<S>` that
   pushes formatted events into an `Arc<Mutex<RingBuffer<String>>>` consumed by
   `log_pane.rs`. Keeps a single `tracing` source of truth.
3. **Stderr progress** — `tracing-subscriber::fmt::layer().with_writer(stderr)`
   when stdout/stderr are detached (CI mode); suppressed in interactive mode.

`tui-logger` was rejected because it introduces a parallel `log`-facade layer.

### 2.4 Project layout (`leiden-tui`)

```
crates/leiden-tui/src/
├── main.rs       // color_eyre::install → logging::init → ratatui::init → app.run → ratatui::restore
├── app.rs        // App struct, AppState enum, tick + handle_key
├── worker.rs     // spawns the orchestrator, owns Sender<LeidenEvent>
├── event.rs      // mpsc::Receiver<LeidenEvent>, key bindings (q/r/l/g/s/?)
├── logging.rs    // tracing-subscriber setup + in-memory Layer for log pane
└── ui/
    ├── mod.rs        // fn render(frame, app)
    ├── community.rs  // Table widget — community list
    ├── graph.rs      // Canvas widget — graph view (BFS-laid-out)
    └── log_pane.rs   // Paragraph widget — scrolling in-memory tracing buffer
```

### 2.5 Alternatives considered

| Alternative | Rejected because |
|---|---|
| `tui-rs` | Abandoned; Ratatui is its successor. |
| `termion` / `termwiz` backends | Unix-only / heavier deps. Crossterm is the documented default. |
| `ratatui::run()` one-shot | Returns no error from closure to outside; loses error propagation. |
| `tui-logger` widget | Parallel `log`-facade layer; a 30-line custom `Layer` is cleaner. |
| Manual `unsafe` raw-mode handling | Forbidden by Ratatui policy and unnecessary since `init()`. |
| `color-eyre`/`anyhow` library-wide | Only used at `main` boundary; library stays `thiserror`-only. |

---

## 3. CLI I/O Contracts

### 3.1 Decision

- `clap` v4 **derive** for CLI argument parsing.
- `serde_json` (not `simd-json`) for JSON I/O.
- Input parsers live in `leiden-cli`, not in the `leiden` library.
- Format dispatch: extension hint + first-non-whitespace byte sniff.
- Progress via `tracing-subscriber` (stderr); partition via `writeln!(stdout.lock())`.

### 3.2 Rationale

- **clap derive** is the documented default for static arg sets; compile-time
  typed errors; smaller boilerplate than the builder API.
- **serde_json**: small partitions (< 100 KB) don't benefit from SIMD; the
  ergonomics and stable error types dominate. `simd-json` requires mutable
  buffers and gives weaker derive support.
- **Parsers in the binary**: keeps the library free of file-format coupling;
  `CsrGraph::from_edges(impl IntoIterator<Item=(Id, Id, f64)>)` is the only
  library API the CLI calls into for graph construction.
- **Byte-sniff > extension-only**: handles `.json.bak`, `.txt.gz`, and stdin
  pipes. One 4 KiB read is cheap.
- **`tracing` for progress**: satisfies Constitution Principle VI; integrates
  with the TUI's in-memory layer (single source of truth).

### 3.3 Format dispatch algorithm

```text
dispatch(path):
    if path.extension() == "json": return Json   // fast hint
    first = peek_first_non_whitespace(path)
    if first == b'{': return Json
    return EdgeList
```

### 3.4 Alternatives considered

| Alternative | Rejected because |
|---|---|
| `simd-json` for output | Marginal gain on small payloads; weaker derive ergonomics. |
| `clap` builder API | More verbose for static arg sets; no compile-time type checking. |
| `argh` | Lighter but no derive→typed-error integration; smaller ecosystem. |
| Extension-only dispatch | Misclassifies `.json.bak`, `.txt.gz`, stdin pipes. |
| `println!` for partition | Violates Principle VI; breaks shell pipelines. |
| Parsers in the library | Couples graph crate to file formats; bloats dep tree for consumers. |

---

## 4. Crate Layout — Final Decision

A single Cargo workspace with three crates:

| Crate | Purpose | Key deps |
|---|---|---|
| `leiden` (library) | All algorithm logic, domain types, events, error | `thiserror`, `tracing`, `rand`, `serde` (partition output), `proptest`, `criterion` |
| `leiden-cli` (binary) | Edge-list + JSON input → run → JSON or text partition output | `clap`, `serde_json`, `tracing-subscriber`, `thiserror` |
| `leiden-tui` (binary) | Interactive partition viewer, iteration progress, log pane | `ratatui`, `crossterm`, `color-eyre` (main-only), `tracing-subscriber`, `thiserror` |

Lint profile from `rust-code-rigor.md` applied verbatim at the workspace root.
Crate-level overrides are not used; the strict profile is the floor for all three.

---

## 5. Open Clarifications Resolved

The spec already carries a `## Clarifications` section with five resolved
questions (input format, node-id representation, termination, weight type,
output format). No additional `NEEDS CLARIFICATION` markers remain.

The single new decision introduced by this plan — **use Ratatui for the TUI** —
is documented in §2 above with rationale and alternatives, satisfying the
"NEEDS CLARIFICATION" gate.