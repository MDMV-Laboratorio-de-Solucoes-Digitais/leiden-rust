//! Quality function trait and implementations.

// ref: Traag 2019 §3 — Quality functions for Leiden partition scoring.

pub mod modularity;

use crate::graph::{CsrGraph, NodeId};
use crate::partition::Partition;

pub use modularity::{Modularity, MoveComponents};

/// A quality function over partitions of a graph.
pub trait QualityFunction {
    /// Total quality of a partition.
    fn total_quality<Id: NodeId>(&self, graph: &CsrGraph<Id>, partition: &Partition) -> f64;

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
