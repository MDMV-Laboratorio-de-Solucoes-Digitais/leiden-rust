//! Orchestrator types (stub for Phase 2; real implementation in Phase 3).

// ref: Traag 2019 §3 — orchestrator/mod.rs stub; real outer loop (Algorithm A.2) lands in Phase 3.

/// Placeholder — real `Leiden` orchestrator lands in Phase 3 (US1).
#[derive(Debug)]
pub struct Leiden;

/// Placeholder — real `RunResult` lands in Phase 3 (US1).
#[derive(Debug)]
pub struct RunResult<Id> {
    _marker: std::marker::PhantomData<Id>,
}

/// Placeholder — real `TerminationReason` lives in `crate::events`.
/// This re-export keeps `library-api.md §1` surface intact.
pub use crate::events::TerminationReason;

/// Placeholder threading policy — real type lives in `crate::events`.
pub use crate::events::ThreadingPolicy;
