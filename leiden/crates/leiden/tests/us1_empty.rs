//! US1: Empty graph returns empty partition with `DegenerateInput`.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::doc_markdown,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters, TerminationReason};

#[test]
fn empty_graph_returns_empty_partition() {
    let edges: Vec<Edge<String>> = vec![];

    // LeidenError::EmptyGraph is returned on construct if empty
    let graph_err = CsrGraph::from_edges(edges);
    assert!(graph_err.is_err());

    // When a 0-node graph is run directly
    let empty_csr: CsrGraph<String> = CsrGraph::empty();
    let result = Leiden::new()
        .with_parameters(LeidenParameters::default())
        .run(&empty_csr)
        .expect("empty graph returns RunResult with DegenerateInput");

    assert_eq!(result.partition.len(), 0);
    assert_eq!(result.quality, 0.0);
    assert_eq!(result.iterations, 0);
    assert_eq!(
        result.termination_reason,
        TerminationReason::DegenerateInput
    );
}
