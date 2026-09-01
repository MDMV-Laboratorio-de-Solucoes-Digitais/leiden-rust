//! Pathological weight-ratio stress tests (T119a).

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    reason = "test code: assertions permitted per Constitution §III"
)]

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};
use std::collections::{HashMap, HashSet, VecDeque};

#[test]
fn pathological_weight_ratios_no_nan() {
    // Two K3 cliques: {1, 2, 3} and {4, 5, 6}, joined by bridge (3, 4)
    // Edge (1, 2) has weight f64::MIN_POSITIVE
    // Edge (5, 6) has weight 1e300 (or f64::MAX)
    let edges = vec![
        // Clique 1
        Edge {
            source: 1,
            target: 2,
            weight: f64::MIN_POSITIVE,
        },
        Edge {
            source: 2,
            target: 3,
            weight: 1.0,
        },
        Edge {
            source: 1,
            target: 3,
            weight: 1.0,
        },
        // Bridge
        Edge {
            source: 3,
            target: 4,
            weight: 0.1,
        },
        // Clique 2
        Edge {
            source: 4,
            target: 5,
            weight: 1.0,
        },
        Edge {
            source: 5,
            target: 6,
            weight: 1e300,
        },
        Edge {
            source: 4,
            target: 6,
            weight: 1.0,
        },
    ];

    let graph = CsrGraph::from_edges(edges).expect("valid graph with extreme weights");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph);

    assert!(
        result.is_ok(),
        "Leiden::run must complete on extreme weights"
    );
    let res = result.expect("run succeeds");

    // Quality must be finite and not NaN
    assert!(
        res.quality.is_finite(),
        "quality must be finite, got {}",
        res.quality
    );
    assert!(!res.quality.is_nan(), "quality must not be NaN");

    // Every community must be internally connected
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

        assert_eq!(
            visited.len(),
            members.len(),
            "community members must be internally connected"
        );
    }
}
