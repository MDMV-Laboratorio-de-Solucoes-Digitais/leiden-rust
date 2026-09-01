# Library API Contract: `leiden`

**Branch**: `[001-leiden-algorithm]` | **Date**: 2026-08-30
**Stability**: this contract is the v1 public surface; breaking changes require
a MAJOR version bump.

This document defines the **public library API** exposed by the `leiden`
crate (in `crates/leiden/src/lib.rs`). All items are `pub`; all internal
modules use `pub(crate)` / `pub(super)` to keep `unreachable_pub = deny` clean.

---

## 1. Re-exports

```rust
//! `leiden` — deterministic Leiden community detection.
//!
//! Faithful implementation of Traag, Waltman, van Eck (2019), Sci. Rep.
//! 9:5233. Deterministic, panic-free, fully documented.
//!
//! # Quick start
//!
//! ```
//! use leiden::{CsrGraph, Edge, Leiden, LeidenParameters, NodeId};
//!
//! fn smoke() -> Result<(), leiden::LeidenError> {
//!     let edges = vec![
//!         Edge { source: "a".to_string(), target: "b".to_string(), weight: 1.0 },
//!         Edge { source: "c".to_string(), target: "d".to_string(), weight: 1.0 },
//!     ];
//!     let graph = CsrGraph::from_edges(edges)?;
//! let result = Leiden::new()
//!     .with_parameters(LeidenParameters { gamma: 1.0, seed: None, iteration_cap: 10 })
//!     .run(&graph)?;
//!     debug_assert!(result.quality.is_finite());
//!     Ok(())
//! }
//! ```

pub use crate::error::LeidenError;
pub use crate::graph::{CsrGraph, Edge, NodeId};
pub use crate::orchestrator::{Leiden, RunResult, TerminationReason, ThreadingPolicy};
pub use crate::partition::Partition;
pub use crate::quality::{Modularity, QualityFunction, MoveComponents};
pub use crate::params::LeidenParameters;
pub use crate::events::{LeidenEvent, Phase};
```

---

## 2. Builder: `Leiden`

```rust
/// Orchestrator entry point. Construct with `Leiden::new()`, chain
/// configuration with the `with_*` methods, then call `run` with a graph.
pub struct Leiden { /* private */ }

impl Leiden {
    /// Construct a Leiden orchestrator with default configuration.
    pub fn new() -> Self;

    /// Override the resolution parameter γ (default `1.0`). Panics-free;
    /// validation occurs in `run`.
    pub fn with_parameters(self, params: LeidenParameters) -> Self;

    /// Run the algorithm on `graph` and return the partition plus quality.
    ///
    /// # Errors
    ///
    /// Returns `LeidenError::InvalidGamma` if `params.gamma <= 0`,
    /// `LeidenError::InvalidIterationCap` if `params.iteration_cap < 1`,
    /// `LeidenError::Graph` on malformed input (delegated from `CsrGraph`).
    pub fn run<Id: NodeId>(
        self,
        graph: &CsrGraph<Id>,
    ) -> Result<RunResult<Id>, LeidenError>;
}

/// **(Formal enforcement of FR-006 — no `&mut self` on `Leiden`)** The
/// `Leiden` orchestrator exposes **only** builder methods that take `self` and
/// return `Self` (`with_parameters`, `with_event_sink`, `with_threads`), plus
/// the consuming `run` method. There is no `&mut self` or `&mut self` returning
/// non-`Self` method on the public `Leiden` surface. This is enforced at
/// compile time by a sealed marker trait in `tasks.md` T066a
/// (`leiden_public_api_has_no_mut_methods`): a `const _: () = ...` block
/// instantiates a private marker that requires the trait shape, so any future
/// PR that adds an `&mut self` method fails the build.
```

---

## 3. Graph Construction: `CsrGraph`

See `data-model.md §1.3` for the full struct definition. The contract surface
is:

```rust
impl<Id: NodeId> CsrGraph<Id> {
    pub fn from_edges<I>(edges: I) -> Result<Self, LeidenError>
    where I: IntoIterator<Item = Edge<Id>>;

    pub fn node_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;
    pub fn total_weight(&self) -> f64;
    pub fn degree_of(&self, internal: u32) -> f64;
    pub fn neighbours_of(&self, internal: u32) -> &[u32];
    pub fn weights_of(&self, internal: u32) -> &[f64];
}

impl<Id: NodeId> Debug for CsrGraph<Id>;
```

**Determinism contract**: `from_edges` numbers nodes in **first-seen order**.
Two calls with identical input yield identical internal indices.

**Validation contract**: `from_edges` returns a `LeidenError` on:
- negative or non-finite weight → `LeidenError::InvalidWeight { line, value }`
- self-loop → `LeidenError::SelfLoop { line: None, node }` (the library emits `line: None` because the `IntoIterator<Item = Edge<Id>>` API has no line context; the CLI emits `line: Some(N)` for the source line)
- empty input (no edges, no nodes) → `LeidenError::EmptyGraph`
- (parser-only, not here) dangling node id → `LeidenError::DanglingNode` (CLI JSON adjacency parser)

---

---

## 4. Partition: `Partition`

See `data-model.md §1.4` for the full struct definition and invariants.

```rust
impl Partition {
    /// Build the singleton partition (every node in its own community).
    pub fn singletons(node_count: usize) -> Self;

    /// Number of distinct communities.
    pub fn community_count(&self) -> u32;

    /// Look up the community of an internal node.
    pub fn community_of(&self, node: u32) -> u32;

    /// Move a node to a (possibly empty) target community, updating
    /// `sigma_in` / `sigma_tot` incrementally.
    pub fn move_node(&mut self, node: u32, to: u32);

    /// Renumber communities to a dense `0..k` range.
    pub fn renumber(&mut self);

    /// True iff this partition is a refinement of `other` (every community
    /// of `self` is a subset of some community of `other`).
    pub fn is_refinement_of(&self, other: &Partition) -> bool;
}

impl Debug for Partition;
```

---

## 5. Algorithm Parameters: `LeidenParameters`

```rust
/// `seed` is metadata only in v1 (the deterministic algorithm ignores it);
/// see `data-model.md §1.5` and `spec.md §Clarifications 2026-08-30 (seed field)`.
#[derive(Debug, Clone)]
pub struct LeidenParameters {
    pub gamma: f64,
    pub seed: Option<u64>,
    pub iteration_cap: u32,
}

impl Default for LeidenParameters {
    fn default() -> Self {
        Self { gamma: 1.0, seed: None, iteration_cap: 10 }
    }
}

impl LeidenParameters {
    /// Validates parameters. Fails fast in declaration order: checks `gamma` first
    /// (returning `LeidenError::InvalidGamma` if `gamma <= 0.0` or non-finite),
    /// then `iteration_cap` (returning `LeidenError::InvalidIterationCap` if `iteration_cap < 1`).
    pub fn validate(&self) -> Result<(), LeidenError>;
}
```

---

## 6. Quality Function Trait & Move Components

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MoveComponents {
    pub k_i: f64,
    pub sigma_in_to_target: f64,
    pub sigma_tot_target: f64,
    pub sigma_in_from_current: f64,
    pub sigma_tot_current: f64,
}

impl MoveComponents {
    pub fn new(
        k_i: f64,
        sigma_in_to_target: f64,
        sigma_tot_target: f64,
        sigma_in_from_current: f64,
        sigma_tot_current: f64,
    ) -> Self;
}

pub trait QualityFunction {
    fn total_quality<Id: NodeId>(
        &self,
        graph: &CsrGraph<Id>,
        partition: &Partition,
    ) -> f64;

    /// Modular-style ΔQ for moving `node` from its current community to
    /// `target_community`. When `target_community == current_community`,
    /// this unconditionally returns `0.0`.
    fn delta_move<Id: NodeId>(
        &self,
        graph: &CsrGraph<Id>,
        partition: &Partition,
        node: u32,
        target_community: u32,
        components: &MoveComponents,
    ) -> f64;
}

pub struct Modularity { pub gamma: f64 }
impl QualityFunction for Modularity { /* … */ }
```

---

## 7. Observability Events: `LeidenEvent`

See `data-model.md §1.8` for the full enum. The contract is:

- Library emits `LeidenEvent` variants via `tracing::info!` (with structured
  fields) AND pushes the same payload to any `mpsc::Sender<LeidenEvent>`
  registered via `Leiden::with_event_sink`.
- Consumers MUST treat `LeidenEvent` as a value type; the library never panics
  on a closed or full channel.
- The TUI registers itself as a sink; the CLI does not (CLI observes events
  via the global `tracing-subscriber` writing to stderr).

```rust
impl Leiden {
    pub fn with_event_sink(self, tx: std::sync::mpsc::Sender<LeidenEvent>) -> Self;
}
```

---

## 8. Run Result: `RunResult<Id>`

```rust
#[derive(Debug, Clone)]
pub struct RunResult<Id: NodeId> { // `Ord` is part of the NodeId supertraits per data-model.md §1.1
    pub partition: Vec<(Id, u32)>,
    pub quality: f64,
    pub iterations: u32,
    pub termination_reason: TerminationReason,
    /// The seed supplied via `LeidenParameters::seed`, round-tripped verbatim.
    /// `None` in → `None` out; `Some(s)` in → `Some(s)` out. v1 does not consume
    /// this field; reserved for a future stochastic variant. See
    /// `spec.md` Clarifications 2026-08-30 (seed field) and `data-model.md §1.10`.
    pub seed: Option<u64>,
}

pub enum TerminationReason { Converged, IterationCap, DegenerateInput }
```

The `partition` vector is sorted by `Id`'s natural `Ord` ordering before
return. All node-id types implement `Ord`, which the `NodeId` trait
requires (see §3).

---

## 9. Error Type: `LeidenError`

See `data-model.md §1.11`. The contract is:

- Every variant carries enough context (line, field, value) for the caller to
  format a useful message.
- `thiserror` derives `Display` + `Error`; `From<std::io::Error>` is **not**
  implemented at the library level (Principle III — fallible conversions use
  `TryFrom`). The CLI's `CliError` provides the bridge.

---

## 10. Versioning

This contract is the v1 surface. Any breaking change requires:
1. A MAJOR version bump in `Cargo.toml`.
2. A `docs:` commit documenting the migration path.
3. A constitution amendment if the change affects a principle (e.g. lifting
   `missing_docs = deny` would be a MAJOR governance change).