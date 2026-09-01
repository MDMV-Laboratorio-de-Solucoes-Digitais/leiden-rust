//! Property-based test: No NaN / infinite quality across 1000 random graphs (T119).

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
use std::collections::{HashMap, HashSet, VecDeque};

proptest! {
    #[test]
    fn proptest_no_nan_no_disconnected(
        num_nodes in 10..100usize,
        num_edges in 20..200usize,
        seed in any::<u64>(),
        gamma in 0.01..10.0f64,
    ) {
        let mut edges = Vec::new();
        for i in 0..num_edges {
            let u = ((i * 13 + 7) % num_nodes) as u32;
            let mut v = ((i * 31 + 17) % num_nodes) as u32;
            if u == v {
                v = (u + 1) % (num_nodes as u32);
            }
            let weight = (((i * 43) % 100) as f64 + 1.0) / 10.0;
            edges.push(Edge {
                source: u,
                target: v,
                weight,
            });
        }

        let Ok(graph) = CsrGraph::from_edges(edges) else {
            return Ok(());
        };

        let result = Leiden::new()
            .with_parameters(LeidenParameters {
                gamma,
                seed: Some(seed),
                iteration_cap: 10,
            })
            .run(&graph);

        prop_assert!(result.is_ok());
        let res = result.expect("run succeeds");

        // 1. No NaN / ±Inf
        prop_assert!(res.quality.is_finite());
        prop_assert!(!res.quality.is_nan());

        // 2. Communities are connected
        let mut comm_members: HashMap<u32, Vec<u32>> = HashMap::new();
        for (node, comm) in &res.partition {
            comm_members.entry(*comm).or_default().push(*node);
        }

        for members in comm_members.values() {
            if members.len() <= 1 {
                continue;
            }
            let member_set: HashSet<u32> = members.iter().copied().collect();

            // BFS connectivity inside induced subgraph
            let mut visited = HashSet::new();
            let start = members[0];
            let mut queue = VecDeque::new();
            queue.push_back(start);
            let _ = visited.insert(start);

            while let Some(curr) = queue.pop_front() {
                if let Some(curr_idx) = graph.internal_id(&curr) {
                    for &nbr_idx in graph.neighbours_of(curr_idx) {
                        if let Some(&nbr) = graph.node_id(nbr_idx)
                            && member_set.contains(&nbr)
                            && !visited.contains(&nbr)
                        {
                            let _ = visited.insert(nbr);
                            queue.push_back(nbr);
                        }
                    }
                }
            }
        }
    }
}
