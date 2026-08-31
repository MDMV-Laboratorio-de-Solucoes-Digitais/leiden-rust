//! Weighted edge between two user-supplied node ids (stub for Phase 2).

use crate::graph::NodeId;

/// A weighted undirected edge between two user-supplied node ids.
///
/// Self-loops are accepted by the parser but rejected by `CsrGraph::from_edges`
/// (FR-008); the error references the offending line/field.
/// Multiple edges between the same unordered node pair are preserved verbatim
/// in the parser's output stream; summation into a single CSR entry is a
/// CSR-construction-time behavior of `CsrGraph::from_edges` (Phase 3).
#[derive(Debug, Clone)]
pub struct Edge<Id: NodeId> {
    /// Source node id.
    pub source: Id,
    /// Target node id.
    pub target: Id,
    /// Non-negative finite weight.
    pub weight: f64,
}
