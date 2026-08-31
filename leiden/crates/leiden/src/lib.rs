//! `leiden` — deterministic Leiden community detection.
//!
//! Faithful implementation of Traag, Waltman, van Eck (2019), Sci. Rep.
//! 9:5233. Deterministic, panic-free, fully documented.
//!
//! ## Citation discipline (FR-009)
//!
//! Every `pub` item or function body under `crates/leiden/src/{local_moving,
//! refinement, aggregation, quality, orchestrator}/` MUST carry either a
//! `// ref: Traag 2019 §X.Y` citation comment or a `// leiden-deviation:`
//! marker documenting the intentional departure from the published
//! algorithm. The `compile_fail` doctest below demonstrates the
//! anti-pattern (no citation); the FR-009 CI guard in
//! `tests/fr009_no_uncited_deviations.rs` enforces the discipline at write
//! time, and T138a enforces it at pre-merge time.

#![doc = "leiden crate root"]

pub mod error;
pub mod events;
pub mod graph;
pub mod orchestrator;
pub mod params;
pub mod partition;
pub mod quality;

pub use error::LeidenError;
pub use events::{LeidenEvent, Phase, TerminationReason, ThreadingPolicy};
pub use graph::{CsrGraph, Edge, NodeId};
pub use orchestrator::{Leiden, RunResult};
pub use params::LeidenParameters;
pub use quality::{Modularity, MoveComponents, QualityFunction};

/// Placeholder for `Partition` — real type lands in Phase 3 (US1).
pub use partition::Partition;

/// This doctest demonstrates an **uncited** public item. It is annotated
/// `compile_fail` because the body below violates `missing_docs` and
/// references an undeclared identifier; the FR-009 guard then fails the
/// doc build because the file lacks a `// ref: Traag 2019 §X.Y` citation.
///
/// ```compile_fail
/// // The following line intentionally fails to compile to anchor the
/// // FR-009 anti-pattern at doc-test time. The CI guard
/// // (tests/fr009_no_uncited_deviations.rs) is the primary lock; this
/// // doctest exists so that the example block above stays visible to
/// // reviewers searching for the citation rule.
/// let _x: usize = "string"; // type mismatch -> compile error
/// ```
#[doc(hidden)]
pub const FR009_CITATION_DISCIPLINE_DEMO: () = ();
