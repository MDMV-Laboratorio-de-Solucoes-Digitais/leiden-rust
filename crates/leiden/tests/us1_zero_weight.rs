//! US1: Zero-weight edges do not panic and compute finite modularity.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};

#[test]
fn zero_weight_edge_does_not_panic() {
    let edges = vec![
        Edge {
            source: "0".to_string(),
            target: "1".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "1".to_string(),
            target: "2".to_string(),
            weight: 1.0,
        },
        Edge {
            source: "0".to_string(),
            target: "2".to_string(),
            weight: 0.0, // Zero-weight edge
        },
    ];

    let graph = CsrGraph::from_edges(edges).expect("zero-weight edge is valid and non-negative");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("algorithm completes without panic on zero-weight edge");

    assert!(result.quality.is_finite(), "quality must be finite");
    assert!(!result.quality.is_nan(), "quality must not be NaN");
    assert_eq!(result.partition.len(), 3);
}
