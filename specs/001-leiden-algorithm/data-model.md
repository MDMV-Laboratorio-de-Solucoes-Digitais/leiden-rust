# Data Model: Leiden Algorithm in Rust

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30
**Output of**: `$speckit-plan` Phase 1 (Design & Contracts)

The data model covers the domain types in the `leiden` library crate plus the
two binary crates' I/O structs. All public items carry `///` documentation
and `#[derive(Debug)]` per Constitution Principle IV. Error variants are
defined in `error.rs` with `thiserror` per Principle III.

```rust
use std::num::NonZeroU32;
```

---

## 1. Library Domain Types

### 1.1 `NodeId` trait

```rust
/// A stable user-supplied identifier for a graph node.
///
/// The library does not assume any particular representation; callers may use
/// strings, integers, UUIDs, or any other `Hash + Eq` type. Internally, every
/// node is mapped to a dense `u32` index; that mapping is private and
/// preserved across all operations on a graph.
///
/// `Ord` is required so `RunResult::partition` can sort assignments by
/// user-supplied id (FR-001; `library-api.md §7`).
pub trait NodeId: Hash + Eq + Clone + Ord {}

impl<T> NodeId for T where T: Hash + Eq + Clone + Ord {}
```

### 1.2 `Edge<Id>`

```rust
/// A weighted undirected edge between two user-supplied node ids.
///
/// Self-loops are accepted by the parser but rejected by `CsrGraph::from_edges`
/// (FR-008); the error references the offending line/field.
/// Multiple edges between the same unordered node pair are **preserved verbatim**
/// in the parser's output stream (both CLI and library); they are **not**
/// deduplicated, validated, or rejected by the parser. Summation into a single
/// CSR entry whose weight is the sum of the parallel weights, in first-seen
/// order, is a CSR-construction-time behavior of `CsrGraph::from_edges`
/// (per `spec.md FR-001` and `spec.md §Edge Cases` "Self-loops and parallel
/// edges", and the canonical-neighbour-id ordering rule added in the 2026-08-31
/// `$speckit-analyze` finding A4 remediation).
pub struct Edge<Id: NodeId> {
    pub source: Id,
    pub target: Id,
    pub weight: f64,
}
```

### 1.3 `CsrGraph<Id>`

```rust
/// An undirected weighted graph in compressed sparse row form over dense u32
/// indices.
///
/// `node_ids` is the public id ↔ internal index mapping. `adjacency` and
/// `adjacency_weight` form the CSR structure: for internal index `i`,
/// `adjacency[offsets[i]..offsets[i+1]]` are the indices of its neighbours
/// (each appearing once, regardless of edge direction), and
/// `adjacency_weight` carries the parallel weights.
///
/// `total_weight` is the sum of all edge weights (i.e. `m`, the modularity
/// denominator). For empty graphs it is `0.0`.
///
/// Constructed only via `CsrGraph::from_edges`, which validates input and
/// returns a typed `LeidenError` on malformed input (FR-008).
pub struct CsrGraph<Id: NodeId> {
    pub(crate) node_ids: Vec<Id>,
    pub(crate) index_of: HashMap<Id, u32>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) adjacency: Vec<u32>,
    pub(crate) adjacency_weight: Vec<f64>,
    pub(crate) degrees: Vec<f64>,
    pub(crate) total_weight: f64,
}

impl<Id: NodeId> CsrGraph<Id> {
    /// Build a CSR graph from an iterator of edges.
    ///
    /// Rejects negative weights, self-loops, NaN/±∞ weights, and dangling
    /// references. Errors carry the offending line/field index from the
    /// caller's source. Deterministic: nodes are numbered in first-seen order.
    pub fn from_edges<I>(edges: I) -> Result<Self, LeidenError>
    where I: IntoIterator<Item = Edge<Id>>;

    pub fn node_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;
    pub fn total_weight(&self) -> f64;
    pub fn degree_of(&self, internal: u32) -> f64;
    pub fn neighbours_of(&self, internal: u32) -> &[u32];
    pub fn weights_of(&self, internal: u32) -> &[f64];
}
```

### 1.4 `Partition`

```rust
/// An assignment of every internal node index to a community id, plus the
/// derived per-community statistics used by every phase.
///
/// `assignment` is dense: `assignment[i] = community of node i`. Community
/// ids are dense `u32` starting at `0` and renumbered after each phase.
///
/// `sigma_tot[c]` is the sum of degrees of nodes in community `c`. Used by
/// the modularity ΔQ formula in O(1) per move.
///
/// `internal_to_community` is the inverse map (community → node set).
pub struct Partition {
    pub(crate) assignment: Vec<u32>,
    pub(crate) sigma_in: Vec<f64>,
    pub(crate) sigma_tot: Vec<f64>,
    pub(crate) internal_to_community: Vec<Vec<u32>>,
    pub(crate) community_count: u32,
}

impl Partition {
    /// Build the singleton partition (every node in its own community).
    pub fn singletons(node_count: usize) -> Self;

    /// Number of distinct communities.
    pub fn community_count(&self) -> u32;

    /// Look up the community of an internal node.
    pub fn community_of(&self, node: u32) -> u32;

    /// Move a node to a (possibly empty) target community, updating
    /// `sigma_in` / `sigma_tot` incrementally. If `to >= self.community_count`,
    /// the target community is auto-created and `community_count` is incremented
    /// (i.e., the partition is extended lazily). The caller is responsible for
    /// calling `renumber()` after a phase to re-establish dense `0..k` ids.
    ///
    /// **Internally-connected invariant (definitional authority: FR-002)**:
    /// `move_node` does NOT enforce the FR-002 "every community's induced
    /// subgraph is connected" invariant on its own. A naive move may produce
    /// communities whose induced subgraphs are disconnected; the invariant is
    /// re-established by the **refinement phase** (`refinement/mod.rs` per
    /// `spec.md FR-002` and `tasks.md` T032a / T044). Callers that need a
    /// FR-002-compliant partition MUST run `refinement` after a sequence of
    /// `move_node` calls; `move_node` alone is suitable only for internal use
    /// inside the local-moving phase, where the subsequent refinement restores
    /// the invariant before `Leiden::run` returns (remediation for
    /// `$speckit-analyze` finding A5, 2026-08-31).
    pub fn move_node(&mut self, node: u32, to: u32);

    /// Renumber communities to a dense `0..k` range. Called after every
    /// phase that may have created empty communities.
    pub fn renumber(&mut self);

    /// True iff this partition is a refinement of `other` (every community
    /// of `self` is a subset of some community of `other`). Used by the
    /// orchestrator to detect stable iterations.
    pub fn is_refinement_of(&self, other: &Partition) -> bool;
}
```

### 1.5 `LeidenParameters`

```rust
/// User-supplied parameters for one algorithm run.
///
/// `gamma` defaults to `1.0`; `gamma <= 0.0` is rejected by `validate`
/// (FR-003). `iteration_cap` defaults to `10` (FR-003a, matching Traag et al.
/// 2019). `seed` is `None` for the v1 deterministic variant (FR-004): v1
/// does not consume `seed` for any algorithm decision; tie-breaks use the
/// lowest internal node id. `seed` is carried through `RunResult.seed`
/// verbatim for forward compatibility only (see `spec.md` Assumptions
/// "Algorithm variant" and Clarifications 2026-08-30 (seed field)). Stochastic
/// variants, if added later, MUST be gated behind a Cargo feature flag AND
/// require a Constitution amendment.
pub struct LeidenParameters {
    pub gamma: f64,
    pub seed: Option<u64>,
    pub iteration_cap: u32,
}

impl LeidenParameters {
    pub fn default_gamma() -> f64 { 1.0 }
    pub fn default_iteration_cap() -> u32 { 10 }
}
```

### 1.6 `QualityFunction` trait

```rust
/// A quality function over partitions of a graph.
///
/// Modularity is the only v1 implementation. CPM and Reichardt–Bornholdt
/// are explicit out-of-scope per the spec's Assumptions; the trait exists
/// so a future feature can add them without breaking the public API.
pub trait QualityFunction {
    /// Total quality of a partition.
    fn total_quality(
        &self,
        graph: &CsrGraph<impl NodeId>,
        partition: &Partition,
    ) -> f64;

    /// Modular-style ΔQ for moving `node` from its current community to
    /// `target_community`. `MoveComponents` pre-computes the per-phase
    /// quantities needed by the incremental formula.
    fn delta_move(
        &self,
        graph: &CsrGraph<impl NodeId>,
        partition: &Partition,
        node: u32,
        target_community: u32,
        components: &MoveComponents,
    ) -> f64;
}

/// Modularity quality function (Traag 2019, Eq. 1).
pub struct Modularity {
    pub(crate) gamma: f64,
}
```

### 1.7 `MoveComponents`

```rust
/// Cached per-node quantities used by `QualityFunction::delta_move` to
/// avoid recomputation across phases.
///
/// `k_i` is the weighted degree of the node (self-loops counted once).
/// `sigma_in_to_target` / `sigma_tot_target` are computed once per candidate
/// community in `local_moving`.
pub struct MoveComponents {
    pub(crate) k_i: f64,
    pub(crate) sigma_in_to_target: f64,
    pub(crate) sigma_tot_target: f64,
    pub(crate) sigma_in_from_current: f64,
    pub(crate) sigma_tot_current: f64,
}
```

### 1.8 `LeidenEvent`

```rust
/// A structured observability event emitted by every phase.
///
/// Emitted via `tracing` (library) and also sent over an `mpsc::Sender` to
/// the TUI's render loop (binary). `tracing` events are the source of truth;
/// the TUI consumes the same payload via a custom subscriber layer.
pub enum LeidenEvent {
    GraphLoaded { nodes: usize, edges: usize, total_weight: f64 },
    IterationStarted { index: u32, phase: Phase },
    LocalMovingProgress { iteration: u32, moved_nodes: u32 },
    LocalMovingDelta { iteration: u32, delta_q: f64 },
    RefinementMerged { iteration: u32, from: u32, to: u32 },
    Aggregation { iteration: u32, aggregate_nodes: usize },
    QualityComputed { iteration: u32, quality: f64 },
    IterationFinished { index: u32, quality: f64 },
    Terminated { iterations: u32, reason: TerminationReason, quality: f64 },
}

pub enum Phase { LocalMoving, Refinement, Aggregation }
```

### 1.9 `TerminationReason`

```rust
/// Why the orchestrator stopped.
///
/// `Converged` is the paper's "stable iteration": no node moved in local
/// moving AND refinement didn't split anything. `IterationCap` is the
/// user-set ceiling. `DegenerateInput` is reserved for runs on a graph
/// with zero edges or zero nodes where the algorithm short-circuits.
pub enum TerminationReason { Converged, IterationCap, DegenerateInput }
```

### 1.10 `RunResult<Id>`

```rust
/// The bundle returned from a single `Leiden::run` invocation.
///
/// `partition` maps user-supplied node ids to community ids. `quality` is
/// the final modularity value. `termination_reason` and `iterations` are
/// the orchestrator's verdict.
pub struct RunResult<Id: NodeId> {
    pub partition: Vec<(Id, u32)>,
    pub quality: f64,
    pub iterations: u32,
    pub termination_reason: TerminationReason,
    /// The seed supplied to `Leiden::run` via `LeidenParameters::seed`,
    /// round-tripped verbatim. `None` in / `None` out; `Some(s)` in / `Some(s)` out.
    /// v1 does not consume this field; reserved for a future stochastic variant
    /// (see `spec.md` Clarifications 2026-08-30 (seed field)).
    pub seed: Option<u64>,
    /// The threading policy supplied to `Leiden::with_threads`, round-tripped
    /// verbatim. `SingleThreaded` is produced when the builder is not called.
    /// In v1 only `SingleThreaded` is ever produced; `ThreadPoolSize` is
    /// reserved for a future multi-threaded variant that requires a
    /// Constitution amendment (see `spec.md` FR-012; `tasks.md` T129).
    pub threading: ThreadingPolicy,
}
```

**Builder parameter vs result-field distinction**: the `Leiden::with_threads` builder takes a `NonZeroU32` parameter (per `spec.md` FR-012 and `tasks.md` T129); the `RunResult.threading` field reports `ThreadingPolicy` (an enum value, currently always `SingleThreaded` in v1).

```rust
/// Threading policy applied to a `Leiden::run` invocation.
///
/// In v1 only `SingleThreaded` is ever produced; the run is strictly
/// sequential. `ThreadPoolSize` is reserved for a future multi-threaded
/// variant and is gated behind a Constitution amendment (see `spec.md`
/// FR-012; `tasks.md` T129). Callers MUST treat `ThreadPoolSize` as an
/// unreachable variant in v1.
pub enum ThreadingPolicy {
    SingleThreaded,
    ThreadPoolSize(NonZeroU32),
}
```

### 1.11 `LeidenError` (thiserror enum)

```rust
/// All fallible operations in the library return this error type.
///
/// Variants carry enough context (line/field, offending value) to satisfy
/// FR-008. No `From<io::Error>` blanket impl: fallible conversions use
/// `TryFrom` (Principle III, `fallible_impl_from = deny`).
#[derive(Debug, thiserror::Error)]
pub enum LeidenError {
    /// Wraps a general graph-input-shape failure that does not fit any other
    /// variant. Used for input-shape errors that lack a specific field or value
    /// to highlight (e.g. malformed edge-list header `# nodes=N` with a count
    /// that disagrees with the actual unique nodes, per `cli-schema.md §1.3.1`).
    /// For specific per-field errors, prefer `InvalidWeight`, `SelfLoop`,
    /// `DanglingNode`, or `EmptyGraph` instead of this catch-all. The `line`
    /// field is `Some(N)` when the CLI parser emits this error (source line
    /// known) and `None` when the library emits it directly (no source-line
    /// context in the `IntoIterator<Item = Edge<Id>>` API).
    #[error("graph input: {message}")]
    Graph { message: String, line: Option<usize> },

    #[error("invalid weight `{value}` at line {line}: must be finite and ≥ 0")]
    InvalidWeight { line: usize, value: f64 },

    /// Self-loop rejected at the input boundary.
    ///
    /// `line` is `Some(N)` when emitted by the CLI parser (where the source line
    /// number is known) and `None` when emitted by `CsrGraph::from_edges` (whose
    /// `IntoIterator<Item = Edge<Id>>` API carries no line context). The `node`
    /// field is the offending user-supplied node id rendered as a `String` at
    /// both boundaries. This shape is locked by `spec.md` FR-008 and `tasks.md`
    /// T024a (library, `line == None`) + T081 (CLI, `line == Some(N)`).
    #[error("self-loop at line {line:?} on node `{node}`: not permitted")]
    SelfLoop { line: Option<usize>, node: String },

    #[error("node id `{0}` appears in edges but not in any declared node set")]
    DanglingNode(String),

    #[error("resolution γ must be > 0; got {0}")]
    InvalidGamma(f64),

    #[error("iteration cap must be ≥ 1; got {0}")]
    InvalidIterationCap(u32),

    #[error("graph is empty: no nodes")]
    EmptyGraph,
}
```

---

## 2. State Transitions

The orchestrator advances through these states:

```text
INIT
  │
  ▼
GRAPH_LOAD ── (validate CsrGraph) ──► GRAPH_LOADED
  │
  ▼
SINGLETON_PARTITION
  │
  ▼
  ┌─────────────── LOOP START ───────────────┐
  │                                           │
  ▼                                           │
LOCAL_MOVING ─── ΔQ for each node ───► LOCAL_MOVED   ◄─────────┐
  │                                                            │
  ▼                                                            │
REFINEMENT ─── deterministic merge ───► REFINED                 │
  │                                                            │
  ▼                                                            │
AGGREGATION ─── build aggregate ───► AGGREGATED                 │
  │                                                            │
  ▼                                                            │
TERMINATION_CHECK                                               │
  │  ┌── converged ──► DONE (Converged)                         │
  │  ├── cap reached ──► DONE (IterationCap)                    │
  │  └── otherwise ──► LOOP START ──────────────────────────────┘
```

`DONE` always returns a `RunResult`. `DegenerateInput` is a one-shot early exit
from `GRAPH_LOADED` when the graph has zero edges (empty-graph case from
US-1 acceptance scenario 3); the partition is the singleton partition over
zero nodes.

---

## 3. Binary-Crate I/O Structs

### 3.1 `leiden-cli` Parser Structs

```rust
/// JSON adjacency document (FR-007a).
#[derive(Debug, serde::Deserialize)]
struct AdjacencyDoc {
    nodes: Vec<serde_json::Value>,
    edges: Vec<[serde_json::Value; 2]>,
    #[serde(default)]
    weights: Option<Vec<f64>>,
}
```

### 3.2 `leiden-cli` Output Structs

```rust
/// JSON partition output (FR-007b).
///
/// All field names are explicitly annotated to lock the JSON shape: `gamma`,
/// `seed`, `iterations`, `termination_reason` (snake_case, NOT the PascalCase
/// Rust variant name), `quality`, `threading` (added per FR-007b post-A18
/// reconciliation; always `"SingleThreaded"` in v1), and `assignments`. The
/// `termination_reason` value is the snake_case string
/// (`"converged" | "iteration_cap" | "degenerate_input"`), NOT the PascalCase
/// Rust variant. The `threading` value is the string `"SingleThreaded"` in v1
/// (per FR-012).
#[derive(Debug, serde::Serialize)]
struct PartitionOutput<'a> {
    gamma: f64,
    seed: Option<u64>,
    iterations: u32,
    #[serde(rename = "termination_reason")]
    termination_reason: &'a str,
    quality: f64,
    #[serde(rename = "threading")]
    threading: &'a str,
    #[serde(rename = "assignments")]
    assignments: Vec<Assignment<'a>>,
}

#[derive(Debug, serde::Serialize)]
struct Assignment<'a> {
    #[serde(rename = "node")]
    node: &'a str,
    #[serde(rename = "community")]
    community: u32,
}
```

### 3.3 `leiden-cli` Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{path}:{line}: expected 2 or 3 fields, got {got}")]
    ParseFieldCount { path: String, line: usize, got: usize },

    #[error("{path}:{line}: invalid weight `{value}`: {source}")]
    ParseWeight {
        path: String, line: usize, value: String,
        #[source] source: std::num::ParseFloatError,
    },

    #[error("unsupported output format `{0}`; expected `json` or `text`")]
    UnsupportedFormat(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("leiden: {0}")]
    Leiden(#[from] LeidenError),
}
```

### 3.4 `leiden-tui` App State

```rust
/// TUI state machine.
#[derive(Debug)]
pub enum AppState {
    Idle,
    Running { iteration: u32 },
    Done { iterations: u32, quality: f64 },
    Error(String),
}

#[derive(Debug)]
pub struct App {
    pub state: AppState,
    pub events: Vec<LeidenEvent>,
    pub log: LogRing,
    pub show_log: bool,
    pub show_graph: bool,
    pub selected_community: Option<u32>,
}
```

Note: an `Error → Idle` recovery transition test (T101a) is required to satisfy spec.md FR-008 graceful-recovery expectations.

---

### 3.5 CLI Exit-Code Mapping (authoritative; matches `contracts/cli-schema.md §1.6`)

| `CliError` variant | Exit code | Source |
|---|---|---|
| (success path — no error) | `0` | T091, T091a |
| `UnsupportedFormat` | `2` | T077, T091a |
| `Leiden(InvalidGamma)` / `Leiden(InvalidIterationCap)` | `3` | T083, T091a |
| `ParseFieldCount` / `ParseWeight` / `Leiden(InvalidWeight)` / `Leiden(SelfLoop)` / `Leiden(DanglingNode)` / `Leiden(Graph)` | `4` | T080, T081, T082, T091a |
| `Io` | `5` | T083a, T091a |
| (unreachable: unexpected internal error) | `1` | defensive fallback only |

The mapping is exercised end-to-end by `tasks.md` T091a (`all_exit_codes_exercised`).

---

## 4. Validation Rules (consolidated)

| Constraint | Enforced at | Source |
|---|---|---|
| Weight ≥ 0, finite | `CsrGraph::from_edges` | FR-001, FR-008 |
| No self-loops | `CsrGraph::from_edges` | FR-008 |
| No dangling node ids | `CsrGraph::from_edges` (JSON path) | FR-008 |
| γ > 0 | `LeidenParameters::validate` | FR-003 |
| iteration_cap ≥ 1 | `LeidenParameters::validate` | FR-003a |
| Partition: every node in exactly one community | invariant of `Partition` | FR-001 |
| Community: internally connected | post-condition of refinement | FR-002 |
| Determinism under fixed seed/γ | determinism property test | FR-004 |
| Quality finite, no NaN | property test asserts `!q.is_nan()` | SC-003 |