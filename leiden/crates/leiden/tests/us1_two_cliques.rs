//! US1: Detect communities on two cliques connected by a bridge.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters, NodeId};

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    PathBuf::from(manifest_dir)
        .join("../../fixtures")
        .join(name)
}

fn load_edg(name: &str) -> CsrGraph<String> {
    let path = fixture_path(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"));

    let mut edges = Vec::new();
    let mut nodes = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 1 {
            nodes.push(parts[0].to_string());
        } else if parts.len() >= 2 {
            let src = parts[0].to_string();
            let dst = parts[1].to_string();
            let weight = if parts.len() >= 3 {
                parts[2]
                    .parse::<f64>()
                    .expect("valid float weight in fixture")
            } else {
                1.0
            };
            edges.push(Edge {
                source: src,
                target: dst,
                weight,
            });
        }
    }

    if edges.is_empty() && !nodes.is_empty() {
        CsrGraph::from_nodes_and_edges(nodes, edges).expect("valid graph")
    } else {
        CsrGraph::from_edges(edges).expect("valid graph")
    }
}

/// Helper checking if community subgraph is internally connected.
fn is_community_internally_connected<Id: NodeId>(
    graph: &CsrGraph<Id>,
    community_nodes: &[u32],
) -> bool {
    if community_nodes.len() <= 1 {
        return true;
    }

    let comm_set: HashSet<u32> = community_nodes.iter().copied().collect();
    let start = community_nodes[0];

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start);
    let _ = visited.insert(start);

    while let Some(u) = queue.pop_front() {
        for &v in graph.neighbours_of(u) {
            if comm_set.contains(&v) && visited.insert(v) {
                queue.push_back(v);
            }
        }
    }

    visited.len() == community_nodes.len()
}

#[test]
fn two_cliques_yields_two_communities() {
    let graph = load_edg("two_cliques.edg");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("algorithm runs successfully");

    assert!(
        result.quality > 0.4,
        "modularity {} must exceed 0.4",
        result.quality
    );

    let mut communities = HashSet::new();
    for (_node, comm) in &result.partition {
        let _ = communities.insert(*comm);
    }
    assert_eq!(
        communities.len(),
        2,
        "two cliques graph should produce exactly 2 communities"
    );
}

#[test]
fn two_cliques_communities_are_internally_connected() {
    let graph = load_edg("two_cliques.edg");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("algorithm runs successfully");

    let mut comm_to_nodes: std::collections::HashMap<u32, Vec<u32>> =
        std::collections::HashMap::new();
    for (user_id, comm) in &result.partition {
        if let Some(internal_id) = graph.internal_id(user_id) {
            comm_to_nodes.entry(*comm).or_default().push(internal_id);
        }
    }

    for (comm, nodes) in comm_to_nodes {
        assert!(
            is_community_internally_connected(&graph, &nodes),
            "community {comm} induced subgraph must be internally connected"
        );
    }
}
