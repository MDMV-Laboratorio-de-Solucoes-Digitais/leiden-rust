//! Property-based test: Returned communities are internally connected (T120).

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
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn proptest_communities_connected(
        num_nodes in 5..40usize,
        density in 0.15..0.7f64,
        gamma in 0.2..3.0f64,
        seed in any::<u64>(),
    ) {
        let mut edges = Vec::new();
        for u in 0..num_nodes {
            for v in (u + 1)..num_nodes {
                let p = ((u * 41 + v * 23 + 7) % 100) as f64 / 100.0;
                if p < density {
                    edges.push(Edge {
                        source: u as u32,
                        target: v as u32,
                        weight: 1.0 + (((u + v) % 7) as f64),
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
                seed: Some(seed),
                iteration_cap: 10,
            })
            .run(&graph);

        prop_assert!(result.is_ok());
        let res = result.expect("run succeeds");

        // Verify connected component invariant for each community
        let mut comm_members: HashMap<u32, Vec<u32>> = HashMap::new();
        for (node, comm) in &res.partition {
            comm_members.entry(*comm).or_default().push(*node);
        }

        for members in comm_members.values() {
            if members.len() <= 1 {
                continue;
            }
            let member_set: HashSet<u32> = members.iter().copied().collect();
            let start = members[0];
            let mut visited = HashSet::new();
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

            prop_assert_eq!(
                visited.len(),
                members.len(),
                "Community must be internally connected in induced subgraph"
            );
        }
    }
}
