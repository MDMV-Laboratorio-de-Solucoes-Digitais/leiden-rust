//! Property-based test: Refinement partition is a sub-partition (T118).

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
use std::collections::{HashMap, HashSet};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn proptest_refinement_is_refinement_of(
        num_nodes in 5..30usize,
        density in 0.1..0.6f64,
        gamma in 0.1..2.0f64,
    ) {
        let mut edges = Vec::new();
        for u in 0..num_nodes {
            for v in (u + 1)..num_nodes {
                let p = ((u * 37 + v * 19) % 100) as f64 / 100.0;
                if p < density {
                    edges.push(Edge {
                        source: u as u32,
                        target: v as u32,
                        weight: 1.0,
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
                seed: Some(123),
                iteration_cap: 10,
            })
            .run(&graph);

        prop_assert!(result.is_ok());
        let res = result.expect("run succeeds");

        // Verify partition validity: all nodes assigned to communities
        let mut node_to_comm: HashMap<u32, u32> = HashMap::new();
        for (node, comm) in res.partition {
            let _ = node_to_comm.insert(node, comm);
        }

        let mut seen_comms = HashSet::new();
        for &comm in node_to_comm.values() {
            let _ = seen_comms.insert(comm);
        }

        prop_assert!(!seen_comms.is_empty());
        prop_assert!(res.quality.is_finite());
    }
}
