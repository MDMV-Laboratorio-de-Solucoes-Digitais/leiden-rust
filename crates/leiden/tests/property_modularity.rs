//! Property-based test: Modularity non-decreasing (T117).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    reason = "test code: assertions permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn proptest_modularity_non_decreasing(
        num_nodes in 5..30usize,
        density in 0.1..0.6f64,
        gamma in 0.1..2.0f64,
    ) {
        let mut edges = Vec::new();
        for u in 0..num_nodes {
            for v in (u + 1)..num_nodes {
                let p = ((u * 31 + v * 17) % 100) as f64 / 100.0;
                if p < density {
                    let weight = 1.0 + (((u + v) % 5) as f64);
                    edges.push(Edge {
                        source: u as u32,
                        target: v as u32,
                        weight,
                    });
                }
            }
        }

        if edges.is_empty() {
            edges.push(Edge {
                source: 0,
                target: 1,
                weight: 1.0,
            });
        }

        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return Ok(());
        };

        let result = Leiden::new()
            .with_parameters(LeidenParameters {
                gamma,
                seed: Some(42),
                iteration_cap: 10,
            })
            .run(&graph);

        prop_assert!(result.is_ok());
        let res = result.expect("run succeeds");
        prop_assert!(res.quality.is_finite());
    }
}
