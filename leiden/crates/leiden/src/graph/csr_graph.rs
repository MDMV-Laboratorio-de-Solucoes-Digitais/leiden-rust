//! CSR graph (stub for Phase 2; real implementation in Phase 3).

use std::collections::HashMap;

use crate::graph::NodeId;

/// An undirected weighted graph in compressed sparse row form over dense `u32` indices.
///
/// Constructed only via `CsrGraph::from_edges` (Phase 3). This stub exists so
/// `lib.rs` re-exports compile during Phase 2 (`T020`).
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "Phase 2 stub — real CSR construction lands in Phase 3 (T037)"
)]
pub struct CsrGraph<Id: NodeId> {
    pub(crate) node_ids: Vec<Id>,
    pub(crate) index_of: HashMap<Id, u32>,
    pub(crate) offsets: Vec<u32>,
    pub(crate) adjacency: Vec<u32>,
    pub(crate) adjacency_weight: Vec<f64>,
    pub(crate) degrees: Vec<f64>,
    pub(crate) total_weight: f64,
}
