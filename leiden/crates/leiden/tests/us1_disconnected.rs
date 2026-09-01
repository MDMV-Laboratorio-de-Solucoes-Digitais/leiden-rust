//! US1: Disconnected graph components receive separate communities.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args,
    clippy::unnecessary_debug_formatting,
    reason = "test code: assertions and diagnostic unwraps permitted per Constitution §III"
)]

use std::collections::HashSet;
use std::path::PathBuf;

use leiden::{CsrGraph, Edge, Leiden, LeidenParameters};

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
                parts[2].parse::<f64>().expect("valid float weight")
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

#[test]
fn disconnected_components_separated() {
    let graph = load_edg("disconnected_two_triangles.edg");
    let result = Leiden::new()
        .with_parameters(LeidenParameters {
            gamma: 1.0,
            seed: Some(0),
            iteration_cap: 10,
        })
        .run(&graph)
        .expect("runs on disconnected graph");

    // Two disjoint 3-cliques: triangle 1 is {0, 1, 2}, triangle 2 is {3, 4, 5}
    let mut triangle1_comms = HashSet::new();
    let mut triangle2_comms = HashSet::new();

    for (node, comm) in &result.partition {
        match node.as_str() {
            "0" | "1" | "2" => {
                let _ = triangle1_comms.insert(*comm);
            }
            "3" | "4" | "5" => {
                let _ = triangle2_comms.insert(*comm);
            }
            _ => {}
        }
    }

    // Communities must be disjoint between disconnected components
    for comm in &triangle1_comms {
        assert!(
            !triangle2_comms.contains(comm),
            "community {comm} contains nodes from both disconnected components"
        );
    }

    // Total distinct communities >= 2
    let all_comms: HashSet<u32> = result.partition.iter().map(|p| p.1).collect();
    assert!(
        all_comms.len() >= 2,
        "disconnected graph with 2 components must produce at least 2 communities"
    );
}
